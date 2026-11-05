//! Conduite de la voiture : moteur, direction, collisions, caméra, affichage de la voiture conduite.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::collide::{camera_clearance, floor_at, push_out};
use super::{VisitInput, Visitor, lerp_angle, yaw_facing};
use crate::editor::Editor;
use crate::model::{ElemRef, House, Part, Prop, Shape, prop_parts};

/// Voiture conduite : l'avant est vers +z local, `heading` en radians autour de la verticale.
#[derive(Clone, Copy, Debug)]
pub(super) struct Car {
    prop: usize,
    pos: Vec3,
    heading: f32,
    /// m/s, négatif en marche arrière.
    speed: f32,
    steer: f32,
    /// Rotation des roues (pour l'affichage).
    spin: f32,
    /// Temps sans bouger la souris : la caméra se recale alors derrière la voiture.
    look_idle: f32,
}

const MAX_SPEED: f32 = 33.0;
const MAX_REVERSE: f32 = 6.0;
const WHEELBASE: f32 = 2.7;
const WHEEL_RADIUS: f32 = 0.34;
const DRIVE_BOOM: f32 = 6.5;

fn car_tf(c: &Car) -> Transform {
    Transform::from_translation(c.pos).with_rotation(Quat::from_rotation_y(c.heading))
}

impl Car {
    fn forward(&self) -> Vec3 {
        Vec3::new(self.heading.sin(), 0.0, self.heading.cos())
    }

    /// Moteur, freins et direction (modèle « bicyclette »), puis collisions et sol.
    fn advance(&mut self, throttle: f32, steer_in: f32, handbrake: bool, solids: &[Transform], house: &House, dt: f32) {
        self.speed = car_speed(self.speed, throttle, handbrake, dt);
        // la direction braque moins à haute vitesse
        let max_steer = 0.6 / (1.0 + self.speed.abs() * 0.1);
        self.steer += (steer_in * max_steer - self.steer) * (1.0 - (-6.0 * dt).exp());
        self.heading += self.speed / WHEELBASE * self.steer.tan() * dt;
        let f = self.forward();
        let mut pos = self.pos + f * self.speed * dt;
        if collide_body(&mut pos, f, solids) && self.speed.abs() > 0.5 {
            // choc : la voiture rebondit et perd l'essentiel de sa vitesse
            self.speed *= -0.3;
        }
        pos.y = floor_at(pos, solids, house.ground(Vec2::new(pos.x, pos.z)));
        self.pos = pos;
        self.spin = (self.spin + self.speed * dt / WHEEL_RADIUS) % TAU;
    }
}

/// Carrosserie approchée par quatre cercles alignés sur sa longueur ; vrai si elle a touché.
fn collide_body(pos: &mut Vec3, f: Vec3, solids: &[Transform]) -> bool {
    let mut hit = false;
    for _ in 0..2 {
        for o in [-1.45, -0.5, 0.5, 1.45] {
            let c0 = *pos + f * o;
            let mut c = c0;
            for tf in solids {
                push_out(&mut c, tf, 0.85, 1.4);
            }
            if c != c0 {
                *pos += c - c0;
                hit = true;
            }
        }
    }
    hit
}

impl Visitor {
    /// Monte dans la voiture `i` du plan.
    pub(super) fn board(&mut self, i: usize, p: &Prop, ed: &mut Editor) {
        self.car = Some(Car {
            prop: i,
            pos: Vec3::new(p.pos.x, ed.house.prop_y(p), p.pos.y),
            heading: p.rot.to_radians(),
            speed: 0.0,
            steer: 0.0,
            spin: 0.0,
            look_idle: 2.0,
        });
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
        car.advance(inp.mv.y, -inp.mv.x, inp.handbrake, solids, &ed.house, dt);
        let cam = self.car_camera(&mut car, &ed.parts, &ed.house, inp.look, dt);
        self.speed_kmh = Some(car.speed.abs() * 3.6);
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
        self.fov += (0.8 + 0.25 * (car.speed.abs() / MAX_SPEED).min(1.0) - self.fov) * (1.0 - (-3.0 * dt).exp());
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

/// Laisse la voiture là où on l'a conduite et la rend à l'affichage normal.
pub(super) fn park(car: &Car, ed: &mut Editor) {
    if let Some(p) = ed.house.props.get_mut(car.prop) {
        p.pos = Vec2::new(car.pos.x, car.pos.z);
        p.rot = car.heading.to_degrees().rem_euclid(360.0);
    }
    ed.driving = None;
    ed.dirty = true;
}

#[derive(Component)]
pub struct CarBody;

#[derive(Component)]
pub struct CarWheel {
    base: Transform,
    front: bool,
}

impl CarWheel {
    /// Braquage (roues avant) et rotation de la roue autour de son axe.
    fn rotation(&self, car: &Car) -> Quat {
        let steer = if self.front { car.steer } else { 0.0 };
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
                e.insert(CarWheel { base, front: part.tf.translation.z > 0.0 });
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
        let car = Car {
            prop: i,
            pos: Vec3::new(20.0, 0.0, 5.0),
            heading: FRAC_PI_2,
            speed: 0.0,
            steer: 0.0,
            spin: 0.0,
            look_idle: 0.0,
        };
        park(&car, &mut ed);
        assert_eq!(ed.house.props[i].pos, Vec2::new(20.0, 5.0));
        assert!((ed.house.props[i].rot - 90.0).abs() < 1e-3);
        assert!(ed.driving.is_none() && ed.dirty);
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
