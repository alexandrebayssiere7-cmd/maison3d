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
        yaw_axes(self.yaw)
    }
}

/// Avant et droite à l'horizontale pour un regard tourné de `yaw` (0 : vers -z).
pub fn yaw_axes(yaw: f32) -> (Vec3, Vec3) {
    let (s, c) = yaw.sin_cos();
    (Vec3::new(-s, 0.0, -c), Vec3::new(c, 0.0, -s))
}

/// Entrées d'une image pour la caméra d'édition.
#[derive(Clone, Default, Debug)]
struct CamInput {
    motion: Vec2,
    rmb_pressed: bool,
    rmb: bool,
    lmb: bool,
    mmb: bool,
    alt: bool,
    shift: bool,
    /// ZQSD (x vers la droite, y vers l'avant), Espace / C, A / E, flèches (x vers la gauche, y vers le haut).
    mv: Vec2,
    up: f32,
    turn: f32,
    arrows: Vec2,
    /// Molette ou trackpad, et pincements : absents quand le pointeur est sur un panneau.
    scroll: Option<(MouseScrollUnit, Vec2)>,
    pinch: Vec<f32>,
    /// Pointeur sur un panneau sans bouton de caméra tenu.
    over_ui: bool,
    sensitivity: f32,
    dt: f32,
}

/// Lit clavier et souris ; `typing` : un champ de texte a le clavier, `over_area` : pointeur sur un panneau.
fn read_input(
    mouse: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    motion: Vec2,
    scroll: &AccumulatedMouseScroll,
    pinch: Vec<f32>,
    typing: bool,
    over_area: bool,
) -> CamInput {
    // touches physiques : ZQSD sur AZERTY = WASD sur QWERTY
    let modifier =
        keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::SuperLeft, KeyCode::SuperRight]);
    let key = |k: KeyCode| if !typing && !modifier && keys.pressed(k) { 1.0 } else { 0.0 };
    let (rmb, mmb) = (mouse.pressed(MouseButton::Right), mouse.pressed(MouseButton::Middle));
    CamInput {
        motion,
        rmb_pressed: mouse.just_pressed(MouseButton::Right),
        rmb,
        lmb: mouse.pressed(MouseButton::Left),
        mmb,
        alt: keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        mv: Vec2::new(key(KeyCode::KeyD) - key(KeyCode::KeyA), key(KeyCode::KeyW) - key(KeyCode::KeyS)),
        up: key(KeyCode::Space) - key(KeyCode::KeyC),
        // A (AZERTY) / Q (QWERTY) tourne à gauche, E à droite
        turn: key(KeyCode::KeyQ) - key(KeyCode::KeyE),
        arrows: Vec2::new(
            key(KeyCode::ArrowLeft) - key(KeyCode::ArrowRight),
            key(KeyCode::ArrowUp) - key(KeyCode::ArrowDown),
        ),
        scroll: (!over_area).then_some((scroll.unit, scroll.delta)),
        pinch,
        over_ui: over_area && !rmb && !mmb,
        sensitivity: 1.0,
        dt: 0.0,
    }
}

impl CamState {
    fn track_rmb(&mut self, inp: &CamInput) {
        if inp.rmb_pressed {
            self.rmb_travel = 0.0;
        }
        if inp.rmb {
            self.rmb_travel += inp.motion.length();
        }
    }

    /// Au retour de visite, on reprend la vue au-dessus du visiteur.
    fn resume_after_walk(&mut self, pos: Vec3, yaw: f32) {
        if self.was_walking {
            self.focus = Vec3::new(pos.x, self.focus.y, pos.z);
            self.yaw = yaw;
            self.was_walking = false;
        }
    }

    /// Une image de caméra d'édition ; `arrows` : les flèches pivotent la vue (rien n'est sélectionné).
    fn steer(&mut self, inp: &CamInput, arrows: bool, level_y: f32) {
        let (fwd, right) = self.axes();
        let pan = inp.mmb || (inp.rmb && inp.shift);
        if pan && !inp.over_ui {
            let k = self.dist * 0.0015;
            self.focus += (-right * inp.motion.x + fwd * inp.motion.y) * k;
        }
        // on pivote sur place (comme tourner la tête) : l'œil reste fixe, c'est le point visé qui bouge
        let eye_before = self.eye();
        let angles_before = (self.yaw, self.pitch);
        let orbit = inp.rmb || (inp.alt && inp.lmb);
        if orbit && !pan && !inp.over_ui {
            self.yaw -= inp.motion.x * 0.005 * inp.sensitivity;
            self.pitch = (self.pitch + inp.motion.y * 0.005 * inp.sensitivity).clamp(MIN_PITCH, MAX_PITCH);
        }
        if let Some((unit, delta)) = inp.scroll {
            self.wheel(unit, delta, inp.shift);
        }
        for z in &inp.pinch {
            self.dist = (self.dist * (1.0 - z * 1.5)).clamp(2.0, 300.0);
        }
        self.yaw += inp.turn * 1.5 * inp.dt;
        // flèches : pivoter / incliner la vue (quand rien n'est sélectionné, sinon elles déplacent l'objet)
        if arrows {
            self.yaw += inp.arrows.x * 1.5 * inp.dt;
            self.pitch = (self.pitch + inp.arrows.y * inp.dt).clamp(MIN_PITCH, MAX_PITCH);
        }
        if (self.yaw, self.pitch) != angles_before {
            self.focus = eye_before - self.offset() * self.dist;
        }
        self.fly(inp);
        self.follow_level(level_y, inp.dt);
    }

