//! Visite : un personnage articulé, vu à la 1re ou à la 3e personne, qui marche, court, saute,
//! ouvre les portes et placards, s'assoit et s'allonge (sur les meubles ou par terre).

mod avatar;
mod car;
mod collide;
mod interact;

use std::f32::consts::{FRAC_PI_4, TAU};

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::camera::Projection;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub use avatar::{AvatarRoot, animate_avatar, spawn_avatar};
use avatar::{Pose, eyes, target_pose};
pub use car::sync_car;
use car::{Car, park};
use collide::{camera_clearance, collider, floor_at, push_out, solid};
use interact::yaw_facing;

use crate::camera::CamState;
use crate::editor::Editor;
use crate::model::{ElemRef, House, Part, elev};
use crate::render::MainCamera;

/// Gabarit du visiteur : rayon, taille, marche franchissable.
const RADIUS: f32 = 0.25;
const HEIGHT: f32 = 1.75;
const STEP: f32 = 0.45;
/// Distance max (depuis la tête) pour interagir avec un objet.
const REACH: f32 = 2.5;

// vitesses réalistes : marche ~1,9 m/s, course ~5,6 m/s, avec inertie
const WALK_SPEED: f32 = 1.9;
const RUN_SPEED: f32 = 5.6;
const GROUND_ACCEL: f32 = 8.0;
const GROUND_DECEL: f32 = 14.0;
const AIR_ACCEL: f32 = 2.0;
const GRAVITY: f32 = 14.0;
const JUMP_SPEED: f32 = 4.6;
const HIPS_Y: f32 = 0.95;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stance {
    Stand,
    Sit,
    SitFloor,
    Lie,
}

#[derive(Resource)]
pub struct Visitor {
    pub active: bool,
    /// Pieds du personnage (position physique).
    pub pos: Vec3,
    vel: Vec3,
    grounded: bool,
    pub yaw: f32,
    pitch: f32,
    body_yaw: f32,
    pub third: bool,
    /// Distance voulue de la caméra 3e personne (fixe) et distance réelle (raccourcie par les murs).
    boom: f32,
    boom_cur: f32,
    stance: Stance,
    /// Point d'appui (siège, lit, sol) et orientation quand on est assis ou allongé.
    anchor: Vec3,
    anchor_yaw: f32,
    /// Où se relever.
    stand_pos: Vec3,
    pose: Pose,
    phase: f32,
    dip: f32,
    fov: f32,
    /// Position et orientation affichées du personnage (lissées).
    root: Vec3,
    root_yaw: f32,
    pub target: Option<ElemRef>,
    /// Actions possibles, affichées sous le viseur.
    pub actions: Vec<&'static str>,
    car: Option<Car>,
    /// Vitesse affichée au compteur pendant la conduite.
    pub speed_kmh: Option<f32>,
}

impl Default for Visitor {
    fn default() -> Self {
        Self {
            active: false,
            pos: Vec3::ZERO,
            vel: Vec3::ZERO,
            grounded: false,
            yaw: 0.0,
            pitch: 0.0,
            body_yaw: 0.0,
            third: false,
            boom: 3.0,
            boom_cur: 3.0,
            stance: Stance::Stand,
            anchor: Vec3::ZERO,
            anchor_yaw: 0.0,
            stand_pos: Vec3::ZERO,
            pose: Pose { hips_y: HIPS_Y, ..default() },
            phase: 0.0,
            dip: 0.0,
            fov: FRAC_PI_4,
            root: Vec3::ZERO,
            root_yaw: 0.0,
            target: None,
            actions: vec![],
            car: None,
            speed_kmh: None,
        }
    }
}

// ---------------------------------------------------------------- entrées

/// Entrées d'une image de visite, lues au clavier et à la souris.
#[derive(Clone, Copy, Default, Debug)]
struct VisitInput {
    /// Mouvement de la souris, déjà multiplié par la sensibilité.
    look: Vec2,
    /// Glisser à deux doigts sur le trackpad.
    scroll: Option<Vec2>,
    /// ZQSD : x vers la droite, y vers l'avant.
    mv: Vec2,
    run: bool,
    /// Espace : appui (saut, se relever) et maintien (frein à main).
    jump: bool,
    handbrake: bool,
    /// E ou clic : action principale ; E seul pour descendre de voiture.
    act: bool,
    key_e: bool,
    sit: bool,
    lie: bool,
    switch_view: bool,
}

