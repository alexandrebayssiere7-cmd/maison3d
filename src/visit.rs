//! Visite : un personnage articulé, vu à la 1re ou à la 3e personne, qui marche, court, saute,
//! ouvre les portes et placards, s'assoit et s'allonge (sur les meubles ou par terre).

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::camera::Projection;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::camera::CamState;
use crate::editor::Editor;
use crate::model::*;
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

/// Angles des articulations (radians) et hauteur du bassin ; indice 0 = gauche, 1 = droite.
#[derive(Clone, Copy, Default, Debug)]
struct Pose {
    hips_y: f32,
    lie: f32,
    spine: f32,
    head: f32,
    sh: [f32; 2],
    el: [f32; 2],
    hip: [f32; 2],
    kn: [f32; 2],
}

impl Pose {
    fn blend(&mut self, t: &Pose, k: f32) {
        let l = |a: &mut f32, b: f32| *a += (b - *a) * k;
        l(&mut self.hips_y, t.hips_y);
        l(&mut self.lie, t.lie);
        l(&mut self.spine, t.spine);
        l(&mut self.head, t.head);
        for i in 0..2 {
            l(&mut self.sh[i], t.sh[i]);
            l(&mut self.el[i], t.el[i]);
            l(&mut self.hip[i], t.hip[i]);
            l(&mut self.kn[i], t.kn[i]);
        }
    }
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

/// Voiture conduite : l'avant est vers +z local, `heading` en radians autour de la verticale.
#[derive(Clone, Copy, Debug)]
struct Car {
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

#[derive(Component)]
pub struct AvatarRoot;

#[derive(Component, Clone, Copy)]
pub enum Joint {
    Hips,
    Spine,
    Head,
    Shoulder(usize),
    Elbow(usize),
    Hip(usize),
    Knee(usize),
}

// ---------------------------------------------------------------- personnage

pub fn spawn_avatar(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let ball = meshes.add(Sphere::new(0.5).mesh().uv(24, 14));
    let mut mat = |r: u8, g: u8, b: u8| mats.add(StandardMaterial { base_color: Color::srgb_u8(r, g, b), perceptual_roughness: 0.8, ..default() });
    let (skin, shirt, pants, shoes, hair, eye) =
        (mat(225, 180, 150), mat(50, 110, 170), mat(45, 50, 70), mat(35, 30, 30), mat(70, 45, 30), mat(20, 20, 25));
    let part = |mesh: &Handle<Mesh>, m: &Handle<StandardMaterial>, c: [f32; 3], s: [f32; 3]| {
        (Mesh3d(mesh.clone()), MeshMaterial3d(m.clone()), Transform::from_translation(c.into()).with_scale(s.into()))
    };
    let joint = |j: Joint, at: [f32; 3]| (j, Transform::from_translation(at.into()), Visibility::Inherited);

    // le personnage regarde vers -Z ; toutes les articulations pivotent autour de X
    commands.spawn((AvatarRoot, Transform::default(), Visibility::Hidden)).with_children(|root| {
        root.spawn(joint(Joint::Hips, [0.0, HIPS_Y, 0.0])).with_children(|hips| {
            hips.spawn(part(&cube, &pants, [0.0, 0.0, 0.0], [0.34, 0.16, 0.2]));
            hips.spawn(joint(Joint::Spine, [0.0, 0.0, 0.0])).with_children(|sp| {
                sp.spawn(part(&cube, &shirt, [0.0, 0.3, 0.0], [0.38, 0.5, 0.21]));
                sp.spawn(part(&cube, &skin, [0.0, 0.6, 0.0], [0.09, 0.1, 0.09]));
                sp.spawn(joint(Joint::Head, [0.0, 0.72, 0.0])).with_children(|h| {
                    h.spawn(part(&ball, &skin, [0.0, 0.0, 0.0], [0.2, 0.25, 0.22]));
                    h.spawn(part(&ball, &hair, [0.0, 0.05, 0.02], [0.215, 0.2, 0.225]));
                    h.spawn(part(&ball, &eye, [-0.045, 0.02, -0.1], [0.03, 0.03, 0.02]));
                    h.spawn(part(&ball, &eye, [0.045, 0.02, -0.1], [0.03, 0.03, 0.02]));
                });
                for (i, side) in [(0, -1.0), (1, 1.0)] {
                    sp.spawn(joint(Joint::Shoulder(i), [side * 0.24, 0.52, 0.0])).with_children(|s| {
                        s.spawn(part(&cube, &shirt, [0.0, -0.14, 0.0], [0.1, 0.3, 0.1]));
                        s.spawn(joint(Joint::Elbow(i), [0.0, -0.29, 0.0])).with_children(|e| {
                            e.spawn(part(&cube, &skin, [0.0, -0.13, 0.0], [0.08, 0.27, 0.08]));
                            e.spawn(part(&ball, &skin, [0.0, -0.3, 0.0], [0.09, 0.1, 0.09]));
                        });
                    });
                }
            });
            for (i, side) in [(0, -1.0), (1, 1.0)] {
                hips.spawn(joint(Joint::Hip(i), [side * 0.1, -0.03, 0.0])).with_children(|l| {
                    l.spawn(part(&cube, &pants, [0.0, -0.22, 0.0], [0.14, 0.45, 0.15]));
                    l.spawn(joint(Joint::Knee(i), [0.0, -0.45, 0.0])).with_children(|k| {
                        k.spawn(part(&cube, &pants, [0.0, -0.21, 0.0], [0.12, 0.43, 0.13]));
                        k.spawn(part(&cube, &shoes, [0.0, -0.44, -0.05], [0.11, 0.07, 0.26]));
                    });
                });
            }
        });
    });
}

fn hips_tf(p: &Pose) -> Transform {
    Transform::from_xyz(0.0, p.hips_y, 0.0).with_rotation(Quat::from_rotation_x(p.lie))
}

fn root_tf(v: &Visitor) -> Transform {
    Transform::from_translation(v.root).with_rotation(Quat::from_rotation_y(v.root_yaw))
}

/// Position des yeux d'après la pose courante (suit le balancement de la marche).
fn eyes(v: &Visitor) -> Vec3 {
    let spine = Transform::from_rotation(Quat::from_rotation_x(v.pose.spine));
    let head = Transform::from_xyz(0.0, 0.72, 0.0).with_rotation(Quat::from_rotation_x(v.pose.head));
    (root_tf(v) * hips_tf(&v.pose) * spine * head).transform_point(Vec3::new(0.0, 0.03, -0.09))
}

/// Pose visée selon la posture et la vitesse ; la marche et la course suivent `phase`.
fn target_pose(v: &Visitor, speed: f32, t: f32) -> Pose {
    let head = (v.pitch * 0.6).clamp(-0.6, 0.6);
    match v.stance {
        Stance::Sit => Pose { hips_y: 0.1, spine: -0.05, head, sh: [0.35; 2], el: [0.9; 2], hip: [1.5; 2], kn: [-1.5; 2], ..default() },
        Stance::SitFloor => Pose { hips_y: 0.12, spine: 0.12, head, sh: [-0.45; 2], el: [0.0; 2], hip: [1.45, 1.3], kn: [-0.2, -0.6], ..default() },
        Stance::Lie => Pose { hips_y: 0.13, lie: FRAC_PI_2, head: (v.pitch * 0.3).clamp(-0.4, 0.4), sh: [0.12; 2], el: [0.2; 2], ..default() },
        Stance::Stand if !v.grounded => Pose {
            hips_y: HIPS_Y,
            spine: -0.1,
            head,
            sh: [-0.4, 0.5],
            el: [0.7; 2],
            hip: [0.8, 0.15],
            kn: [-1.2, -0.4],
            ..default()
        },
        Stance::Stand => {
            let run = ((speed - WALK_SPEED) / (RUN_SPEED - WALK_SPEED)).clamp(0.0, 1.0);
            let moving = (speed / WALK_SPEED).min(1.0);
            let (s, c) = v.phase.sin_cos();
            let leg = moving * (0.42 + 0.45 * run);
            let arm = moving * (0.3 + 0.55 * run);
            // le genou plie quand la jambe revient vers l'avant
            let knee = |c: f32| -moving * (0.08 + (0.45 + 1.1 * run) * c.max(0.0)) - v.dip * 3.0;
            let breathe = (t * 2.0).sin() * 0.012 * (1.0 - moving);
            Pose {
                hips_y: HIPS_Y - v.dip - moving * (0.015 + 0.045 * run) * (0.5 + 0.5 * (2.0 * v.phase).cos()),
                spine: -0.03 - 0.22 * run * moving + breathe,
                head,
                sh: [-arm * s, arm * s],
                el: [0.15 + 1.25 * run * moving; 2],
                hip: [leg * s + v.dip * 1.5, -leg * s + v.dip * 1.5],
                kn: [knee(c), knee(-c)],
                ..default()
            }
        }
    }
}

/// Applique la pose aux articulations et place / montre le personnage.
pub fn animate_avatar(
    v: Res<Visitor>,
    mut root: Single<(&mut Transform, &mut Visibility), (With<AvatarRoot>, Without<MainCamera>)>,
    mut joints: Query<(&Joint, &mut Transform), (Without<AvatarRoot>, Without<MainCamera>)>,
) {
    let (tf, vis) = &mut *root;
    **tf = root_tf(&v);
    // en 1re personne (ou caméra collée au personnage) on ne se voit pas
    **vis = if v.active && v.third && v.boom_cur > 0.6 && v.car.is_none() { Visibility::Visible } else { Visibility::Hidden };
    let p = &v.pose;
    let rx = Quat::from_rotation_x;
    for (j, mut t) in &mut joints {
        match *j {
            Joint::Hips => *t = hips_tf(p),
            Joint::Spine => t.rotation = rx(p.spine),
            Joint::Head => t.rotation = rx(p.head),
            Joint::Shoulder(i) => t.rotation = rx(p.sh[i]),
            Joint::Elbow(i) => t.rotation = rx(p.el[i]),
            Joint::Hip(i) => t.rotation = rx(p.hip[i]),
            Joint::Knee(i) => t.rotation = rx(p.kn[i]),
        }
    }
}

// ---------------------------------------------------------------- meubles où s'asseoir / s'allonger

/// Orientation du personnage pour regarder dans la direction `d` (il regarde vers -Z).
fn yaw_facing(d: Vec3) -> f32 {
    (-d.x).atan2(-d.z)
}

/// Orientation pour être allongé la tête vers `d` (la tête part vers +Z une fois couché).
fn yaw_head_towards(d: Vec3) -> f32 {
    d.x.atan2(d.z)
}

/// Où s'asseoir (`lie` = faux) ou s'allonger sur un meuble : (posture, point d'appui, orientation).
fn furniture_spot(p: &Prop, lie: bool, viewer: Vec3) -> Option<(Stance, Vec3, f32)> {
    let base = Transform::from_xyz(p.pos.x, elev(p.level), p.pos.y).with_rotation(Quat::from_rotation_y(p.rot.to_radians()));
    let local_viewer = base.compute_affine().inverse().transform_point3(viewer);
    let side = if local_viewer.x < 0.0 { -1.0 } else { 1.0 };
    let world = |x: f32, y: f32, z: f32| base.transform_point(Vec3::new(x, y, z));
    let dir = |x: f32, z: f32| base.rotation * Vec3::new(x, 0.0, z);
    let sit = |at: Vec3, facing: Vec3| Some((Stance::Sit, at, yaw_facing(facing)));
    let lie_down = |at: Vec3, head: Vec3| Some((Stance::Lie, at, yaw_head_towards(head)));
    match (p.kind, lie) {
        (PropKind::Chair, _) => sit(world(0.0, 0.48, 0.05), dir(0.0, 1.0)),
        (PropKind::Toilet, _) => sit(world(0.0, 0.42, 0.1), dir(0.0, 1.0)),
        (PropKind::Sofa, false) => {
            // la place la plus proche de celui qui s'assoit
            let x = [-0.55, 0.0, 0.55].into_iter().min_by(|a, b| (a - local_viewer.x).abs().total_cmp(&(b - local_viewer.x).abs()))?;
            sit(world(x, 0.45, 0.1), dir(0.0, 1.0))
        }
        (PropKind::Sofa, true) => lie_down(world(0.05, 0.45, 0.05), dir(-1.0, 0.0)),
        (PropKind::Bed, false) => sit(world(side * 0.7, 0.55, 0.2), dir(side, 0.0)),
        (PropKind::Bed, true) => lie_down(world(side * 0.38, 0.55, 0.15), dir(0.0, -1.0)),
        (PropKind::Bathtub, true) => lie_down(world(0.0, 0.12, 0.15), dir(0.0, -1.0)),
        (PropKind::Bathtub, false) => sit(world(0.0, 0.12, 0.3), dir(0.0, -1.0)),
        _ => None,
    }
}

/// Action principale (touche E) : s'allonger pour un lit / une baignoire, sinon s'asseoir.
fn prefers_lying(k: PropKind) -> bool {
    matches!(k, PropKind::Bed | PropKind::Bathtub)
}

// ---------------------------------------------------------------- logique de visite

#[allow(clippy::too_many_arguments)]
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
    let dt = time.delta_secs().min(0.05);
    let v = &mut *v;
    if !ed.walk {
        if let Some(car) = v.car.take() {
            park(&car, &mut ed);
            v.speed_kmh = None;
        }
        if v.active {
            v.active = false;
            v.target = None;
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            if let Projection::Perspective(p) = &mut *cam.1 {
                p.fov = FRAC_PI_4;
            }
        }
        return;
    }
    if !v.active {
        let start = ed.walk_start.take().unwrap_or(Vec3::new(cs.focus.x, elev(ed.level), cs.focus.z));
        *v = Visitor { third: v.third, boom: v.boom, ..default() };
        v.active = true;
        v.pos = start + Vec3::Y * 0.05;
        (v.yaw, v.body_yaw, v.root_yaw) = (cs.yaw, cs.yaw, cs.yaw);
        v.root = v.pos;
        v.boom_cur = v.boom;
        // souris capturée : on regarde autour de soi sans cliquer
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

    // --- regard et options de vue
    let d = motion.delta * settings.sensitivity;
    v.yaw -= d.x * 0.0025;
    v.pitch = (v.pitch - d.y * 0.0025).clamp(-1.45, 1.45);
    if keys.just_pressed(KeyCode::KeyV) {
        v.third = !v.third;
    }
    // glisser à deux doigts sur le trackpad tourne aussi la vue (même sens qu'en édition) ;
    // la distance de la caméra reste fixe : pas de zoom
    if scroll.unit == MouseScrollUnit::Pixel {
        v.yaw -= scroll.delta.x * 0.004;
        v.pitch = (v.pitch + scroll.delta.y * 0.004).clamp(-1.45, 1.45);
    }
    if v.third {
        // au-delà, la caméra passerait sous le sol et se collerait au personnage
        v.pitch = v.pitch.clamp(-1.2, 0.55);
    }
    let look = Quat::from_euler(EulerRot::YXZ, v.yaw, v.pitch, 0.0);
    let (sy, cy) = v.yaw.sin_cos();
    let (fwd, right) = (Vec3::new(-sy, 0.0, -cy), Vec3::new(cy, 0.0, -sy));
    let key = |k: KeyCode| if keys.pressed(k) { 1.0 } else { 0.0 };
    let mv = Vec2::new(key(KeyCode::KeyD) - key(KeyCode::KeyA), key(KeyCode::KeyW) - key(KeyCode::KeyS));
    let solids: Vec<Transform> = ed.parts.iter().filter(|p| solid(p)).map(|p| p.bounds()).collect();

    if let Some(car) = v.car {
        let (cam_tf, proj) = &mut *cam;
        drive(v, car, &mut ed, &keys, &solids, d, dt, cam_tf, proj);
        return;
    }

    // --- déplacement (debout uniquement)
    let mut speed = 0.0;
    if v.stance == Stance::Stand {
        let input = (fwd * mv.y + right * mv.x).normalize_or_zero();
        // on ne sprinte qu'en avançant ; à reculer on marche
        let running = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) && mv.y > 0.0;
        let want = input * if running { RUN_SPEED } else { WALK_SPEED };
        let mut hv = Vec3::new(v.vel.x, 0.0, v.vel.z);
        let accel = match (v.grounded, want.length() >= hv.length()) {
            (false, _) => AIR_ACCEL,
            (true, true) => GROUND_ACCEL,
            (true, false) => GROUND_DECEL,
        };
        let diff = want - hv;
        hv += diff.clamp_length_max(accel * dt);
        let before = v.pos;
        let mut pos = v.pos + hv * dt;
        for _ in 0..3 {
            for tf in &solids {
                push_out(&mut pos, tf, RADIUS, HEIGHT);
            }
        }
        // la vitesse perdue contre un mur ne s'accumule pas
        let moved = (pos - before) / dt.max(1e-4);
        v.vel.x = moved.x;
        v.vel.z = moved.z;
        if v.grounded && keys.just_pressed(KeyCode::Space) {
            v.vel.y = JUMP_SPEED;
            v.grounded = false;
        }
        v.vel.y -= GRAVITY * dt;
        pos.y += v.vel.y * dt;
        let floor = floor_at(pos, &solids, ed.house.ground(Vec2::new(pos.x, pos.z)));
        let was_grounded = v.grounded;
        v.grounded = pos.y <= floor;
        if v.grounded {
            if !was_grounded && v.vel.y < -2.5 {
                // réception : les jambes amortissent, on perd un peu d'élan
                v.dip = (-v.vel.y * 0.03).min(0.2);
                v.vel.x *= 0.7;
                v.vel.z *= 0.7;
            }
            pos.y = floor;
            v.vel.y = 0.0;
        }
        v.pos = pos;
        speed = Vec2::new(v.vel.x, v.vel.z).length();
        let run = ((speed - WALK_SPEED) / (RUN_SPEED - WALK_SPEED)).clamp(0.0, 1.0);
        if v.grounded {
            // un cycle = deux pas ; la foulée s'allonge en courant
            v.phase = (v.phase + speed * dt * TAU / (1.3 + 1.1 * run)) % TAU;
        }
        // le corps se tourne avec la caméra : en 3e personne on voit le personnage pivoter en douceur
        v.body_yaw = if v.third { lerp_angle(v.body_yaw, v.yaw, 1.0 - (-12.0 * dt).exp()) } else { v.yaw };
    }
    v.dip = (v.dip - dt * 0.5).max(0.0);

