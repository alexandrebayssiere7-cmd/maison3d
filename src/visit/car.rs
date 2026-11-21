//! Conduite de la voiture : moteur, direction, collisions, caméra, affichage de la voiture conduite.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::collide::{camera_clearance, push_out, support_height};
use super::{VisitInput, Visitor, lerp_angle, yaw_facing};
use crate::command::Command;
use crate::editor::Editor;
use crate::model::{ElemRef, House, Part, Prop, Shape, prop_parts};
use crate::terrain::HALF;

/// Voiture conduite : l'avant est vers +z local, `heading` en radians autour de la verticale.
#[derive(Clone, Copy, Debug)]
pub(super) struct Car {
    prop: usize,
    /// Point de la caisse au niveau du sol, sous son centre.
    pos: Vec3,
    vel: Vec3,
    heading: f32,
    /// Tangage (nez vers le haut > 0) et roulis (côté +x vers le haut > 0), suivant le sol.
    pitch: f32,
    roll: f32,
    steer: f32,
    /// Rotation des roues (pour l'affichage).
    spin: f32,
    grounded: bool,
    /// Débattement de suspension de chaque roue (ordre de `WHEELS`), pour l'affichage.
    drop: [f32; 4],
    /// Temps sans bouger la souris : la caméra se recale alors derrière la voiture.
    look_idle: f32,
}

/// Commandes du conducteur : accélérateur (-1 frein / marche arrière .. 1), volant (1 = gauche), frein à main.
struct DriveInput {
    throttle: f32,
    steer: f32,
    handbrake: bool,
}

const MAX_SPEED: f32 = 33.0;
const MAX_REVERSE: f32 = 6.0;
const WHEELBASE: f32 = 2.7;
const TRACK: f32 = 1.64;
const WHEEL_RADIUS: f32 = 0.34;
/// Position (x, z) des roues dans le repère de la voiture : avant gauche, avant droite, arrière gauche, arrière droite.
const WHEELS: [(f32, f32); 4] = [(-0.82, 1.35), (0.82, 1.35), (-0.82, -1.35), (0.82, -1.35)];
const DRIVE_BOOM: f32 = 6.5;
const G: f32 = 9.81;

fn car_tf(c: &Car) -> Transform {
    let rot = Quat::from_rotation_y(c.heading) * Quat::from_rotation_x(-c.pitch) * Quat::from_rotation_z(c.roll);
    Transform::from_translation(c.pos).with_rotation(rot)
}

impl Car {
    fn new(prop: usize, pos: Vec3, heading: f32) -> Self {
        Car {
            prop,
            pos,
            vel: Vec3::ZERO,
            heading,
            pitch: 0.0,
            roll: 0.0,
            steer: 0.0,
            spin: 0.0,
            grounded: false,
            drop: [0.0; 4],
            look_idle: 2.0,
        }
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(self.heading.sin(), 0.0, self.heading.cos())
    }

    /// Vitesse au sol, en m/s.
    fn speed(&self) -> f32 {
        Vec2::new(self.vel.x, self.vel.z).length()
    }

    #[cfg(test)]
    fn forward_speed(&self) -> f32 {
        self.vel.dot(self.forward())
    }

    /// Un pas de conduite, découpé en petits pas : suspension et collisions restent stables même à 120 km/h.
    fn advance(&mut self, input: &DriveInput, solids: &[Transform], house: &House, dt: f32) {
        let support = |p: Vec2, y: f32| support_height(p, y, house.ground(p), solids);
        for _ in 0..4 {
            self.step(input, dt / 4.0, &support, solids);
        }
    }

    /// Un pas de physique : la caisse repose sur ses quatre roues qui suivent le sol (relief, dalles,
    /// routes), la gravité la fait retomber après une bosse, les pentes freinent ou accélèrent,
    /// les pneus adhèrent latéralement (sauf au frein à main) et les obstacles l'arrêtent.
    /// `support(xz, y)` donne la hauteur du sol franchissable sous un point.
    fn step(&mut self, input: &DriveInput, dt: f32, support: &dyn Fn(Vec2, f32) -> f32, solids: &[Transform]) {
        let ground_before = self.wheel_ground(self.pos, support).iter().sum::<f32>() / 4.0;
        if self.grounded {
            self.roll_on_ground(input, dt);
        }
        self.vel.y -= G * dt;
        let mut pos = self.pos + self.vel * dt;
        let pushed = collide_body(&mut pos, self.forward(), solids);
        self.bounce(pushed);
        let h = self.wheel_ground(pos, support);
        self.settle(pos, h, ground_before, dt);
        self.suspend(h, dt);
    }