    fn wheel(&mut self, unit: MouseScrollUnit, delta: Vec2, shift: bool) {
        let (fwd, right) = self.axes();
        match unit {
            MouseScrollUnit::Line => self.dist = (self.dist * (1.0 - delta.y * 0.1)).clamp(2.0, 300.0),
            // trackpad : glisser à deux doigts fait tourner la vue (Maj : la déplace)
            MouseScrollUnit::Pixel if shift => {
                let k = self.dist * 0.0015;
                self.focus += (right * delta.x - fwd * delta.y) * k;
            }
            MouseScrollUnit::Pixel => {
                self.yaw -= delta.x * 0.004;
                self.pitch = (self.pitch + delta.y * 0.004).clamp(MIN_PITCH, MAX_PITCH);
            }
        }
    }

    /// Déplacement « en vol » : Z avance exactement là où l'on regarde (pente comprise).
    fn fly(&mut self, inp: &CamInput) {
        let look = -self.offset();
        let side = look.cross(Vec3::Y).normalize_or_zero();
        let speed = (self.dist * 0.5).clamp(3.0, 40.0) * if inp.shift { 2.5 } else { 1.0 };
        self.focus += (look * inp.mv.y + side * inp.mv.x + Vec3::Y * inp.up) * speed * inp.dt;
    }

