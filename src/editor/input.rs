//! Interactions souris dans le monde 3D : outils de tracé, aperçus, survol, grille.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, egui};

use super::tools::{big_enough, fmt_m};
use super::{Editor, Tool};
use crate::camera::CamState;
use crate::command::Command;
use crate::model::{ElemRef, House, OpeningKind, Part, PropKind, Shape, elev, pick_part, prop_parts};
use crate::render::MainCamera;

/// Entrées souris d'une image, lues par le système.
#[derive(Clone, Copy, Default, Debug)]
pub struct PointerInput {
    /// Rayon sous le curseur (aucun hors de la fenêtre).
    pub ray: Option<Ray3d>,
    /// Maj : placement libre, sans grille.
    pub free: bool,
    /// Alt + clic gauche est réservé à la caméra.
    pub alt: bool,
    pub over_ui: bool,
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
    /// Clic droit sans glisser : annule l'action en cours.
    pub cancel: bool,
    pub dt: f32,
}

/// Rôle d'un aperçu filaire, qui donne sa couleur.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hint {
    Selected,
    Hovered,
    Erase,
    Preview,
    TooSmall,
    Fits,
    Blocked,
}

impl Hint {
    pub fn color(self) -> Color {
        match self {
            Hint::Selected => Color::srgb(1.0, 0.85, 0.1),
            Hint::Hovered => Color::srgba(1.0, 1.0, 1.0, 0.7),
            Hint::Erase => Color::srgb(1.0, 0.25, 0.25),
            Hint::Preview => Color::srgb(0.3, 0.9, 1.0),
            Hint::TooSmall => Color::srgb(1.0, 0.6, 0.2),
            Hint::Fits => Color::srgb(0.3, 1.0, 0.4),
            Hint::Blocked => Color::srgb(1.0, 0.3, 0.3),
        }
    }
}

/// Ce qu'il faut dessiner cette image par-dessus la scène.
#[derive(Default, Debug)]
pub struct Overlay {
    pub boxes: Vec<(Vec<Part>, Hint)>,
    /// Point visé par un outil de tracé.
    pub cursor: Option<Vec3>,
    /// Contour du pinceau de relief.
    pub brush: Vec<Vec3>,
    /// Où le bonhomme serait posé.
    pub figure: Option<Vec3>,
}

/// Le point visé dans le monde pour cette image.
struct Aim {
    origin: Vec3,
    dir: Vec3,
    /// Point du sol visé (relief au rez-de-chaussée, plancher ailleurs) et sa version aimantée.
    ground3: Option<Vec3>,
    ground: Option<Vec2>,
    p: Option<Vec2>,
    hit: Option<(ElemRef, f32)>,
    /// Partie colorable de l'élément visé.
    part: usize,
    click: bool,
    released: bool,
}

impl Tool {
    /// Outils qui posent quelque chose au point de la grille visé (petit cercle sous le curseur).
    fn places_at_cursor(self) -> bool {
        !matches!(
            self,
            Tool::Select
                | Tool::Paint
                | Tool::Eyedropper
                | Tool::Erase
                | Tool::Opening(_)
                | Tool::Visitor
                | Tool::Terrain
        )
    }
}

impl Editor {
    pub fn grid_visible(&self) -> bool {
        self.show_grid && !self.walk
    }

    fn owned_parts(&self, e: ElemRef) -> Vec<Part> {
        self.parts.iter().filter(|p| p.owner == e).cloned().collect()
    }

    /// Une image d'interaction souris : met à jour l'outil en cours et dit quoi dessiner.
    pub fn pointer(&mut self, inp: &PointerInput) -> Overlay {
        let mut o = Overlay::default();
        if let Some(sel) = self.selected {
            o.boxes.push((self.owned_parts(sel), Hint::Selected));
        }
        if inp.cancel {
            self.cancel();
        }
        self.hovered = None;
        self.measure = None;
        self.cursor = None;
        if self.walk {
            return o;
        }
        let Some(ray) = inp.ray else { return o };
        let aim = self.aim(ray, inp);
        if let Some(p) = aim.p.filter(|_| self.tool.places_at_cursor() && !inp.over_ui) {
            let y = if self.level == 0 { self.house.terrain.height(p) } else { elev(self.level) };
            o.cursor = Some(Vec3::new(p.x, y + 0.03, p.y));
        }
        self.tool_frame(&aim, inp, &mut o);
        if let Some(h) = self.hovered.filter(|h| Some(*h) != self.selected) {
            let hint = if self.tool == Tool::Erase { Hint::Erase } else { Hint::Hovered };
            o.boxes.push((self.owned_parts(h), hint));
        }
        o
    }