    /// Hauteur du sol sous chaque roue, la caisse étant en `pos`.
    fn wheel_ground(&self, pos: Vec3, support: &dyn Fn(Vec2, f32) -> f32) -> [f32; 4] {
        let yaw = Quat::from_rotation_y(self.heading);
        WHEELS.map(|(x, z)| {
            let p = pos + yaw * Vec3::new(x, 0.0, z);
            support(Vec2::new(p.x, p.z), pos.y)
        })
    }

    /// Moteur, freins, pente, direction (modèle « bicyclette ») et adhérence des pneus.
    fn roll_on_ground(&mut self, input: &DriveInput, dt: f32) {
        let yaw = Quat::from_rotation_y(self.heading);
        let (mut v_f, mut v_s) = (self.vel.dot(yaw * Vec3::Z), self.vel.dot(yaw * Vec3::X));
        v_f = car_speed(v_f, input.throttle, input.handbrake, dt);
        // la pente tire la voiture vers le bas
        v_f -= G * self.pitch.sin() * dt;
        // la direction braque moins à haute vitesse
        let max_steer = 0.6 / (1.0 + v_f.abs() * 0.1);
        self.steer += (input.steer * max_steer - self.steer) * (1.0 - (-6.0 * dt).exp());
        self.heading += v_f / WHEELBASE * self.steer.tan() * dt;
        // adhérence latérale : forte, sauf roues bloquées au frein à main (dérapage)
        let grip = if input.handbrake { 2.5 } else { 14.0 } * dt;
        v_s -= v_s.clamp(-grip, grip);
        let yaw = Quat::from_rotation_y(self.heading);
        let h = yaw * Vec3::Z * v_f + yaw * Vec3::X * v_s;
        self.vel.x = h.x;
        self.vel.z = h.z;
        self.spin = (self.spin + v_f * dt / WHEEL_RADIUS) % TAU;
    }

    /// Choc contre un obstacle (repoussée de `pushed`) : on annule la vitesse vers lui avec un
    /// petit rebond, et on perd de l'élan.
    fn bounce(&mut self, pushed: Vec3) {
        if pushed.length_squared() <= 1e-10 {
            return;
        }
        let n = pushed.normalize();
        let vn = self.vel.dot(n);
        if vn < 0.0 {
            self.vel -= n * vn * 1.3;
            self.vel.x *= 0.8;
            self.vel.z *= 0.8;
        }
    }

    /// Contact avec le sol `h` (sous chaque roue) : jamais en dessous ; en montée la caisse est
    /// portée vers le haut, et décolle si le sol se dérobe d'un coup, en haut d'une bosse.
    fn settle(&mut self, mut pos: Vec3, h: [f32; 4], ground_before: f32, dt: f32) {
        // la roue la plus enfoncée porte la caisse : aucune ne passe sous le sol
        let tilted = car_tf(&Car { pos, ..*self });
        let deficit = WHEELS
            .iter()
            .zip(h)
            .map(|(&(x, z), g)| g - tilted.transform_point(Vec3::new(x, 0.0, z)).y)
            .fold(f32::NEG_INFINITY, f32::max);
        self.grounded = deficit >= -0.02;
        if self.grounded {
            // au contact, la caisse est plaquée sur ses roues (y compris en descente)
            pos.y += deficit;
            // à l'atterrissage la suspension absorbe la chute ; en montée on garde l'élan vertical
            let ground = h.iter().sum::<f32>() / 4.0;
            // dt nul (image figée) : 0 / 0 donnerait NaN, que `clamp` laisse passer
            self.vel.y = ((ground - ground_before) / dt.max(1e-4)).clamp(0.0, 12.0);
            let (front, back) = (f32::midpoint(h[0], h[1]), f32::midpoint(h[2], h[3]));
            let (left, right) = (f32::midpoint(h[0], h[2]), f32::midpoint(h[1], h[3]));
            let k = 1.0 - (-14.0 * dt).exp();
            self.pitch += (((front - back) / WHEELBASE).atan() - self.pitch) * k;
            self.roll += (((right - left) / TRACK).atan() - self.roll) * k;
        }
        self.pos = pos;
    }