    // --- personnage affiché : rejoint en douceur le siège, le lit ou la position debout
    let (root_target, yaw_target) = match v.stance {
        Stance::Stand => (v.pos, v.body_yaw),
        _ => (v.anchor, v.anchor_yaw),
    };
    let k = if v.root.distance(root_target) > 0.25 { 1.0 - (-6.0 * dt).exp() } else { 1.0 };
    v.root = v.root.lerp(root_target, k);
    v.root_yaw = lerp_angle(v.root_yaw, yaw_target, if v.stance == Stance::Stand && !v.third { 1.0 } else { 1.0 - (-8.0 * dt).exp() });
    let t = time.elapsed_secs();
    let target = target_pose(v, speed, t);
    let rate = if v.stance == Stance::Stand { 18.0 } else { 5.0 };
    v.pose.blend(&target, 1.0 - (-rate * dt).exp());

    // --- caméra
    let eye = eyes(v);
    let (cam_tf, proj) = &mut *cam;
    if v.third {
        let pivot = eye + Vec3::Y * 0.1;
        let desired = look * Vec3::new(0.35, 0.15, v.boom);
        let (len, dir) = (desired.length(), desired.normalize());
        let free = camera_clearance(&ed.parts, pivot, dir, len, ed.house.ground(Vec2::new(pivot.x, pivot.z)));
        // on se rapproche tout de suite d'un obstacle, on s'en écarte doucement
        v.boom_cur = if free < v.boom_cur { free } else { v.boom_cur + (free - v.boom_cur) * (1.0 - (-4.0 * dt).exp()) };
        **cam_tf = Transform::from_translation(pivot + dir * v.boom_cur).with_rotation(look);
    } else {
        **cam_tf = Transform::from_translation(eye).with_rotation(look);
    }
    let run = ((speed - WALK_SPEED) / (RUN_SPEED - WALK_SPEED)).clamp(0.0, 1.0);
    v.fov += (0.82 + 0.13 * run - v.fov) * (1.0 - (-4.0 * dt).exp());
    if let Projection::Perspective(p) = &mut **proj {
        p.fov = v.fov;
    }