    fn aim(&mut self, ray: Ray3d, inp: &PointerInput) -> Aim {
        let (origin, dir) = (ray.origin, *ray.direction);
        // au rez-de-chaussée on vise le relief, ailleurs le plancher de l'étage
        let ground3 = if self.level == 0 {
            self.house.terrain.ray(origin, dir)
        } else {
            let floor = Vec3::new(0.0, elev(self.level), 0.0);
            ray.intersect_plane(floor, InfinitePlane3d::new(Vec3::Y)).map(|t| ray.get_point(t))
        };
        let ground = ground3.map(|p| Vec2::new(p.x, p.z));
        let p = ground.map(|g| self.snap(g, inp.free));
        self.cursor = p;
        let hit = pick_part(&self.parts, origin, dir);
        Aim {
            origin,
            dir,
            ground3,
            ground,
            p,
            hit: hit.map(|(part, d)| (part.owner, d)),
            part: hit.map_or(0, |(part, _)| part.slot),
            click: inp.just_pressed && !inp.over_ui && !inp.alt,
            released: inp.just_released && !inp.alt,
        }
    }

    fn tool_frame(&mut self, a: &Aim, inp: &PointerInput, o: &mut Overlay) {
        let hit = a.hit.map(|h| h.0);
        match self.tool {
            Tool::Select => {
                if !inp.over_ui {
                    self.hovered = hit;
                }
                self.select_input(a.p, hit, a.click, inp.pressed, a.released);
            }
            Tool::Wall | Tool::Road => self.segment_frame(a, o),
            Tool::Room | Tool::Floor | Tool::Roof | Tool::Pool => self.rect_frame(a, o),
            Tool::Opening(kind) => self.opening_frame(kind, a, inp.free, o),
            Tool::Prop(kind) => self.prop_frame(kind, a, o),
            Tool::Visitor => self.visitor_frame(a, inp.over_ui, o),
            Tool::Terrain => {
                o.brush = a.ground.map(|g| self.brush_ring(g)).unwrap_or_default();
                self.sculpt_input(a.ground, a.click, inp.pressed, inp.dt);
            }
            Tool::Paint | Tool::Eyedropper | Tool::Erase if !inp.over_ui => {
                self.hovered = hit;
                if let (true, Some(e)) = (a.click, hit) {
                    self.pick_click(e, a.part);
                }
            }
            Tool::Paint | Tool::Eyedropper | Tool::Erase => {}
        }
    }

    fn segment_frame(&mut self, a: &Aim, o: &mut Overlay) {
        let Some(p) = a.p else { return };
        if let Some(start) = self.wall_start {
            o.boxes.push((self.segment(start, p).parts_closed(None), Hint::Preview));
        }
        self.segment_input(p, a.click, a.released);
    }

    fn rect_frame(&mut self, a: &Aim, o: &mut Overlay) {
        let Some(p) = a.p else { return };
        self.rect_input(p, a.click, a.released);
        if let Some(start) = self.drag_start {
            let hint = if big_enough(start, p) { Hint::Preview } else { Hint::TooSmall };
            o.boxes.push((self.rect_result(start, p).parts_closed(None), hint));
        }
    }

    fn opening_frame(&mut self, kind: OpeningKind, a: &Aim, free: bool, o: &mut Overlay) {
        let Some((ElemRef::Wall(wi) | ElemRef::Opening(wi, _), dist)) = a.hit else { return };
        let hp = a.origin + a.dir * dist;
        let Some((op, ok)) = self.opening_at(wi, kind, Vec2::new(hp.x, hp.z), free) else { return };
        let mut w = self.house.walls[wi].clone();
        w.openings.push(op.clone());
        let oi = w.openings.len() - 1;
        let preview: Vec<Part> = House { walls: vec![w], ..default() }
            .parts_closed(None)
            .into_iter()
            .filter(|p| p.owner == ElemRef::Opening(0, oi))
            .collect();
        o.boxes.push((preview, if ok { Hint::Fits } else { Hint::Blocked }));
        self.measure = Some(format!("à {} du début du mur", fmt_m(op.t - op.width / 2.0)));
        if a.click {
            // refus déjà affiché dans le message par Editor::apply
            let _ = self.apply([Command::AddOpening { wall: wi, opening: op }]);
        }
    }