    /// Chaque roue descend ou remonte pour toucher le sol `h` (suspension), dans la limite du débattement.
    fn suspend(&mut self, h: [f32; 4], dt: f32) {
        let body = car_tf(self);
        for (i, &(x, z)) in WHEELS.iter().enumerate() {
            let contact = body.transform_point(Vec3::new(x, 0.0, z));
            let target = if self.grounded { (h[i] - contact.y).clamp(-0.2, 0.2) } else { -0.15 };
            self.drop[i] += (target - self.drop[i]) * (1.0 - (-20.0 * dt).exp());
        }
    }
}

/// Carrosserie approchée par quatre cercles alignés sur sa longueur ; renvoie de combien elle a été repoussée.
fn collide_body(pos: &mut Vec3, f: Vec3, solids: &[Transform]) -> Vec3 {
    let mut pushed = Vec3::ZERO;
    for _ in 0..2 {
        for o in [-1.45, -0.5, 0.5, 1.45] {
            let c0 = *pos + f * o;
            let mut c = c0;
            for tf in solids {
                push_out(&mut c, tf, 0.85, 1.4);
            }
            *pos += c - c0;
            pushed += c - c0;
        }
    }
    pushed
}

impl Visitor {
    /// Monte dans la voiture `i` du plan.
    pub(super) fn board(&mut self, i: usize, p: &Prop, ed: &mut Editor) {
        self.car = Some(Car::new(i, Vec3::new(p.pos.x, ed.house.prop_y(p), p.pos.y), p.rot.to_radians()));
        ed.driving = Some(i);
        ed.dirty = true;
        self.boom_cur = 1.0;
    }

    /// Un pas de conduite : moteur, freins, direction, collisions, caméra, descente du véhicule.
    pub(super) fn drive(
        &mut self,
        mut car: Car,
        ed: &mut Editor,
        inp: &VisitInput,
        solids: &[Transform],
        dt: f32,
    ) -> Transform {
        let input = DriveInput { throttle: inp.mv.y, steer: -inp.mv.x, handbrake: inp.handbrake };
        car.advance(&input, solids, &ed.house, dt);
        // en quittant la visite, la caméra d'édition se recentre sur `pos`
        // a NaN position would break the editor camera until restart: keep the last finite one
        if car.pos.is_finite() {
            self.pos = car.pos;
        }
        let cam = self.car_camera(&mut car, &ed.parts, &ed.house, inp.look, dt);
        self.speed_kmh = Some(car.speed() * 3.6);
        self.target = None;
        self.actions = vec![
            "Z / S : accélérer, freiner, reculer",
            "Q / D : tourner  •  Espace : frein à main",
            "E : descendre",
            if self.third { "V : vue intérieure" } else { "V : vue extérieure" },
        ];
        if inp.key_e {
            self.get_out(&car);
            park(&car, ed);
        } else {
            self.car = Some(car);
        }
        cam
    }

    /// Caméra : libre à la souris, se recale derrière la voiture après une seconde.
    fn car_camera(&mut self, car: &mut Car, parts: &[Part], house: &House, mouse: Vec2, dt: f32) -> Transform {
        car.look_idle = if mouse.length() > 0.5 { 0.0 } else { car.look_idle + dt };
        if car.look_idle > 1.0 {
            let k = 1.0 - (-3.0 * dt).exp();
            self.yaw = lerp_angle(self.yaw, yaw_facing(car.forward()), k);
            self.pitch += (if self.third { -0.18 } else { -0.05 } - self.pitch) * k;
        }
        let look = self.look();
        let body = car_tf(car);
        self.fov += (0.8 + 0.25 * (car.speed() / MAX_SPEED).min(1.0) - self.fov) * (1.0 - (-3.0 * dt).exp());
        if !self.third {
            // place conducteur, à gauche
            return Transform::from_translation(body.transform_point(Vec3::new(0.4, 1.25, -0.25))).with_rotation(look);
        }
        let pivot = car.pos + Vec3::Y * 1.7;
        let desired = look * Vec3::new(0.0, 0.4, DRIVE_BOOM);
        let (len, dir) = (desired.length(), desired.normalize());
        let free = camera_clearance(parts, pivot, dir, len, house.ground(Vec2::new(pivot.x, pivot.z)));
        self.follow_boom(free, 3.0, dt);
        Transform::from_translation(pivot + dir * self.boom_cur).with_rotation(look)
    }

