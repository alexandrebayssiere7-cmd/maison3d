//! Caméra orbitale (édition) et caméra « visite » à hauteur d'homme.

use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::editor::Editor;
use crate::model::elev;
use crate::render::MainCamera;
use crate::visit::Visitor;

#[derive(Resource)]
pub struct CamState {
    pub focus: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    was_walking: bool,
    /// Distance parcourue par la souris pendant le clic droit (pour distinguer clic et glisser).
    pub rmb_travel: f32,
    level_y: f32,
}

/// Inclinaison : un peu vers le haut jusqu'à presque à la verticale vers le bas.
const MIN_PITCH: f32 = -1.2;
const MAX_PITCH: f32 = 1.55;

impl Default for CamState {
    fn default() -> Self {
        Self {
            focus: Vec3::new(5.0, 0.0, 4.0),
            yaw: -0.6,
            pitch: 0.6,
            dist: 28.0,
            was_walking: false,
            rmb_travel: 0.0,
            level_y: 0.0,
        }
    }
}

impl CamState {
    /// Direction du point visé vers l'œil.
    fn offset(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(sy * cp, sp, cy * cp)
    }

    fn eye(&self) -> Vec3 {
        self.focus + self.offset() * self.dist
    }

    /// Avant horizontal et droite de la caméra orbitale.
    pub fn axes(&self) -> (Vec3, Vec3) {
        let (s, c) = self.yaw.sin_cos();
        (Vec3::new(-s, 0.0, -c), Vec3::new(c, 0.0, -s))
    }
}

pub fn camera_control(
    mut cs: ResMut<CamState>,
    mut ed: ResMut<Editor>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut pinch: EventReader<PinchGesture>,
    time: Res<Time>,
    mut egui: EguiContexts,
    mut cam: Single<&mut Transform, With<MainCamera>>,
    visitor: Res<Visitor>,
    settings: Res<crate::menu::Settings>,
) {
    let ctx = egui.ctx_mut();
    let over_ui = ctx.is_pointer_over_area() && !mouse.pressed(MouseButton::Right) && !mouse.pressed(MouseButton::Middle);
    let typing = ctx.wants_keyboard_input();
    let d = motion.delta;
    let dt = time.delta_secs();

    if mouse.just_pressed(MouseButton::Right) {
        cs.rmb_travel = 0.0;
    }
    if mouse.pressed(MouseButton::Right) {
        cs.rmb_travel += d.length();
    }
    // touches physiques : ZQSD sur AZERTY = WASD sur QWERTY
    let modifier = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::SuperLeft, KeyCode::SuperRight]);
    let key = |k: KeyCode| if !typing && !modifier && keys.pressed(k) { 1.0 } else { 0.0 };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let mv = Vec2::new(key(KeyCode::KeyD) - key(KeyCode::KeyA), key(KeyCode::KeyW) - key(KeyCode::KeyS));

    ed.animate_doors(dt);
    // la visite pilote la caméra elle-même ; au retour on reprend la vue au-dessus du visiteur
    if ed.walk {
        cs.was_walking = true;
        return;
    }
    if cs.was_walking {
        cs.focus = Vec3::new(visitor.pos.x, cs.focus.y, visitor.pos.z);
        cs.yaw = visitor.yaw;
        cs.was_walking = false;
    }

    let (fwd, right) = cs.axes();
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let orbit = mouse.pressed(MouseButton::Right) || (alt && mouse.pressed(MouseButton::Left));
    let pan = mouse.pressed(MouseButton::Middle) || (mouse.pressed(MouseButton::Right) && shift);
    if pan && !over_ui {
        let k = cs.dist * 0.0015;
        cs.focus += (-right * d.x + fwd * d.y) * k;
    }
    // on pivote sur place (comme tourner la tête) : l'œil reste fixe, c'est le point visé qui bouge
    let eye_before = cs.eye();
    let angles_before = (cs.yaw, cs.pitch);
    if orbit && !pan && !over_ui {
        cs.yaw -= d.x * 0.005 * settings.sensitivity;
        cs.pitch = (cs.pitch + d.y * 0.005 * settings.sensitivity).clamp(MIN_PITCH, MAX_PITCH);
    }
    if !ctx.is_pointer_over_area() {
        match scroll.unit {
            MouseScrollUnit::Line => cs.dist = (cs.dist * (1.0 - scroll.delta.y * 0.1)).clamp(2.0, 300.0),
            // trackpad : glisser à deux doigts fait tourner la vue (Maj : la déplace)
            MouseScrollUnit::Pixel if shift => {
                let k = cs.dist * 0.0015;
                cs.focus += (right * scroll.delta.x - fwd * scroll.delta.y) * k;
            }
            MouseScrollUnit::Pixel => {
                cs.yaw -= scroll.delta.x * 0.004;
                cs.pitch = (cs.pitch + scroll.delta.y * 0.004).clamp(MIN_PITCH, MAX_PITCH);
            }
        }
        for PinchGesture(z) in pinch.read() {
            cs.dist = (cs.dist * (1.0 - z * 1.5)).clamp(2.0, 300.0);
        }
    }
    // A (AZERTY) / Q (QWERTY) tourne à gauche, E à droite
    let rot_keys = key(KeyCode::KeyQ) - key(KeyCode::KeyE);
    cs.yaw += rot_keys * 1.5 * dt;
    // flèches : pivoter / incliner la vue (quand rien n'est sélectionné, sinon elles déplacent l'objet)
    if ed.selected.is_none() {
        cs.yaw += (key(KeyCode::ArrowLeft) - key(KeyCode::ArrowRight)) * 1.5 * dt;
        cs.pitch = (cs.pitch + (key(KeyCode::ArrowUp) - key(KeyCode::ArrowDown)) * 1.0 * dt).clamp(MIN_PITCH, MAX_PITCH);
    }
    if (cs.yaw, cs.pitch) != angles_before {
        cs.focus = eye_before - cs.offset() * cs.dist;
    }
    // déplacement « en vol » : Z avance exactement là où l'on regarde (pente comprise)
    let look = -cs.offset();
    let side = look.cross(Vec3::Y).normalize_or_zero();
    let up = key(KeyCode::Space) - key(KeyCode::KeyC);
    let speed = (cs.dist * 0.5).clamp(3.0, 40.0) * if shift { 2.5 } else { 1.0 };
    cs.focus += (look * mv.y + side * mv.x + Vec3::Y * up) * speed * dt;
    // ne pas passer sous le sol de l'étage courant
    let floor = cs.level_y + 0.3;
    let eye_y = cs.eye().y;
    if eye_y < floor {
        cs.focus.y += floor - eye_y;
    }
    // la caméra monte / descend en douceur quand on change d'étage
    let step = (elev(ed.level) - cs.level_y) * (dt * 8.0).min(1.0);
    cs.level_y += step;
    cs.focus.y += step;

    **cam = Transform::from_translation(cs.eye()).looking_at(cs.focus, Vec3::Y);
}