    fn prop_frame(&mut self, kind: PropKind, a: &Aim, o: &mut Overlay) {
        let Some(p) = a.p else { return };
        let prop = self.new_prop(kind, p);
        o.boxes.push((prop_parts(&prop, ElemRef::Prop(0), 0.0, self.house.prop_y(&prop)), Hint::Preview));
        if a.click {
            // refus déjà affiché dans le message par Editor::apply
            let _ = self.apply([Command::AddProp(prop)]);
        }
    }

    /// Le bonhomme se pose sur ce qui est sous le curseur, sinon sur le sol de l'étage.
    fn visitor_frame(&mut self, a: &Aim, over_ui: bool, o: &mut Overlay) {
        if over_ui {
            return;
        }
        let Some(f) = a.hit.map(|(_, d)| a.origin + a.dir * d).or(a.ground3) else { return };
        o.figure = Some(f);
        if a.click {
            self.walk_start = Some(f);
            self.walk = true;
            self.tool = Tool::Select;
        }
    }

    /// Contour du pinceau, épousant le relief.
    fn brush_ring(&self, g: Vec2) -> Vec<Vec3> {
        let r = self.brush_radius;
        (0..=48)
            .map(|k| {
                let q = g + Vec2::from_angle(k as f32 / 48.0 * TAU) * r;
                Vec3::new(q.x, self.house.terrain.height(q) + 0.1, q.y)
            })
            .collect()
    }
}

fn draw_parts(gizmos: &mut Gizmos, parts: &[Part], color: Color) {
    for p in parts.iter().filter(|p| p.shape != Shape::Light) {
        gizmos.cuboid(p.bounds(), color);
    }
}

/// Petit bonhomme filaire (pieds en `f`).
fn draw_figure(gizmos: &mut Gizmos, f: Vec3) {
    let c = Color::srgb(1.0, 0.55, 0.1);
    let at = |x: f32, y: f32| f + Vec3::new(x, y, 0.0);
    gizmos.line(at(-0.15, 0.0), at(0.0, 0.9), c);
    gizmos.line(at(0.15, 0.0), at(0.0, 0.9), c);
    gizmos.line(at(0.0, 0.9), at(0.0, 1.45), c);
    gizmos.line(at(-0.35, 1.0), at(0.0, 1.4), c);
    gizmos.line(at(0.35, 1.0), at(0.0, 1.4), c);
    gizmos.sphere(Isometry3d::from_translation(at(0.0, 1.62)), 0.15, c);
    gizmos.circle(Isometry3d::new(f + Vec3::Y * 0.02, Quat::from_rotation_x(FRAC_PI_2)), 0.35, c);
}

/// Grille de l'étage autour du point visé par la caméra, et les axes x (rouge) et z (bleu).
fn draw_grid(gizmos: &mut Gizmos, grid: f32, base: f32, focus: Vec3) {
    let step = grid.max(0.1);
    let n = ((60.0 / step) as u32).min(300) & !1;
    let c = (Vec2::new(focus.x, focus.z) / step).round() * step;
    gizmos.grid(
        Isometry3d::new(Vec3::new(c.x, base + 0.01, c.y), Quat::from_rotation_x(FRAC_PI_2)),
        UVec2::splat(n),
        Vec2::splat(step),
        Color::srgba(1.0, 1.0, 1.0, 0.12),
    );
    gizmos.line(
        Vec3::new(-1000.0, base + 0.02, 0.0),
        Vec3::new(1000.0, base + 0.02, 0.0),
        Color::srgba(1.0, 0.3, 0.3, 0.5),
    );
    gizmos.line(
        Vec3::new(0.0, base + 0.02, -1000.0),
        Vec3::new(0.0, base + 0.02, 1000.0),
        Color::srgba(0.3, 0.5, 1.0, 0.5),
    );
}