    /// Descend côté conducteur.
    fn get_out(&mut self, car: &Car) {
        let out = car_tf(car).transform_point(Vec3::new(1.5, 0.0, -0.2));
        self.pos = Vec3::new(out.x, car.pos.y + 0.05, out.z);
        self.root = self.pos;
        self.vel = Vec3::ZERO;
        self.body_yaw = self.yaw;
        self.root_yaw = self.yaw;
        self.speed_kmh = None;
        self.car = None;
    }
}

/// Nouvelle vitesse : moteur (qui s'essouffle vers la vitesse max), freins, frein à main, frottements.
fn car_speed(speed: f32, throttle: f32, handbrake: bool, dt: f32) -> f32 {
    // freiner jusqu'à l'arrêt complet, puis seulement repartir dans l'autre sens
    let forward = speed > 0.0;
    let backward = speed < 0.0;
    let braking = handbrake || throttle == 0.0 || (throttle > 0.0 && backward) || (throttle < 0.0 && forward);
    let accel = if handbrake {
        -12.0 * speed.signum()
    } else if throttle > 0.0 {
        if backward { 10.0 } else { 5.0 * (1.0 - speed / MAX_SPEED).max(0.0) }
    } else if throttle < 0.0 {
        if forward { -10.0 } else { -3.0 * (1.0 + speed / MAX_REVERSE).max(0.0) }
    } else {
        // roue libre : frottements + résistance de l'air
        -(1.2 + 0.012 * speed * speed) * speed.signum()
    };
    let new = speed + accel * dt;
    // un frein arrête la voiture mais ne la fait pas repartir dans l'autre sens
    if braking && new * speed <= 0.0 { 0.0 } else { new }
}

/// Laisse la voiture là où on l'a conduite (dans les limites du terrain, sinon le monde ne se
/// recharge plus) et la rend à l'affichage normal ; le stationnement s'annule. Une position
/// refusée par la validation (NaN) laisse la voiture à sa dernière place valide.
pub(super) fn park(car: &Car, ed: &mut Editor) {
    // f32::clamp garde NaN pour que `set` le refuse ; Vec2::clamp en ferait un coin du terrain
    let pos = Vec2::new(car.pos.x.clamp(-HALF, HALF), car.pos.z.clamp(-HALF, HALF));
    let rot = car.heading.to_degrees().rem_euclid(360.0);
    if ed.house.props.get(car.prop).is_some_and(|p| moved(p, pos, rot)) {
        let fields = serde_json::Map::from_iter([
            ("pos".to_owned(), serde_json::json!(pos)),
            ("rot".to_owned(), serde_json::json!(rot)),
        ]);
        // garer ne change aucun indice : le garage ouvert pour sortir la voiture reste ouvert
        let doors = std::mem::take(&mut ed.doors);
        // refus déjà affiché dans le message par Editor::apply ; la maison reste intacte
        let _ = ed.apply([Command::Set { target: ElemRef::Prop(car.prop), fields }]);
        ed.doors = doors;
    }
    ed.driving = None;
    ed.dirty = true;
}

/// La voiture a roulé ou tourné depuis qu'on y est monté, au-delà du tassement de la suspension.
fn moved(p: &Prop, pos: Vec2, rot: f32) -> bool {
    let turn = (rot - p.rot).rem_euclid(360.0);
    p.pos.distance(pos) > 0.01 || (0.5..359.5).contains(&turn)
}

#[derive(Component)]
pub struct CarBody;

#[derive(Component)]
pub struct CarWheel {
    base: Transform,
    /// Indice dans `WHEELS` (0 et 1 = roues avant, directrices).
    index: usize,
}

impl CarWheel {
    /// Braquage (roues avant) et rotation de la roue autour de son axe.
    fn rotation(&self, car: &Car) -> Quat {
        let steer = if self.index < 2 { car.steer } else { 0.0 };
        Quat::from_rotation_y(steer) * Quat::from_rotation_x(car.spin) * self.base.rotation
    }
}