fn read_input(
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    motion: Vec2,
    scroll: &AccumulatedMouseScroll,
    sensitivity: f32,
) -> VisitInput {
    let key = |k: KeyCode| if keys.pressed(k) { 1.0 } else { 0.0 };
    VisitInput {
        look: motion * sensitivity,
        scroll: (scroll.unit == MouseScrollUnit::Pixel).then_some(scroll.delta),
        mv: Vec2::new(key(KeyCode::KeyD) - key(KeyCode::KeyA), key(KeyCode::KeyW) - key(KeyCode::KeyS)),
        run: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        jump: keys.just_pressed(KeyCode::Space),
        handbrake: keys.pressed(KeyCode::Space),
        act: keys.just_pressed(KeyCode::KeyE) || mouse.just_pressed(MouseButton::Left),
        key_e: keys.just_pressed(KeyCode::KeyE),
        sit: keys.just_pressed(KeyCode::KeyC),
        lie: keys.just_pressed(KeyCode::KeyX),
        switch_view: keys.just_pressed(KeyCode::KeyV),
    }
}

fn grab_cursor(window: &mut Window, grab: bool) {
    window.cursor_options.grab_mode = if grab { CursorGrabMode::Locked } else { CursorGrabMode::None };
    window.cursor_options.visible = !grab;
}

fn set_fov(proj: &mut Projection, fov: f32) {
    if let Projection::Perspective(p) = proj {
        p.fov = fov;
    }
}

// ---------------------------------------------------------------- logique de visite

pub fn visit_update(
    mut v: ResMut<Visitor>,
    mut ed: ResMut<Editor>,
    cs: Res<CamState>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cam: Single<(&mut Transform, &mut Projection), (With<MainCamera>, Without<AvatarRoot>)>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    settings: Res<crate::menu::Settings>,
) {
    let v = &mut *v;
    let (cam_tf, proj) = &mut *cam;
    if !ed.walk {
        if v.leave(&mut ed) {
            grab_cursor(&mut window, false);
            set_fov(proj, FRAC_PI_4);
        }
        return;
    }
    if !v.active {
        let start = ed.walk_start.take().unwrap_or(Vec3::new(cs.focus.x, elev(ed.level), cs.focus.z));
        v.enter(start, cs.yaw);
        // souris capturée : on regarde autour de soi sans cliquer
        grab_cursor(&mut window, true);
    }
    let input = read_input(&keys, &mouse, motion.delta, &scroll, settings.sensitivity);
    **cam_tf = v.step(&mut ed, &input, time.delta_secs().min(0.05), time.elapsed_secs());
    set_fov(proj, v.fov);
}

impl Visitor {
    /// Entre en visite, les pieds en `start`, regard selon `yaw`.
    fn enter(&mut self, start: Vec3, yaw: f32) {
        *self = Visitor { third: self.third, boom: self.boom, ..default() };
        self.active = true;
        self.pos = start + Vec3::Y * 0.05;
        (self.yaw, self.body_yaw, self.root_yaw) = (yaw, yaw, yaw);
        self.root = self.pos;
        self.boom_cur = self.boom;
    }

    /// Retour à l'édition : gare la voiture ; vrai si la visite était en cours.
    fn leave(&mut self, ed: &mut Editor) -> bool {
        if let Some(car) = self.car.take() {
            park(&car, ed);
            self.speed_kmh = None;
        }
        if !self.active {
            return false;
        }
        self.active = false;
        self.target = None;
        true
    }