fn draw_overlay(gizmos: &mut Gizmos, o: &Overlay) {
    for (parts, hint) in &o.boxes {
        draw_parts(gizmos, parts, hint.color());
    }
    if let Some(c) = o.cursor {
        gizmos.circle(Isometry3d::new(c, Quat::from_rotation_x(FRAC_PI_2)), 0.15, Hint::Preview.color());
    }
    gizmos.linestrip(o.brush.iter().copied(), Hint::Preview.color());
    if let Some(f) = o.figure {
        draw_figure(gizmos, f);
    }
}

/// Le pointeur est-il sur un panneau egui (les clics ne vont alors pas à la scène) ?
fn pointer_over_ui(ctx: &egui::Context) -> bool {
    ctx.is_pointer_over_area() || ctx.wants_pointer_input()
}

/// Interactions souris dans le monde : outils, aperçus, survol, grille.
pub fn editor_input(
    mut ed: ResMut<Editor>,
    cs: Res<CamState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    cam: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut egui: EguiContexts,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let (camera, cam_tf) = *cam;
    let inp = PointerInput {
        ray: window.cursor_position().and_then(|c| camera.viewport_to_world(cam_tf, c).ok()),
        free: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        alt: keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        over_ui: pointer_over_ui(egui.ctx_mut()),
        pressed: mouse.pressed(MouseButton::Left),
        just_pressed: mouse.just_pressed(MouseButton::Left),
        just_released: mouse.just_released(MouseButton::Left),
        // un clic droit sans glisser annule l'action en cours
        cancel: mouse.just_released(MouseButton::Right) && cs.rmb_travel < 4.0,
        dt: time.delta_secs(),
    };
    if ed.grid_visible() {
        draw_grid(&mut gizmos, ed.grid, elev(ed.level), cs.focus);
    }
    let overlay = ed.pointer(&inp);
    draw_overlay(&mut gizmos, &overlay);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    /// Un éditeur dont la maison contient une table en (0, 0), sans historique.
    fn with_table() -> Editor {
        let mut ed = Editor::default();
        ed.house.props.push(ed.new_prop(PropKind::Table, Vec2::ZERO));
        ed
    }

    /// Un clic (appui et relâché) vu du dessus au point (x, z).
    fn click_at(x: f32, z: f32) -> PointerInput {
        PointerInput {
            ray: Some(Ray3d { origin: Vec3::new(x, 50.0, z), direction: Dir3::NEG_Y }),
            just_pressed: true,
            pressed: true,
            just_released: true,
            dt: 0.1,
            ..default()
        }
    }

    fn refresh(ed: &mut Editor) {
        ed.parts = ed.house.parts_closed(None);
    }

    #[test]
    fn pointer_selects_and_hovers_under_the_cursor() {
        let mut ed = with_table();
        refresh(&mut ed);
        let o = ed.pointer(&PointerInput { just_pressed: true, ..click_at(0.0, 0.0) });
        assert_eq!(ed.selected, Some(ElemRef::Prop(0)));
        assert!(o.boxes.is_empty() && o.cursor.is_none(), "rien à dessiner avant la sélection");
        let o = ed.pointer(&PointerInput { ray: click_at(0.0, 0.0).ray, ..default() });
        assert_eq!(o.boxes[0].1, Hint::Selected);
        assert_eq!(ed.cursor, Some(v(0.0, 0.0)));
        // survol d'un autre élément en mode gomme : en rouge
        ed.house.props.push(ed.new_prop(PropKind::Lamp, v(3.0, 0.0)));
        refresh(&mut ed);
        ed.set_tool(Tool::Erase);
        let o = ed.pointer(&PointerInput { ray: click_at(3.0, 0.0).ray, ..default() });
        assert_eq!(o.boxes.last().map(|b| b.1), Some(Hint::Erase));
        // sur le panneau : pas de survol
        ed.pointer(&PointerInput { over_ui: true, ..click_at(3.0, 0.0) });
        assert_eq!(ed.hovered, None);
        ed.pointer(&click_at(3.0, 0.0));
        assert_eq!(ed.house.props.len(), 1, "la gomme efface la lampe");
    }

    #[test]
    fn pointer_draws_tools_and_places_elements() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Wall);
        let o = ed.pointer(&PointerInput { just_released: false, ..click_at(0.0, 0.0) });
        assert!(o.cursor.is_some());
        let o = ed.pointer(&PointerInput { ray: click_at(4.0, 0.0).ray, ..default() });
        assert_eq!(o.boxes[0].1, Hint::Preview, "aperçu du mur en cours");
        ed.set_tool(Tool::Room);
        ed.pointer(&PointerInput { just_released: false, ..click_at(0.0, 0.0) });
        let o = ed.pointer(&PointerInput { ray: click_at(0.2, 0.2).ray, ..default() });
        assert_eq!(o.boxes[0].1, Hint::TooSmall);
        let o = ed.pointer(&PointerInput { ray: click_at(3.0, 3.0).ray, ..default() });
        assert_eq!(o.boxes[0].1, Hint::Preview);
        ed.set_tool(Tool::Prop(PropKind::Lamp));
        let o = ed.pointer(&click_at(2.0, 2.0));
        assert_eq!((o.boxes[0].1, ed.house.props.len()), (Hint::Preview, 1));
        ed.set_tool(Tool::Terrain);
        let o = ed.pointer(&PointerInput { just_pressed: false, pressed: false, ..click_at(2.0, 2.0) });
        assert_eq!(o.brush.len(), 49);
        assert!(ed.measure.is_some());
    }

    #[test]
    fn pointer_places_openings_on_walls() {
        let mut ed = Editor::default();
        ed.house.walls.push(ed.new_wall(v(0.0, 0.0), v(6.0, 0.0)));
        refresh(&mut ed);
        ed.set_tool(Tool::Opening(OpeningKind::Window));
        let o = ed.pointer(&PointerInput {
            ray: Some(Ray3d { origin: Vec3::new(3.0, 1.5, -5.0), direction: Dir3::Z }),
            just_pressed: true,
            ..default()
        });
        assert_eq!(o.boxes[0].1, Hint::Fits);
        assert_eq!(ed.house.walls[0].openings.len(), 1);
        // à côté du mur : rien
        let o = ed.pointer(&click_at(3.0, 5.0));
        assert!(o.boxes.is_empty());
    }

    #[test]
    fn pointer_drops_the_visitor_and_stops_in_visit() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Visitor);
        let o = ed.pointer(&PointerInput { over_ui: true, ..click_at(1.0, 2.0) });
        assert!(o.figure.is_none() && !ed.walk);
        let o = ed.pointer(&click_at(1.0, 2.0));
        assert!(o.figure.is_some() && ed.walk);
        assert_eq!(ed.tool, Tool::Select);
        assert!(!ed.grid_visible());
        ed.cursor = Some(Vec2::ZERO);
        ed.pointer(&click_at(1.0, 2.0));
        assert_eq!(ed.cursor, None, "en visite, pas de curseur");
        ed.walk = false;
        assert!(ed.grid_visible());
        ed.pointer(&PointerInput { ray: None, ..default() });
        assert_eq!(ed.cursor, None);
    }

    #[test]
    fn pointer_aims_at_the_floor_of_upper_levels_and_cancels() {
        let mut ed = Editor::default();
        ed.set_level(1);
        ed.set_tool(Tool::Floor);
        let o = ed.pointer(&PointerInput { just_released: false, ..click_at(1.0, 1.0) });
        assert!((o.cursor.unwrap().y - elev(1) - 0.03).abs() < 1e-4);
        assert!(ed.drag_start.is_some());
        ed.pointer(&PointerInput { cancel: true, ..default() });
        assert!(ed.drag_start.is_none(), "clic droit : annule le rectangle");
        assert!(!pointer_over_ui(&egui::Context::default()));
    }

    #[test]
    fn hints_have_distinct_colors() {
        let hints =
            [Hint::Selected, Hint::Hovered, Hint::Erase, Hint::Preview, Hint::TooSmall, Hint::Fits, Hint::Blocked];
        let colors: Vec<_> = hints.iter().map(|h| h.color()).collect();
        for (i, c) in colors.iter().enumerate() {
            assert!(!colors[i + 1..].contains(c), "{:?}", hints[i]);
        }
    }
}