    /// Reste au-dessus du sol de l'étage, et suit en douceur les changements d'étage.
    fn follow_level(&mut self, level_y: f32, dt: f32) {
        let floor = self.level_y + 0.3;
        let eye_y = self.eye().y;
        if eye_y < floor {
            self.focus.y += floor - eye_y;
        }
        let step = (level_y - self.level_y) * (dt * 8.0).min(1.0);
        self.level_y += step;
        self.focus.y += step;
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
    let over_area = ctx.is_pointer_over_area();
    // sur un panneau, les pincements restent en attente
    let pinch = if over_area { vec![] } else { pinch.read().map(|p| p.0).collect() };
    let inp = CamInput {
        sensitivity: settings.sensitivity,
        dt: time.delta_secs(),
        ..read_input(&mouse, &keys, motion.delta, &scroll, pinch, ctx.wants_keyboard_input(), over_area)
    };
    cs.track_rmb(&inp);
    ed.animate_doors(inp.dt);
    // la visite pilote la caméra elle-même
    if ed.walk {
        cs.was_walking = true;
        return;
    }
    cs.resume_after_walk(visitor.pos, visitor.yaw);
    cs.steer(&inp, ed.selected.is_none(), elev(ed.level));
    **cam = Transform::from_translation(cs.eye()).looking_at(cs.focus, Vec3::Y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_face_away_from_the_eye_and_right_is_a_quarter_turn() {
        let cam = |yaw| CamState { yaw, ..default() };
        let (fwd, right) = cam(0.0).axes();
        assert!(fwd.abs_diff_eq(Vec3::NEG_Z, 1e-6) && right.abs_diff_eq(Vec3::X, 1e-6));
        let (fwd, right) = cam(std::f32::consts::FRAC_PI_2).axes();
        assert!(fwd.abs_diff_eq(Vec3::NEG_X, 1e-6) && right.abs_diff_eq(Vec3::NEG_Z, 1e-6));
    }

    fn frame(inp: CamInput) -> CamInput {
        CamInput { sensitivity: 1.0, dt: 0.1, ..inp }
    }

    #[test]
    fn reads_keys_unless_typing_or_with_a_modifier() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyQ);
        keys.press(KeyCode::ArrowUp);
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Right);
        let scroll = AccumulatedMouseScroll { unit: MouseScrollUnit::Line, delta: Vec2::Y };
        let inp = read_input(&mouse, &keys, Vec2::X, &scroll, vec![0.1], false, true);
        assert_eq!((inp.mv, inp.turn, inp.arrows), (Vec2::Y, 1.0, Vec2::Y));
        assert!(inp.rmb && inp.rmb_pressed && !inp.over_ui, "clic droit tenu : la caméra garde la main");
        assert!(inp.scroll.is_none(), "molette sur un panneau");
        assert_eq!(read_input(&mouse, &keys, Vec2::X, &scroll, vec![], true, false).mv, Vec2::ZERO);
        keys.press(KeyCode::ControlLeft);
        let inp = read_input(&mouse, &keys, Vec2::X, &scroll, vec![], false, false);
        assert_eq!((inp.mv, inp.scroll), (Vec2::ZERO, Some((MouseScrollUnit::Line, Vec2::Y))));
    }

    #[test]
    fn orbit_turns_around_the_eye_and_pan_moves_the_focus() {
        let mut cs = CamState::default();
        let eye = cs.eye();
        cs.steer(&frame(CamInput { rmb: true, motion: Vec2::new(40.0, 10.0), ..default() }), true, 0.0);
        assert!((cs.eye() - eye).length() < 1e-3, "l'œil reste fixe");
        assert!((cs.yaw - (-0.8)).abs() < 1e-5);
        let focus = cs.focus;
        cs.steer(&frame(CamInput { mmb: true, motion: Vec2::new(10.0, 0.0), ..default() }), true, 0.0);
        assert!((cs.focus - focus).length() > 0.1);
        let yaw = cs.yaw;
        cs.steer(&frame(CamInput { rmb: true, over_ui: true, motion: Vec2::X, ..default() }), true, 0.0);
        assert_eq!(cs.yaw, yaw, "sur un panneau la souris ne tourne pas la vue");
        cs.track_rmb(&CamInput { rmb_pressed: true, rmb: true, motion: Vec2::new(3.0, 4.0), ..default() });
        assert_eq!(cs.rmb_travel, 5.0);
    }

    #[test]
    fn wheel_trackpad_pinch_and_keys() {
        let mut cs = CamState::default();
        cs.steer(&frame(CamInput { scroll: Some((MouseScrollUnit::Line, Vec2::Y)), ..default() }), true, 0.0);
        assert!((cs.dist - 25.2).abs() < 1e-3);
        cs.steer(&frame(CamInput { pinch: vec![0.1], ..default() }), true, 0.0);
        assert!(cs.dist < 25.0);
        let yaw = cs.yaw;
        cs.steer(&frame(CamInput { scroll: Some((MouseScrollUnit::Pixel, Vec2::X)), ..default() }), true, 0.0);
        assert!(cs.yaw < yaw);
        let focus = cs.focus;
        let shifted = CamInput { shift: true, scroll: Some((MouseScrollUnit::Pixel, Vec2::X)), ..default() };
        cs.steer(&frame(shifted), true, 0.0);
        assert!((cs.focus - focus).length() > 0.01);
        let (yaw, pitch) = (cs.yaw, cs.pitch);
        cs.steer(&frame(CamInput { turn: 1.0, arrows: Vec2::new(1.0, 1.0), ..default() }), true, 0.0);
        assert!((cs.yaw - yaw - 0.3).abs() < 1e-5 && (cs.pitch - pitch - 0.1).abs() < 1e-5);
        let yaw = cs.yaw;
        cs.steer(&frame(CamInput { arrows: Vec2::X, ..default() }), false, 0.0);
        assert_eq!(cs.yaw, yaw, "avec une sélection, les flèches déplacent l'objet");
        let focus = cs.focus;
        cs.steer(&frame(CamInput { mv: Vec2::Y, up: 1.0, ..default() }), true, 0.0);
        assert!(cs.focus.distance(focus) > 0.5);
    }

    #[test]
    fn stays_above_the_floor_and_follows_levels() {
        let mut cs = CamState { pitch: -1.0, dist: 10.0, ..default() };
        cs.steer(&frame(CamInput::default()), true, 0.0);
        assert!(cs.eye().y >= 0.3 - 1e-4);
        for _ in 0..30 {
            cs.steer(&frame(CamInput::default()), true, 3.0);
        }
        assert!((cs.level_y - 3.0).abs() < 1e-3);
        cs.was_walking = true;
        cs.resume_after_walk(Vec3::new(7.0, 0.0, 8.0), 1.0);
        assert_eq!((cs.focus.x, cs.focus.z, cs.yaw, cs.was_walking), (7.0, 8.0, 1.0, false));
        cs.resume_after_walk(Vec3::ZERO, 0.0);
        assert_eq!(cs.yaw, 1.0, "une seule fois");
    }
}