    fn look(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    /// Avant et droite à l'horizontale, selon le regard.
    fn axes(&self) -> (Vec3, Vec3) {
        let (sy, cy) = self.yaw.sin_cos();
        (Vec3::new(-sy, 0.0, -cy), Vec3::new(cy, 0.0, -sy))
    }

    /// Une image de visite ; renvoie la position de la caméra.
    fn step(&mut self, ed: &mut Editor, inp: &VisitInput, dt: f32, t: f32) -> Transform {
        self.look_around(inp);
        let solids: Vec<Transform> = ed.parts.iter().filter(|p| solid(p)).map(collider).collect();
        if let Some(car) = self.car {
            return self.drive(car, ed, inp, &solids, dt);
        }
        let speed = if self.stance == Stance::Stand { self.walk(inp, &solids, &ed.house, dt) } else { 0.0 };
        self.dip = (self.dip - dt * 0.5).max(0.0);
        self.update_body(speed, dt, t);
        let cam = self.camera(&ed.parts, &ed.house, speed, dt);
        self.interact(ed, inp, &cam);
        cam
    }

    /// Regard et options de vue.
    fn look_around(&mut self, inp: &VisitInput) {
        self.yaw -= inp.look.x * 0.0025;
        self.pitch = (self.pitch - inp.look.y * 0.0025).clamp(-1.45, 1.45);
        if inp.switch_view {
            self.third = !self.third;
        }
        // glisser à deux doigts sur le trackpad tourne aussi la vue (même sens qu'en édition) ;
        // la distance de la caméra reste fixe : pas de zoom
        if let Some(s) = inp.scroll {
            self.yaw -= s.x * 0.004;
            self.pitch = (self.pitch + s.y * 0.004).clamp(-1.45, 1.45);
        }
        if self.third {
            // au-delà, la caméra passerait sous le sol et se collerait au personnage
            self.pitch = self.pitch.clamp(-1.2, 0.55);
        }
    }

    /// Déplacement debout avec inertie ; renvoie la vitesse horizontale.
    fn walk(&mut self, inp: &VisitInput, solids: &[Transform], house: &House, dt: f32) -> f32 {
        let (fwd, right) = self.axes();
        let input = (fwd * inp.mv.y + right * inp.mv.x).normalize_or_zero();
        // on ne sprinte qu'en avançant ; à reculer on marche
        let running = inp.run && inp.mv.y > 0.0;
        let want = input * if running { RUN_SPEED } else { WALK_SPEED };
        let mut hv = Vec3::new(self.vel.x, 0.0, self.vel.z);
        let accel = match (self.grounded, want.length() >= hv.length()) {
            (false, _) => AIR_ACCEL,
            (true, true) => GROUND_ACCEL,
            (true, false) => GROUND_DECEL,
        };
        hv += (want - hv).clamp_length_max(accel * dt);
        let before = self.pos;
        let mut pos = self.pos + hv * dt;
        for _ in 0..3 {
            for tf in solids {
                push_out(&mut pos, tf, RADIUS, HEIGHT);
            }
        }
        // la vitesse perdue contre un mur ne s'accumule pas
        let moved = (pos - before) / dt.max(1e-4);
        self.vel.x = moved.x;
        self.vel.z = moved.z;
        self.fall(&mut pos, inp.jump, solids, house, dt);
        self.pos = pos;
        let speed = Vec2::new(self.vel.x, self.vel.z).length();
        if self.grounded {
            // un cycle = deux pas ; la foulée s'allonge en courant
            self.phase = (self.phase + speed * dt * TAU / (1.3 + 1.1 * run_amount(speed))) % TAU;
        }
        // le corps se tourne avec la caméra : en 3e personne on voit le personnage pivoter en douceur
        self.body_yaw =
            if self.third { lerp_angle(self.body_yaw, self.yaw, 1.0 - (-12.0 * dt).exp()) } else { self.yaw };
        speed
    }

    /// Saut, gravité et réception sur le sol.
    fn fall(&mut self, pos: &mut Vec3, jump: bool, solids: &[Transform], house: &House, dt: f32) {
        if self.grounded && jump {
            self.vel.y = JUMP_SPEED;
            self.grounded = false;
        }
        self.vel.y -= GRAVITY * dt;
        pos.y += self.vel.y * dt;
        let floor = floor_at(*pos, solids, house.ground(Vec2::new(pos.x, pos.z)));
        let was_grounded = self.grounded;
        self.grounded = pos.y <= floor;
        if !self.grounded {
            return;
        }
        if !was_grounded && self.vel.y < -2.5 {
            // réception : les jambes amortissent, on perd un peu d'élan
            self.dip = (-self.vel.y * 0.03).min(0.2);
            self.vel.x *= 0.7;
            self.vel.z *= 0.7;
        }
        pos.y = floor;
        self.vel.y = 0.0;
    }

    /// Personnage affiché : rejoint en douceur le siège, le lit ou la position debout.
    fn update_body(&mut self, speed: f32, dt: f32, t: f32) {
        let standing = self.stance == Stance::Stand;
        let (root_target, yaw_target) =
            if standing { (self.pos, self.body_yaw) } else { (self.anchor, self.anchor_yaw) };
        let k = if self.root.distance(root_target) > 0.25 { 1.0 - (-6.0 * dt).exp() } else { 1.0 };
        self.root = self.root.lerp(root_target, k);
        let turn = if standing && !self.third { 1.0 } else { 1.0 - (-8.0 * dt).exp() };
        self.root_yaw = lerp_angle(self.root_yaw, yaw_target, turn);
        let target = target_pose(self, speed, t);
        let rate = if standing { 18.0 } else { 5.0 };
        self.pose.blend(&target, 1.0 - (-rate * dt).exp());
    }

    /// Caméra à la 1re ou à la 3e personne ; le champ de vision s'élargit en courant.
    fn camera(&mut self, parts: &[Part], house: &House, speed: f32, dt: f32) -> Transform {
        let (eye, look) = (eyes(self), self.look());
        self.fov += (0.82 + 0.13 * run_amount(speed) - self.fov) * (1.0 - (-4.0 * dt).exp());
        if !self.third {
            return Transform::from_translation(eye).with_rotation(look);
        }
        let pivot = eye + Vec3::Y * 0.1;
        let desired = look * Vec3::new(0.35, 0.15, self.boom);
        let (len, dir) = (desired.length(), desired.normalize());
        let free = camera_clearance(parts, pivot, dir, len, house.ground(Vec2::new(pivot.x, pivot.z)));
        self.follow_boom(free, 4.0, dt);
        Transform::from_translation(pivot + dir * self.boom_cur).with_rotation(look)
    }

    /// On se rapproche tout de suite d'un obstacle, on s'en écarte doucement.
    fn follow_boom(&mut self, free: f32, rate: f32, dt: f32) {
        self.boom_cur = if free < self.boom_cur {
            free
        } else {
            self.boom_cur + (free - self.boom_cur) * (1.0 - (-rate * dt).exp())
        };
    }
}

/// 0 en marchant, 1 en courant à pleine vitesse.
fn run_amount(speed: f32) -> f32 {
    ((speed - WALK_SPEED) / (RUN_SPEED - WALK_SPEED)).clamp(0.0, 1.0)
}

fn lerp_angle(a: f32, b: f32, k: f32) -> f32 {
    let d = (b - a + std::f32::consts::PI).rem_euclid(TAU) - std::f32::consts::PI;
    a + d * k
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Prop, PropKind, demo_house};