/// Affiche la voiture conduite (créée à la montée, supprimée à la descente) et anime ses roues.
pub fn sync_car(
    mut commands: Commands,
    v: Res<Visitor>,
    ed: Res<Editor>,
    mut lib: ResMut<crate::render::Library>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut body: Query<(Entity, &mut Transform), (With<CarBody>, Without<CarWheel>)>,
    mut wheels: Query<(&CarWheel, &mut Transform), Without<CarBody>>,
) {
    let Some(car) = v.car else {
        if let Ok((e, _)) = body.single() {
            commands.entity(e).despawn();
        }
        return;
    };
    if let Ok((_, mut tf)) = body.single_mut() {
        *tf = car_tf(&car);
        for (w, mut t) in &mut wheels {
            t.rotation = w.rotation(&car);
            t.translation = w.base.translation + Vec3::Y * car.drop[w.index];
        }
    } else if let Some(p) = ed.house.props.get(car.prop) {
        spawn_car(&mut commands, &mut lib, &mut meshes, &mut materials, &car, p);
    }
}

/// Crée la voiture conduite, ses roues à part pour les animer.
fn spawn_car(
    commands: &mut Commands,
    lib: &mut crate::render::Library,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    car: &Car,
    p: &Prop,
) {
    let model = Prop { pos: Vec2::ZERO, level: 0, rot: 0.0, ..p.clone() };
    let parts = prop_parts(&model, ElemRef::Prop(car.prop), 0.0, 0.0);
    commands.spawn((CarBody, car_tf(car), Visibility::Visible)).with_children(|root| {
        for part in &parts {
            let bundle = crate::render::part_bundle(lib, meshes, materials, part);
            let base = bundle.2;
            let mut e = root.spawn(bundle);
            if part.shape == Shape::Wheel {
                let (x, z) = (part.tf.translation.x, part.tf.translation.z);
                e.insert(CarWheel { base, index: if z > 0.0 { 0 } else { 2 } + usize::from(x > 0.0) });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;
    use crate::model::{PropKind, demo_house};

    #[test]
    fn car_accelerates_brakes_and_never_creeps() {
        let dt = 1.0 / 60.0;
        let mut s = 0.0;
        for _ in 0..(60 * 5) {
            s = car_speed(s, 1.0, false, dt);
        }
        // 0 à ~70 km/h en 5 s, sans dépasser la vitesse max
        assert!(s * 3.6 > 55.0 && s < MAX_SPEED, "{} km/h", s * 3.6);
        let mut braking = s;
        let mut frames = 0;
        while braking > 0.0 {
            braking = car_speed(braking, -1.0, false, dt);
            frames += 1;
        }
        assert_eq!(braking, 0.0, "le frein ne fait pas reculer");
        assert!(frames < 60 * 3, "freinage trop long : {frames} images");
        // à l'arrêt sans accélérer : reste immobile
        assert_eq!(car_speed(0.0, 0.0, false, dt), 0.0);
        // marche arrière limitée
        let mut r = 0.0;
        for _ in 0..600 {
            r = car_speed(r, -1.0, false, dt);
        }
        assert!(r < 0.0 && r > -MAX_REVERSE);
    }

    /// Roule `secs` secondes plein gaz vers l'avant (+z) ; vérifie à chaque pas que la caisse
    /// et chaque roue restent au-dessus du sol. Renvoie la voiture et la hauteur max de vol.
    fn drive_on(ground: &dyn Fn(Vec2) -> f32, secs: f32, solids: &[Transform]) -> (Car, f32) {
        let mut car = Car::new(0, Vec3::new(0.0, ground(Vec2::ZERO), 0.0), 0.0);
        let input = DriveInput { throttle: 1.0, steer: 0.0, handbrake: false };
        let support = |p: Vec2, _y: f32| ground(p);
        let mut max_air: f32 = 0.0;
        for _ in 0..(secs * 240.0) as usize {
            car.step(&input, 1.0 / 240.0, &support, solids);
            let under = ground(Vec2::new(car.pos.x, car.pos.z));
            assert!(car.pos.y >= under - 0.06, "sous le sol : {} < {under}", car.pos.y);
            for (x, z) in WHEELS {
                let c = car_tf(&car).transform_point(Vec3::new(x, 0.0, z));
                assert!(c.y >= ground(Vec2::new(c.x, c.z)) - 0.25, "roue enterrée");
            }
            max_air = max_air.max(car.pos.y - under);
        }
        (car, max_air)
    }

    #[test]
    fn car_climbs_a_hill_tilted_and_never_clips() {
        // pente de 25 % (14°) qui commence à z = 5
        let hill = |p: Vec2| ((p.y - 5.0) * 0.25).max(0.0);
        let (car, _) = drive_on(&hill, 6.0, &[]);
        assert!(car.pos.z > 30.0, "n'a pas monté : {}", car.pos.z);
        assert!((car.pitch - 0.25f32.atan()).abs() < 0.03, "caisse pas inclinée : {}", car.pitch);
        assert!(car.forward_speed() < 25.0, "la pente doit freiner : {}", car.forward_speed());
    }

    #[test]
    fn car_jumps_off_a_ramp_and_lands() {
        // tremplin de 2 m entre z = 30 et z = 35, puis le sol retombe d'un coup
        let ramp = |p: Vec2| if (30.0..35.0).contains(&p.y) { (p.y - 30.0) * 0.4 } else { 0.0 };
        let (car, max_air) = drive_on(&ramp, 6.0, &[]);
        assert!(max_air > 0.5, "aucun saut : {max_air}");
        assert!(car.grounded && car.pos.y.abs() < 0.03, "pas retombé : {}", car.pos.y);
    }

    #[test]
    fn parked_car_on_flat_ground_stays_put_and_walls_stop_it() {
        let flat = |_: Vec2| 0.0;
        let mut car = Car::new(0, Vec3::ZERO, 0.0);
        let idle = DriveInput { throttle: 0.0, steer: 0.0, handbrake: false };
        for _ in 0..240 {
            car.step(&idle, 1.0 / 120.0, &|p, _| flat(p), &[]);
        }
        assert!(car.vel.length() < 1e-3 && car.pos.length() < 1e-3, "{:?} {:?}", car.pos, car.vel);
        // mur de 3 m de haut en travers à z = 15
        let wall = [Transform::from_xyz(0.0, 1.5, 15.0).with_scale(Vec3::new(20.0, 3.0, 0.3))];
        let (car, _) = drive_on(&flat, 5.0, &wall);
        assert!(car.pos.z + 1.45 < 15.0, "a traversé le mur : {}", car.pos.z);
        assert!(car.forward_speed() < 5.0);
    }

    #[test]
    fn car_parts_and_parking() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        let i = ed.house.props.iter().position(|p| p.kind == PropKind::Car).unwrap();
        let wheels = prop_parts(&ed.house.props[i], ElemRef::Prop(i), 0.0, 0.0)
            .iter()
            .filter(|p| p.shape == Shape::Wheel)
            .count();
        assert_eq!(wheels, 4);
        ed.driving = Some(i);
        let car = Car::new(i, Vec3::new(20.0, 0.0, 5.0), FRAC_PI_2);
        park(&car, &mut ed);
        assert_eq!(ed.house.props[i].pos, Vec2::new(20.0, 5.0));
        assert!((ed.house.props[i].rot - 90.0).abs() < 1e-3);
        assert!(ed.driving.is_none() && ed.dirty);
    }

    #[test]
    fn car_parked_off_the_terrain_stays_inside_and_can_be_undone() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        let i = ed.house.props.iter().position(|p| p.kind == PropKind::Car).unwrap();
        // voiture disparue du plan : rien à garer, aucun point d'annulation
        ed.driving = Some(usize::MAX);
        park(&Car::new(usize::MAX, Vec3::ZERO, 0.0), &mut ed);
        assert!(!ed.can_undo() && ed.driving.is_none());
        let before = ed.house.props[i].pos;
        park(&Car::new(i, Vec3::new(90.0, 0.0, -120.0), 0.0), &mut ed);
        let path = std::env::temp_dir().join(format!("maison3d-park-{}.json", std::process::id()));
        ed.house.save(&path).unwrap();
        let loaded = House::load(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded.unwrap(), ed.house, "le monde enregistré se recharge");
        assert_eq!(ed.house.props[i].pos, Vec2::new(HALF, -HALF));
        assert!(ed.can_undo(), "garer la voiture crée un point d'annulation");
        ed.undo();
        assert_eq!(ed.house.props[i].pos, before);
    }

    #[test]
    fn parking_keeps_open_doors_and_skips_a_car_that_did_not_move() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        let i = ed.house.props.iter().position(|p| p.kind == PropKind::Car).unwrap();
        let p = ed.house.props[i].clone();
        // monté puis descendu sans rouler : la suspension a juste tassé la caisse
        let still = Car::new(i, Vec3::new(p.pos.x + 0.002, 0.0, p.pos.y), (p.rot + 360.1).to_radians());
        park(&still, &mut ed);
        assert!(!ed.can_undo(), "aucun point d'annulation sans déplacement");
        assert_eq!(ed.house.props[i], p);
        // le garage ouvert pour sortir la voiture reste ouvert une fois garée
        let door = ElemRef::Opening(0, 0);
        ed.toggle_door(door);
        park(&Car::new(i, Vec3::new(p.pos.x + 5.0, 0.0, p.pos.y), 0.0), &mut ed);
        assert!(ed.can_undo());
        assert_eq!(ed.doors[&door], (0.0, true), "la porte reste ouverte");
    }

    #[test]
    fn a_car_with_a_nan_position_or_heading_stays_at_its_last_valid_place() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        let i = ed.house.props.iter().position(|p| p.kind == PropKind::Car).unwrap();
        let p = ed.house.props[i].clone();
        let lost = [
            Car::new(i, Vec3::new(f32::NAN, 0.0, f32::NAN), 0.0),
            Car::new(i, Vec3::new(p.pos.x + 5.0, 0.0, p.pos.y), f32::NAN),
        ];
        for car in lost {
            ed.driving = Some(i);
            park(&car, &mut ed);
            assert!(ed.house.validate().is_ok(), "{:?}", ed.house.validate());
            assert_eq!(ed.house.props[i], p, "la voiture reste à sa dernière place valide");
            assert!(ed.driving.is_none() && !ed.can_undo());
        }
        let path = std::env::temp_dir().join(format!("maison3d-park-nan-{}.json", std::process::id()));
        ed.house.save(&path).unwrap();
        let loaded = House::load(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded.unwrap(), ed.house, "le monde enregistré se recharge");
    }

    #[test]
    fn a_car_with_a_nan_position_does_not_move_the_visitor() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        let mut v = Visitor { pos: Vec3::new(1.0, 0.0, 2.0), ..default() };
        let car = Car::new(0, Vec3::new(f32::NAN, 0.0, f32::NAN), 0.0);
        v.drive(car, &mut ed, &VisitInput::default(), &[], 1.0 / 60.0);
        assert_eq!(v.pos, Vec3::new(1.0, 0.0, 2.0), "the editor camera must not inherit NaN");
    }

    #[test]
    fn advance_with_zero_dt_keeps_the_car_finite() {
        let house = demo_house();
        let mut car = Car::new(0, Vec3::new(0.0, house.ground(Vec2::ZERO), 0.0), 0.0);
        let input = DriveInput { throttle: 1.0, steer: 0.0, handbrake: false };
        car.advance(&input, &[], &house, 0.0);
        assert!(car.pos.is_finite() && car.vel.is_finite(), "{:?} {:?}", car.pos, car.vel);
    }

    #[test]
    fn driven_car_is_shown_with_turning_wheels_then_removed() {
        let mut world = World::new();
        world.insert_resource(crate::render::Library::empty());
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let mut ed = Editor::default();
        ed.house = demo_house();
        let i = ed.house.props.iter().position(|p| p.kind == PropKind::Car).unwrap();
        let mut v = Visitor::default();
        v.board(i, &ed.house.props[i].clone(), &mut ed);
        world.insert_resource(ed);
        world.insert_resource(v);
        let id = world.register_system(sync_car);
        world.run_system(id).unwrap();
        assert_eq!(world.query::<&CarWheel>().iter(&world).count(), 4);
        world.resource_mut::<Visitor>().car.as_mut().unwrap().spin = 1.0;
        world.run_system(id).unwrap();
        let turned = world.query::<(&CarWheel, &Transform)>().iter(&world).all(|(w, t)| t.rotation != w.base.rotation);
        assert!(turned, "les roues tournent");
        world.resource_mut::<Visitor>().car = None;
        world.run_system(id).unwrap();
        assert_eq!(world.query::<&CarBody>().iter(&world).count(), 0);
    }
}