    // --- ce qu'on vise et les actions possibles
    let origin = cam_tf.translation;
    let ray = look * Vec3::NEG_Z;
    v.target = pick(&ed.parts, origin, ray)
        .filter(|&(_, dist)| (origin + ray * dist).distance(eye) < REACH)
        .map(|h| h.0)
        .filter(|&e| {
            ed.house.openable(e)
                || matches!(e, ElemRef::Prop(i) if ed.house.props.get(i).is_some_and(|p| p.kind == PropKind::Car
                    || furniture_spot(p, false, v.pos).is_some()
                    || furniture_spot(p, true, v.pos).is_some()))
        });
    let vpos = v.pos;
    let prop = match v.target {
        Some(ElemRef::Prop(i)) => ed.house.props.get(i).cloned(),
        _ => None,
    };
    let seat = |lie: bool| prop.as_ref().and_then(|p| furniture_spot(p, lie, vpos));
    let (e, c, x) = (keys.just_pressed(KeyCode::KeyE) || mouse.just_pressed(MouseButton::Left), keys.just_pressed(KeyCode::KeyC), keys.just_pressed(KeyCode::KeyX));

    v.actions.clear();
    if v.stance != Stance::Stand {
        v.actions.push("ZQSD, Espace ou E : se relever");
        let get_up = e || c || x || keys.just_pressed(KeyCode::Space) || mv != Vec2::ZERO;
        if get_up {
            v.stance = Stance::Stand;
            v.pos = v.stand_pos;
            v.vel = Vec3::ZERO;
        }
    } else {
        let openable = v.target.is_some_and(|t| ed.house.openable(t));
        let car_target = prop.as_ref().is_some_and(|p| p.kind == PropKind::Car);
        if car_target {
            v.actions.push("E : conduire");
        }
        if openable {
            let open = v.target.and_then(|t| ed.doors.get(&t)).is_some_and(|d| d.1);
            v.actions.push(if open { "E : fermer" } else { "E : ouvrir" });
        }
        let (sit, lie) = (seat(false), seat(true));
        let primary_lie = prop.as_ref().is_some_and(|p| prefers_lying(p.kind));
        if !openable && (sit.is_some() || lie.is_some()) {
            v.actions.push(if primary_lie { "E : s'allonger" } else { "E : s'asseoir" });
        }
        v.actions.push(if sit.is_some() && primary_lie { "C : s'asseoir au bord" } else if sit.is_some() { "C : s'asseoir ici" } else { "C : s'asseoir par terre" });
        v.actions.push(if lie.is_some() && !primary_lie { "X : s'allonger dessus" } else if lie.is_some() { "X : s'allonger" } else { "X : s'allonger par terre" });

        let chosen = if e && openable {
            ed.toggle_door(v.target.unwrap());
            None
        } else if let (true, Some(ElemRef::Prop(i)), Some(p)) = (e && car_target, v.target, &prop) {
            v.car = Some(Car {
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
            v.boom_cur = 1.0;
            None
        } else if e {
            if primary_lie { lie.or(sit) } else { sit.or(lie) }
        } else if c && v.grounded {
            sit.or(Some((Stance::SitFloor, v.pos, v.body_yaw)))
        } else if x && v.grounded {
            lie.or(Some((Stance::Lie, v.pos, yaw_head_towards(fwd))))
        } else {
            None
        };
        if let Some((stance, at, yaw)) = chosen {
            v.stand_pos = v.pos;
            v.stance = stance;
            v.anchor = at;
            v.anchor_yaw = yaw;
            v.vel = Vec3::ZERO;
        }
    }
    v.actions.push(if v.third { "V : vue 1re personne" } else { "V : vue 3e personne" });
}

/// Un pas de conduite : moteur, freins, direction, collisions, caméra, descente du véhicule.
#[allow(clippy::too_many_arguments)]
fn drive(
    v: &mut Visitor,
    mut car: Car,
    ed: &mut Editor,
    keys: &ButtonInput<KeyCode>,
    solids: &[Transform],
    mouse: Vec2,
    dt: f32,
    cam: &mut Transform,
    proj: &mut Projection,
) {
    let key = |k: KeyCode| if keys.pressed(k) { 1.0 } else { 0.0 };
    let throttle = key(KeyCode::KeyW) - key(KeyCode::KeyS);
    let steer_in = key(KeyCode::KeyA) - key(KeyCode::KeyD);
    let handbrake = keys.pressed(KeyCode::Space);
    car.speed = car_speed(car.speed, throttle, handbrake, dt);

    // la direction braque moins à haute vitesse ; modèle « bicyclette »
    let max_steer = 0.6 / (1.0 + car.speed.abs() * 0.1);
    car.steer += (steer_in * max_steer - car.steer) * (1.0 - (-6.0 * dt).exp());
    car.heading += car.speed / WHEELBASE * car.steer.tan() * dt;
    let f = Vec3::new(car.heading.sin(), 0.0, car.heading.cos());
    let mut pos = car.pos + f * car.speed * dt;
    // carrosserie approchée par quatre cercles alignés sur sa longueur
    let mut hit = false;
    for _ in 0..2 {
        for o in [-1.45, -0.5, 0.5, 1.45] {
            let c0 = pos + f * o;
            let mut c = c0;
            for tf in solids {
                push_out(&mut c, tf, 0.85, 1.4);
            }
            if c != c0 {
                pos += c - c0;
                hit = true;
            }
        }
    }
    if hit && car.speed.abs() > 0.5 {
        // choc : la voiture rebondit et perd l'essentiel de sa vitesse
        car.speed *= -0.3;
    }
    pos.y = floor_at(pos, solids, ed.house.ground(Vec2::new(pos.x, pos.z)));
    car.pos = pos;
    car.spin = (car.spin + car.speed * dt / WHEEL_RADIUS) % TAU;

    // caméra : libre à la souris, se recale derrière la voiture après une seconde
    car.look_idle = if mouse.length() > 0.5 { 0.0 } else { car.look_idle + dt };
    if car.look_idle > 1.0 {
        let k = 1.0 - (-3.0 * dt).exp();
        v.yaw = lerp_angle(v.yaw, yaw_facing(f), k);
        v.pitch += (if v.third { -0.18 } else { -0.05 } - v.pitch) * k;
    }
    let look = Quat::from_euler(EulerRot::YXZ, v.yaw, v.pitch, 0.0);
    let body = car_tf(&car);
    if v.third {
        let pivot = car.pos + Vec3::Y * 1.7;
        let desired = look * Vec3::new(0.0, 0.4, DRIVE_BOOM);
        let (len, dir) = (desired.length(), desired.normalize());
        let free = camera_clearance(&ed.parts, pivot, dir, len, ed.house.ground(Vec2::new(pivot.x, pivot.z)));
        v.boom_cur = if free < v.boom_cur { free } else { v.boom_cur + (free - v.boom_cur) * (1.0 - (-3.0 * dt).exp()) };
        *cam = Transform::from_translation(pivot + dir * v.boom_cur).with_rotation(look);
    } else {
        // place conducteur, à gauche
        *cam = Transform::from_translation(body.transform_point(Vec3::new(0.4, 1.25, -0.25))).with_rotation(look);
    }
    v.fov += (0.8 + 0.25 * (car.speed.abs() / MAX_SPEED).min(1.0) - v.fov) * (1.0 - (-3.0 * dt).exp());
    if let Projection::Perspective(p) = proj {
        p.fov = v.fov;
    }

    v.speed_kmh = Some(car.speed.abs() * 3.6);
    v.target = None;
    v.actions = vec![
        "Z / S : accélérer, freiner, reculer",
        "Q / D : tourner  •  Espace : frein à main",
        "E : descendre",
        if v.third { "V : vue intérieure" } else { "V : vue extérieure" },
    ];
    if keys.just_pressed(KeyCode::KeyE) {
        // on descend côté conducteur
        let out = body.transform_point(Vec3::new(1.5, 0.0, -0.2));
        v.pos = Vec3::new(out.x, car.pos.y + 0.05, out.z);
        v.root = v.pos;
        v.vel = Vec3::ZERO;
        v.body_yaw = v.yaw;
        v.root_yaw = v.yaw;
        v.speed_kmh = None;
        v.car = None;
        park(&car, ed);
        return;
    }
    v.car = Some(car);
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
fn park(car: &Car, ed: &mut Editor) {
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
    match (v.car, body.single_mut()) {
        (Some(car), Ok((_, mut tf))) => {
            *tf = car_tf(&car);
            for (w, mut t) in &mut wheels {
                let steer = if w.front { car.steer } else { 0.0 };
                t.rotation = Quat::from_rotation_y(steer) * Quat::from_rotation_x(car.spin) * w.base.rotation;
            }
        }
        (Some(car), Err(_)) => {
            let Some(p) = ed.house.props.get(car.prop) else { return };
            let model = Prop { pos: Vec2::ZERO, level: 0, rot: 0.0, ..p.clone() };
            let parts = prop_parts(&model, ElemRef::Prop(car.prop), 0.0, 0.0);
            commands.spawn((CarBody, car_tf(&car), Visibility::Visible)).with_children(|root| {
                for part in &parts {
                    let bundle = crate::render::part_bundle(&mut lib, &mut meshes, &mut materials, part);
                    let base = bundle.2;
                    let mut e = root.spawn(bundle);
                    if part.shape == Shape::Wheel {
                        e.insert(CarWheel { base, front: part.tf.translation.z > 0.0 });
                    }
                }
            });
        }
        (None, Ok((e, _))) => commands.entity(e).despawn(),
        (None, Err(_)) => {}
    }
}

fn lerp_angle(a: f32, b: f32, k: f32) -> f32 {
    let d = (b - a + std::f32::consts::PI).rem_euclid(TAU) - std::f32::consts::PI;
    a + d * k
}

// ---------------------------------------------------------------- collisions

/// Ce qui arrête le visiteur (le volume englobant d'un toit en pente, lui, est surtout vide ;
/// on traverse l'eau, et la route inclinée est portée par le terrain terrassé dessous).
fn solid(p: &Part) -> bool {
    !matches!(p.shape, Shape::Light | Shape::Roof { .. })
        && !matches!(p.owner, ElemRef::Road(_))
        && !(p.shape == Shape::Glass && matches!(p.owner, ElemRef::Pool(_)))
}

/// Bas et haut d'une boîte ; nos pièces ne tournent qu'autour de la verticale.
fn y_range(tf: &Transform) -> (f32, f32) {
    (tf.translation.y - tf.scale.y / 2.0, tf.translation.y + tf.scale.y / 2.0)
}

/// Repousse un cylindre (rayon, hauteur) hors d'une boîte qu'il traverse.
fn push_out(pos: &mut Vec3, tf: &Transform, radius: f32, height: f32) {
    let (y0, y1) = y_range(tf);
    if y1 <= pos.y + STEP || y0 >= pos.y + height {
        return;
    }
    let l = tf.rotation.inverse() * (*pos - tf.translation);
    let h = tf.scale / 2.0;
    let (dx, dz) = (l.x - l.x.clamp(-h.x, h.x), l.z - l.z.clamp(-h.z, h.z));
    let d2 = dx * dx + dz * dz;
    let push = if d2 > 1e-8 {
        if d2 >= radius * radius {
            return;
        }
        let d = d2.sqrt();
        Vec2::new(dx, dz) / d * (radius - d)
    } else {
        // centre à l'intérieur : sortie par le côté le plus proche
        let (px, pz) = (h.x - l.x.abs() + radius, h.z - l.z.abs() + radius);
        if px < pz { Vec2::new(px * l.x.signum(), 0.0) } else { Vec2::new(0.0, pz * l.z.signum()) }
    };
    *pos += tf.rotation * Vec3::new(push.x, 0.0, push.y);
}

/// Hauteur du sol sous les pieds : dessus de la plus haute boîte franchissable, ou le terrain `ground`.
fn floor_at(pos: Vec3, solids: &[Transform], ground: f32) -> f32 {
    let mut best = if pos.y >= ground - 0.05 { ground } else { -50.0 };
    for tf in solids {
        let top = y_range(tf).1;
        if top > pos.y + STEP || top <= best {
            continue;
        }
        let l = tf.rotation.inverse() * (pos - tf.translation);
        let h = tf.scale / 2.0;
        if l.x.abs() <= h.x && l.z.abs() <= h.z {
            best = top;
        }
    }
    best
}

/// Distance libre derrière le personnage pour la caméra 3e personne (murs, sols, toits, terrain).
fn camera_clearance(parts: &[Part], from: Vec3, dir: Vec3, max: f32, ground: f32) -> f32 {
    let mut best = max;
    for p in parts.iter().filter(|p| p.shape != Shape::Light) {
        let hit = match p.shape {
            Shape::Roof { kind, size } => roof_tris(kind, size)
                .iter()
                .filter_map(|t| ray_triangle(from, dir, t.map(|v| p.tf.transform_point(v))))
                .reduce(f32::min),
            // une boîte qui contient déjà la tête (dossier, oreiller…) ne compte pas
            _ => ray_obb(from, dir, &p.bounds()).filter(|&t| t > 0.01),
        };
        if let Some(t) = hit {
            best = best.min(t);
        }
    }
    if dir.y < 0.0 && from.y > ground {
        best = best.min((from.y - ground) / -dir.y);
    }
    (best - 0.25).max(0.15)
}

/// Möller–Trumbore.
fn ray_triangle(o: Vec3, d: Vec3, [a, b, c]: [Vec3; 3]) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-7 {
        return None;
    }
    let s = o - a;
    let u = s.dot(p) / det;
    let q = s.cross(e1);
    let w = d.dot(q) / det;
    let t = e2.dot(q) / det;
    (u >= 0.0 && w >= 0.0 && u + w <= 1.0 && t > 0.0).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solids(open: f32) -> Vec<Transform> {
        let h = demo_house();
        h.parts(Some(0), &|e| if e == ElemRef::Opening(0, 0) { open } else { 0.0 })
            .iter()
            .filter(|p| solid(p))
            .map(|p| p.bounds())
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
        h.props.push(Prop { kind: PropKind::Stairs, pos: Vec2::new(20.0, 0.0), level: 0, rot: 0.0, color: [1.0; 3], texture: 0, height: LEVEL_H });
        let stairs: Vec<Transform> = h.parts_closed(None).iter().filter(|p| solid(p)).map(|p| p.bounds()).collect();
        // 1,6 s à 3 m/s : on s'arrête sur les dernières marches
        let top = walk(Vec3::new(20.0, 0.0, -3.0), 1.6, &stairs);
        assert!(top.y > 2.0, "n'a pas monté l'escalier : {top}");
    }

    #[test]
    fn third_person_camera_stops_before_walls_and_roof() {
        let h = demo_house();
        let parts = h.parts_closed(None);
        // au centre du salon, caméra reculée vers le mur sud (z = 0) : bloquée avant 3,9 m
        let head = Vec3::new(3.0, 1.7, 3.9);
        let free = camera_clearance(&parts, head, -Vec3::Z, 10.0, 0.0);
        assert!(free < 3.8 && free > 3.0, "caméra traverse le mur : {free}");
        // vers le haut : le toit (base à 3 m) arrête la caméra
        let up = camera_clearance(&parts, head, Vec3::Y, 10.0, 0.0);
        assert!(up < 4.0, "caméra traverse le toit : {up}");
        // dehors, rien derrière : distance complète
        assert_eq!(camera_clearance(&parts, Vec3::new(30.0, 1.7, 30.0), Vec3::X, 4.0, 0.0), 3.75);
    }

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
        let wheels = prop_parts(&ed.house.props[i], ElemRef::Prop(i), 0.0, 0.0).iter().filter(|p| p.shape == Shape::Wheel).count();
        assert_eq!(wheels, 4);
        ed.driving = Some(i);
        let car = Car { prop: i, pos: Vec3::new(20.0, 0.0, 5.0), heading: FRAC_PI_2, speed: 0.0, steer: 0.0, spin: 0.0, look_idle: 0.0 };
        park(&car, &mut ed);
        assert_eq!(ed.house.props[i].pos, Vec2::new(20.0, 5.0));
        assert!((ed.house.props[i].rot - 90.0).abs() < 1e-3);
        assert!(ed.driving.is_none() && ed.dirty);
    }

    #[test]
    fn furniture_spots_face_the_right_way() {
        let chair = Prop { kind: PropKind::Chair, pos: Vec2::ZERO, level: 0, rot: 0.0, color: [1.0; 3], texture: 0, height: LEVEL_H };
        let (stance, at, yaw) = furniture_spot(&chair, false, Vec3::new(0.0, 0.0, 2.0)).unwrap();
        assert_eq!(stance, Stance::Sit);
        assert!((at.y - 0.48).abs() < 1e-4);
        // assis, on regarde vers l'avant de la chaise (+z)
        let facing = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
        assert!(facing.z > 0.99, "{facing}");
        let bed = Prop { kind: PropKind::Bed, ..chair.clone() };
        let (stance, _, yaw) = furniture_spot(&bed, true, Vec3::new(2.0, 0.0, 0.0)).unwrap();
        assert_eq!(stance, Stance::Lie);
        // allongé, la tête va vers la tête de lit (-z)
        assert!((Quat::from_rotation_y(yaw) * Vec3::Z).z < -0.99);
        assert!(furniture_spot(&Prop { kind: PropKind::Table, ..chair }, false, Vec3::ZERO).is_none());
    }
}