    fn solids(open: f32) -> Vec<Transform> {
        let h = demo_house();
        h.parts(Some(0), &|e| if e == ElemRef::Opening(0, 0) { open } else { 0.0 })
            .iter()
            .filter(|p| solid(p))
            .map(collider)
            .collect()
    }

    /// Marche tout droit de `from` vers +z pendant `secs` secondes.
    fn walk(from: Vec3, secs: f32, s: &[Transform]) -> Vec3 {
        let mut pos = from;
        for _ in 0..(secs * 60.0) as usize {
            pos += Vec3::Z * 3.0 / 60.0;
            for _ in 0..3 {
                for tf in s {
                    push_out(&mut pos, tf, RADIUS, HEIGHT);
                }
            }
            pos.y = floor_at(pos, s, 0.0);
        }
        pos
    }

    #[test]
    fn run_amount_goes_from_walk_to_run() {
        assert_eq!(run_amount(0.0), 0.0);
        assert_eq!(run_amount(WALK_SPEED), 0.0);
        assert!((run_amount(f32::midpoint(WALK_SPEED, RUN_SPEED)) - 0.5).abs() < 1e-6);
        assert_eq!(run_amount(RUN_SPEED * 2.0), 1.0);
    }

    #[test]
    fn walls_block_and_open_doors_let_through() {
        // devant la façade sud (mur en z = 0), fenêtre en x = 1.2 : allège pleine, on bute
        let blocked = walk(Vec3::new(1.2, 0.0, -2.0), 2.0, &solids(0.0));
        assert!(blocked.z < -0.2, "traverse le mur : {blocked}");
        // porte d'entrée en x = 3 : fermée on bute, ouverte on entre
        assert!(walk(Vec3::new(3.0, 0.0, -2.0), 2.0, &solids(0.0)).z < -0.2);
        let inside = walk(Vec3::new(3.1, 0.0, -2.0), 2.0, &solids(1.0));
        assert!(inside.z > 1.0, "porte ouverte infranchissable : {inside}");
    }

    #[test]
    fn stands_on_floors_and_climbs_stairs() {
        let s = solids(0.0);
        assert!((floor_at(Vec3::new(5.0, 0.0, 4.0), &s, 0.0) - 0.02).abs() < 1e-3, "sur le parquet");
        let mut h = demo_house();
        h.props.clear();
        h.props.push(Prop::new(PropKind::Stairs, Vec2::new(20.0, 0.0), 0, 0.0));
        let stairs: Vec<Transform> = h.parts_closed(None).iter().filter(|p| solid(p)).map(collider).collect();
        // 1,6 s à 3 m/s : on s'arrête sur les dernières marches
        let top = walk(Vec3::new(20.0, 0.0, -3.0), 1.6, &stairs);
        assert!(top.y > 2.0, "n'a pas monté l'escalier : {top}");
    }

    fn prop(kind: PropKind, x: f32, z: f32) -> Prop {
        Prop { color: [1.0; 3], texture: 0, ..Prop::new(kind, Vec2::new(x, z), 0, 0.0) }
    }

    /// Un éditeur dont la maison ne contient que `props`, pièces à jour.
    fn scene(props: Vec<Prop>) -> Editor {
        let mut ed = Editor::default();
        ed.house.props = props;
        ed.parts = ed.house.parts_closed(None);
        ed
    }

    fn run(v: &mut Visitor, ed: &mut Editor, inp: VisitInput, frames: usize) -> Transform {
        let mut cam = Transform::default();
        for i in 0..frames {
            cam = v.step(ed, &inp, 1.0 / 60.0, i as f32 / 60.0);
        }
        cam
    }

    fn entered(at: Vec3) -> Visitor {
        let mut v = Visitor::default();
        v.enter(at, 0.0);
        v
    }

    #[test]
    fn reads_keyboard_and_mouse() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::KeyE);
        let mouse = ButtonInput::<MouseButton>::default();
        let scroll = AccumulatedMouseScroll { unit: MouseScrollUnit::Pixel, delta: Vec2::X };
        let inp = read_input(&keys, &mouse, Vec2::new(2.0, 1.0), &scroll, 2.0);
        assert_eq!(inp.mv, Vec2::Y);
        assert_eq!(inp.look, Vec2::new(4.0, 2.0));
        assert_eq!(inp.scroll, Some(Vec2::X));
        assert!(inp.run && inp.act && inp.key_e && !inp.jump && !inp.sit);
        let lines = AccumulatedMouseScroll { unit: MouseScrollUnit::Line, delta: Vec2::X };
        assert_eq!(read_input(&keys, &mouse, Vec2::new(2.0, 1.0), &lines, 1.0).scroll, None);
    }

    #[test]
    fn enter_and_leave_the_visit() {
        let mut ed = Editor::default();
        let mut v = Visitor { third: true, ..default() };
        v.enter(Vec3::new(1.0, 0.0, 2.0), 0.5);
        assert!(v.active && v.third);
        assert_eq!(v.pos, Vec3::new(1.0, 0.05, 2.0));
        assert_eq!(v.root_yaw, 0.5);
        assert!(v.leave(&mut ed));
        assert!(!v.active && v.target.is_none());
        assert!(!v.leave(&mut ed), "déjà sorti");
    }

    #[test]
    fn looks_around_and_switches_view() {
        let mut v = entered(Vec3::ZERO);
        v.look_around(&VisitInput { look: Vec2::new(100.0, -10_000.0), ..default() });
        assert!((v.yaw + 0.25).abs() < 1e-5);
        assert_eq!(v.pitch, 1.45);
        v.look_around(&VisitInput { switch_view: true, scroll: Some(Vec2::new(10.0, 0.0)), ..default() });
        assert!(v.third);
        assert_eq!(v.pitch, 0.55, "en 3e personne la caméra ne passe pas sous le sol");
        assert!((v.yaw + 0.29).abs() < 1e-5);
    }

    #[test]
    fn walks_runs_and_jumps() {
        let mut ed = scene(vec![]);
        let mut v = entered(Vec3::ZERO);
        let fwd = VisitInput { mv: Vec2::Y, ..default() };
        run(&mut v, &mut ed, fwd, 60);
        assert!(v.pos.z < -1.0 && v.pos.z > -2.0, "marche à ~1,9 m/s vers -z : {}", v.pos);
        assert!(v.actions.contains(&"C : s'asseoir par terre"));
        let start = v.pos.z;
        run(&mut v, &mut ed, VisitInput { run: true, ..fwd }, 60);
        assert!(start - v.pos.z > 3.5, "court : {}", start - v.pos.z);
        assert!(v.grounded);
        run(&mut v, &mut ed, VisitInput { jump: true, ..default() }, 10);
        assert!(v.pos.y > 0.3 && !v.grounded, "saute : {}", v.pos);
        run(&mut v, &mut ed, VisitInput::default(), 120);
        assert!(v.grounded && v.pos.y.abs() < 1e-3);
        assert!(v.fov > 0.8);
    }

    #[test]
    fn sits_and_lies_on_the_floor_then_gets_up() {
        let mut ed = scene(vec![]);
        let mut v = entered(Vec3::ZERO);
        run(&mut v, &mut ed, VisitInput::default(), 5);
        run(&mut v, &mut ed, VisitInput { sit: true, ..default() }, 1);
        assert_eq!(v.stance, Stance::SitFloor);
        run(&mut v, &mut ed, VisitInput::default(), 30);
        assert_eq!(v.actions[0], "ZQSD, Espace ou E : se relever");
        run(&mut v, &mut ed, VisitInput { jump: true, ..default() }, 1);
        assert_eq!(v.stance, Stance::Stand);
        run(&mut v, &mut ed, VisitInput { lie: true, ..default() }, 1);
        assert_eq!(v.stance, Stance::Lie);
    }

    #[test]
    fn sits_on_the_chair_in_front() {
        let mut ed = scene(vec![prop(PropKind::Chair, 0.0, 0.0)]);
        let mut v = entered(Vec3::new(0.0, 0.0, 1.0));
        v.pitch = -0.9;
        run(&mut v, &mut ed, VisitInput::default(), 3);
        assert_eq!(v.target, Some(ElemRef::Prop(0)));
        assert!(v.actions.contains(&"E : s'asseoir") && v.actions.contains(&"C : s'asseoir ici"));
        run(&mut v, &mut ed, VisitInput { act: true, ..default() }, 1);
        assert_eq!(v.stance, Stance::Sit);
        assert!((v.anchor.y - 0.48).abs() < 1e-3);
    }

    #[test]
    fn third_person_camera_backs_off_behind_the_head() {
        let mut ed = scene(vec![]);
        let mut v = entered(Vec3::ZERO);
        v.third = true;
        let cam = run(&mut v, &mut ed, VisitInput::default(), 30);
        assert!(cam.translation.z > 2.0, "caméra derrière : {}", cam.translation);
    }

    #[test]
    fn drives_the_car_and_gets_out() {
        let mut ed = scene(vec![prop(PropKind::Car, 0.0, 0.0)]);
        let mut v = entered(Vec3::new(0.0, 0.0, 3.5));
        v.pitch = -0.4;
        run(&mut v, &mut ed, VisitInput::default(), 3);
        assert!(v.actions.contains(&"E : conduire"), "{:?}", v.actions);
        run(&mut v, &mut ed, VisitInput { act: true, ..default() }, 1);
        assert_eq!(ed.driving, Some(0));
        // la voiture conduite n'est plus dans les pièces affichées
        ed.parts.clear();
        run(&mut v, &mut ed, VisitInput { mv: Vec2::Y, ..default() }, 120);
        assert!(v.speed_kmh.is_some_and(|s| s > 10.0), "{:?}", v.speed_kmh);
        v.third = true;
        let cam = run(&mut v, &mut ed, VisitInput { handbrake: true, ..default() }, 120);
        assert!(v.speed_kmh.is_some_and(|s| s < 1.0));
        assert!(cam.translation.y > 1.0);
        run(&mut v, &mut ed, VisitInput { key_e: true, ..default() }, 1);
        assert!(v.car.is_none() && ed.driving.is_none());
        assert!(ed.house.props[0].pos.y > 3.0, "voiture garée plus loin : {}", ed.house.props[0].pos);
    }

    #[test]
    fn opens_the_door_in_front() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        ed.parts = ed.house.parts_closed(Some(0));
        // porte d'entrée en x = 3 sur la façade sud (z = 0), vue de dehors
        let mut v = entered(Vec3::new(3.0, 0.0, -1.2));
        v.yaw = std::f32::consts::PI;
        run(&mut v, &mut ed, VisitInput::default(), 3);
        assert_eq!(v.target, Some(ElemRef::Opening(0, 0)), "{:?}", v.target);
        run(&mut v, &mut ed, VisitInput { act: true, ..default() }, 1);
        assert_eq!(ed.doors.get(&ElemRef::Opening(0, 0)).map(|d| d.1), Some(true));
    }
}
