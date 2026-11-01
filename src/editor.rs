//! État de l'éditeur, historique d'annulation et interactions souris dans le monde 3D.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::camera::CamState;
use crate::model::*;
use crate::render::MainCamera;
use crate::terrain::Brush;
use crate::textures::find;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Wall,
    Room,
    Floor,
    Roof,
    Pool,
    Road,
    Terrain,
    Opening(OpeningKind),
    Prop(PropKind),
    Paint,
    Eyedropper,
    Erase,
    Visitor,
}

impl Tool {
    pub fn hint(self) -> &'static str {
        match self {
            Tool::Select => "Clic : sélectionner  •  Glisser : déplacer  •  Suppr : effacer  •  R : pivoter  •  Ctrl+D : dupliquer  •  Flèches : décaler",
            Tool::Wall => "Glisser : un mur  •  ou clic coin après coin (les murs s'enchaînent)  •  Échap / clic droit : terminer  •  Maj : sans grille",
            Tool::Room => "Glisser un rectangle (ou cliquer 2 coins opposés) : 4 murs + le sol d'une pièce",
            Tool::Floor => "Glisser un rectangle (ou cliquer 2 coins opposés) : un sol (plancher, dalle, terrasse)",
            Tool::Roof => "Glisser un rectangle (ou 2 clics) au-dessus des murs — montez d'un étage avec + pour le poser sur les murs",
            Tool::Pool => "Glisser un rectangle (ou 2 clics) : une piscine creusée dans le terrain, coins au mètre",
            Tool::Road => "Glisser : un tronçon  •  ou clic point après point (la route s'enchaîne)  •  Échap / clic droit : terminer",
            Tool::Terrain => "Clic maintenu : sculpter le relief avec le pinceau  •  réglez taille et force à gauche",
            Tool::Opening(_) => "Survoler un mur puis cliquer pour poser  •  vert = ça rentre, rouge = pas la place",
            Tool::Prop(_) => "Clic : poser  •  R : pivoter de 45°  •  Maj : placement libre",
            Tool::Paint => "Clic sur un élément : appliquer la couleur",
            Tool::Eyedropper => "Clic sur un élément : récupérer sa couleur (puis passe en peinture)",
            Tool::Erase => "Clic sur un élément : le supprimer",
            Tool::Visitor => "Cliquez où poser le bonhomme (sol, pièce, étage…) pour visiter  •  Tab : annuler",
        }
    }
}

#[derive(Resource)]
pub struct Editor {
    pub house: House,
    undo: Vec<House>,
    redo: Vec<House>,
    pub dirty: bool,
    /// Pièces actuellement affichées (pour la sélection par rayon).
    pub parts: Vec<Part>,
    pub tool: Tool,
    pub level: i32,
    pub hide_upper: bool,
    pub grid: f32,
    pub show_grid: bool,

    pub wall_height: f32,
    pub wall_thick: f32,
    pub wall_color: Rgb,
    pub wall_texture: usize,
    pub floor_color: Rgb,
    pub floor_texture: usize,
    pub roof_kind: RoofKind,
    pub roof_height: f32,
    pub roof_overhang: f32,
    pub roof_color: Rgb,
    pub roof_texture: usize,
    /// (largeur, hauteur, allège) et couleur par type d'ouverture.
    pub op_dims: [(f32, f32, f32); OpeningKind::ALL.len()],
    pub op_colors: [Rgb; OpeningKind::ALL.len()],
    pub prop_rot: f32,
    pub prop_height: f32,
    pub pool_depth: f32,
    pub pool_color: Rgb,
    pub pool_texture: usize,
    pub road_width: f32,
    pub road_color: Rgb,
    pub road_texture: usize,
    pub brush: Brush,
    pub brush_radius: f32,
    /// Mètres par seconde de clic maintenu.
    pub brush_strength: f32,
    /// Altitude visée par « Aplanir » : celle sous le pinceau au début du geste.
    brush_target: f32,
    pub brush_seed: u32,
    sculpting: bool,
    pub gen_amp: f32,
    pub gen_scale: f32,
    pub gen_mountains: bool,
    pub prop_color: Rgb,
    pub prop_texture: usize,
    pub paint_color: Rgb,
    pub paint_texture: usize,

    pub selected: Option<ElemRef>,
    pub hovered: Option<ElemRef>,
    wall_start: Option<Vec2>,
    drag_start: Option<Vec2>,
    move_drag: Option<(Vec2, House)>,
    pub cursor: Option<Vec2>,
    /// Cote affichée près du pointeur (longueur, surface…).
    pub measure: Option<String>,
    pub message: String,
    pub file: String,
    pub hour: f32,
    pub walk: bool,
    /// Où le bonhomme a été posé (pieds), consommé par la caméra à l'entrée en visite.
    pub walk_start: Option<Vec3>,
    /// Portes et placards : (ouverture actuelle 0..1, ouvert demandé). État de visite, non enregistré.
    pub doors: HashMap<ElemRef, (f32, bool)>,
    pub show_help: bool,
    /// Une modification de propriété est en cours (un seul point d'annulation par geste).
    pub prop_edit_active: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            house: House::default(),
            undo: vec![],
            redo: vec![],
            dirty: true,
            parts: vec![],
            tool: Tool::Select,
            level: 0,
            hide_upper: false,
            grid: 0.5,
            show_grid: true,
            wall_height: LEVEL_H,
            wall_thick: 0.2,
            wall_color: WHITE_TEX,
            wall_texture: find("Crépi blanc"),
            floor_color: WHITE_TEX,
            floor_texture: find("Parquet chêne clair"),
            roof_kind: RoofKind::Gable,
            roof_height: 2.5,
            roof_overhang: 0.5,
            roof_color: WHITE_TEX,
            roof_texture: find("Tuiles terre cuite"),
            op_dims: OpeningKind::ALL.map(|k| k.default_dims()),
            op_colors: OpeningKind::ALL.map(|k| k.default_color()),
            prop_rot: 0.0,
            prop_height: LEVEL_H,
            pool_depth: 1.5,
            pool_color: WHITE_TEX,
            pool_texture: find("Mosaïque bleue"),
            road_width: 4.0,
            road_color: [0.55, 0.55, 0.57],
            road_texture: find("Béton brut"),
            brush: Brush::Raise,
            brush_radius: 6.0,
            brush_strength: 2.0,
            brush_target: 0.0,
            brush_seed: 1,
            sculpting: false,
            gen_amp: 8.0,
            gen_scale: 40.0,
            gen_mountains: false,            prop_color: PropKind::Table.default_color(),
            prop_texture: 0,
            paint_color: [0.8, 0.3, 0.3],
            paint_texture: 0,
            selected: None,
            hovered: None,
            wall_start: None,
            drag_start: None,
            move_drag: None,
            cursor: None,
            measure: None,
            message: String::new(),
            file: "maison.json".into(),
            hour: 14.0,
            walk: false,
            walk_start: None,
            doors: HashMap::new(),
            show_help: true,
            prop_edit_active: false,
        }
    }
}

const MAX_UNDO: usize = 300;

impl Editor {
    /// À appeler juste avant une modification de la maison.
    pub fn checkpoint(&mut self) {
        let snapshot = self.house.clone();
        self.push_undo(snapshot);
    }

    pub fn push_undo(&mut self, h: House) {
        self.undo.push(h);
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
        // les indices des éléments peuvent changer : on referme tout
        self.doors.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        if let Some(h) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.house, h));
            self.after_restore("Annulé");
        }
    }

    pub fn redo(&mut self) {
        if let Some(h) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.house, h));
            self.after_restore("Rétabli");
        }
    }

    fn after_restore(&mut self, msg: &str) {
        self.selected = None;
        self.doors.clear();
        self.cancel();
        self.dirty = true;
        self.message = msg.into();
    }

    /// Remplace toute la maison (nouveau, exemple, ouverture) de façon annulable.
    pub fn replace_house(&mut self, h: House, msg: String) {
        self.checkpoint();
        self.house = h;
        self.after_restore(&msg);
    }

    /// Abandonne l'action en cours (mur en cours de tracé, rectangle, déplacement).
    pub fn cancel(&mut self) {
        self.wall_start = None;
        self.drag_start = None;
        if let Some((_, orig)) = self.move_drag.take() {
            self.house = orig;
            self.dirty = true;
        }
    }

    pub fn set_tool(&mut self, t: Tool) {
        self.cancel();
        if let Tool::Prop(k) = t {
            if self.tool != t {
                self.prop_color = k.default_color();
                self.prop_texture = k.default_texture();
            }
        }
        self.tool = t;
    }

    pub fn toggle_door(&mut self, e: ElemRef) {
        let d = self.doors.entry(e).or_insert((0.0, false));
        d.1 = !d.1;
    }

    /// Fait pivoter progressivement les portes vers leur état demandé.
    pub fn animate_doors(&mut self, dt: f32) {
        for (cur, open) in self.doors.values_mut() {
            let target = if *open { 1.0 } else { 0.0 };
            if *cur != target {
                *cur = if *open { (*cur + dt * 2.5).min(1.0) } else { (*cur - dt * 2.5).max(0.0) };
                self.dirty = true;
            }
        }
    }

    pub fn set_level(&mut self, l: i32) {
        self.cancel();
        self.level = l.clamp(-1, 9);
        self.dirty = true;
    }

    pub fn delete_selected(&mut self) {
        if let Some(e) = self.selected.take() {
            self.checkpoint();
            self.house.remove(e);
        }
    }

    pub fn duplicate_selected(&mut self) {
        if let Some(e) = self.selected {
            let mut h = self.house.clone();
            if let Some(new) = h.duplicate(e, Vec2::splat(self.grid.max(0.5))) {
                self.checkpoint();
                self.house = h;
                self.selected = Some(new);
            }
        }
    }

    pub fn rotate_selected(&mut self) {
        if let Some(ElemRef::Prop(i)) = self.selected {
            self.checkpoint();
            let p = &mut self.house.props[i];
            p.rot = (p.rot + 45.0) % 360.0;
        }
    }

    pub fn nudge_selected(&mut self, d: Vec2) {
        if let Some(e) = self.selected {
            self.checkpoint();
            self.house.translate(e, d);
        }
    }

    pub fn save(&mut self) {
        let res = serde_json::to_string_pretty(&self.house)
            .map_err(|e| e.to_string())
            .and_then(|s| std::fs::write(&self.file, s).map_err(|e| e.to_string()));
        self.message = match res {
            Ok(()) => format!("Enregistré dans {}", self.file),
            Err(e) => format!("Erreur d'enregistrement : {e}"),
        };
    }

    pub fn load(&mut self) {
        let res = std::fs::read_to_string(&self.file)
            .map_err(|e| e.to_string())
            .and_then(|s| serde_json::from_str::<House>(&s).map_err(|e| e.to_string()));
        match res {
            Ok(h) => self.replace_house(h, format!("Ouvert : {}", self.file)),
            Err(e) => self.message = format!("Impossible d'ouvrir {} : {e}", self.file),
        }
    }

    fn snap(&self, p: Vec2, free: bool) -> Vec2 {
        // aimantation prioritaire sur les extrémités de murs existants
        let roads = self.house.roads.iter().filter(|_| self.level == 0).flat_map(|r| [r.a, r.b]);
        let best = self
            .house
            .walls
            .iter()
            .filter(|w| w.level == self.level)
            .flat_map(|w| [w.a, w.b])
            .chain(roads)
            .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
        if let Some(b) = best.filter(|b| b.distance(p) < 0.3) {
            return b;
        }
        let step = if free { 0.05 } else { self.grid };
        (p / step).round() * step
    }

    fn new_wall(&self, a: Vec2, b: Vec2) -> Wall {
        Wall {
            a,
            b,
            level: self.level,
            height: self.wall_height,
            thick: self.wall_thick,
            color: self.wall_color,
            openings: vec![],
            texture: self.wall_texture,
        }
    }

    fn new_slab(&self, a: Vec2, b: Vec2) -> Slab {
        Slab { min: a.min(b), max: a.max(b), level: self.level, color: self.floor_color, texture: self.floor_texture }
    }

    fn new_roof(&self, a: Vec2, b: Vec2) -> Roof {
        Roof {
            min: a.min(b),
            max: a.max(b),
            level: self.level,
            kind: self.roof_kind,
            height: self.roof_height,
            overhang: self.roof_overhang,
            color: self.roof_color,
            texture: self.roof_texture,
        }
    }

    fn new_prop(&self, kind: PropKind, pos: Vec2) -> Prop {
        Prop { kind, pos, level: self.level, rot: self.prop_rot, color: self.prop_color, texture: self.prop_texture, height: self.prop_height }
    }

    fn new_road(&self, a: Vec2, b: Vec2) -> Road {
        Road { a, b, width: self.road_width, color: self.road_color, texture: self.road_texture }
    }

    /// Ce que l'outil mur ou route poserait entre `a` et `b`.
    fn segment(&self, a: Vec2, b: Vec2) -> House {
        match self.tool {
            Tool::Road => House { roads: vec![self.new_road(a, b)], terrain: self.house.terrain.clone(), ..default() },
            _ => House { walls: vec![self.new_wall(a, b)], ..default() },
        }
    }

    fn add_segment(&mut self, a: Vec2, b: Vec2) {
        self.checkpoint();
        match self.tool {
            Tool::Road => self.house.add_road(self.new_road(a, b)),
            _ => self.house.walls.push(self.new_wall(a, b)),
        }
    }

    /// Ce que l'outil rectangle en cours créerait entre `a` et `b`.
    fn rect_result(&self, a: Vec2, b: Vec2) -> House {
        let mut h = House::default();
        let (min, max) = (a.min(b), a.max(b));
        match self.tool {
            Tool::Room => {
                h.walls = room_walls(min, max, self.level, self.wall_height, self.wall_thick, self.wall_color);
                for w in &mut h.walls {
                    w.texture = self.wall_texture;
                }
                h.slabs.push(self.new_slab(a, b));
            }
            Tool::Floor => h.slabs.push(self.new_slab(a, b)),
            Tool::Roof => h.roofs.push(self.new_roof(a, b)),
            Tool::Pool => {
                let (min, max) = (min.round(), max.round().max(min.round() + 1.0));
                h.pools.push(Pool { min, max, depth: self.pool_depth, color: self.pool_color, texture: self.pool_texture });
                h.terrain = self.house.terrain.clone();
            }
            _ => {}
        }
        h
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

fn fmt_m(v: f32) -> String {
    format!("{v:.2} m")
}

/// Interactions souris dans le monde : outils, aperçus, survol, grille.
#[allow(clippy::too_many_arguments)]
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
    let ctx = egui.ctx_mut();
    let over_ui = ctx.is_pointer_over_area() || ctx.wants_pointer_input();
    let ed = &mut *ed;
    let base = elev(ed.level);

    if ed.show_grid && !ed.walk {
        let step = ed.grid.max(0.1);
        let n = ((60.0 / step) as u32).min(300) & !1;
        let c = (Vec2::new(cs.focus.x, cs.focus.z) / step).round() * step;
        gizmos.grid(
            Isometry3d::new(Vec3::new(c.x, base + 0.01, c.y), Quat::from_rotation_x(FRAC_PI_2)),
            UVec2::splat(n),
            Vec2::splat(step),
            Color::srgba(1.0, 1.0, 1.0, 0.12),
        );
        gizmos.line(Vec3::new(-1000.0, base + 0.02, 0.0), Vec3::new(1000.0, base + 0.02, 0.0), Color::srgba(1.0, 0.3, 0.3, 0.5));
        gizmos.line(Vec3::new(0.0, base + 0.02, -1000.0), Vec3::new(0.0, base + 0.02, 1000.0), Color::srgba(0.3, 0.5, 1.0, 0.5));
    }
    if let Some(sel) = ed.selected {
        let parts: Vec<Part> = ed.parts.iter().filter(|p| p.owner == sel).cloned().collect();
        draw_parts(&mut gizmos, &parts, Color::srgb(1.0, 0.85, 0.1));
    }

    // un clic droit sans glisser annule l'action en cours
    if mouse.just_released(MouseButton::Right) && cs.rmb_travel < 4.0 {
        ed.cancel();
    }

    ed.hovered = None;
    ed.measure = None;
    ed.cursor = None;
    if ed.walk {
        return;
    }
    let (camera, cam_tf) = *cam;
    let Some(ray) = window.cursor_position().and_then(|c| camera.viewport_to_world(cam_tf, c).ok()) else {
        return;
    };
    let (origin, dir) = (ray.origin, *ray.direction);
    let free = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    // au rez-de-chaussée on vise le relief, ailleurs le plancher de l'étage
    let ground3 = if ed.level == 0 {
        ed.house.terrain.ray(origin, dir)
    } else {
        ray.intersect_plane(Vec3::new(0.0, base, 0.0), InfinitePlane3d::new(Vec3::Y)).map(|t| ray.get_point(t))
    };
    let ground = ground3.map(|p| Vec2::new(p.x, p.z));
    let p = ground.map(|g| ed.snap(g, free));
    ed.cursor = p;
    let hit = pick(&ed.parts, origin, dir);

    // Alt + clic gauche est réservé à la caméra
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let click = mouse.just_pressed(MouseButton::Left) && !over_ui && !alt;
    let released = mouse.just_released(MouseButton::Left) && !alt;
    let preview = Color::srgb(0.3, 0.9, 1.0);

    if let Some(p) = p {
        if !matches!(ed.tool, Tool::Select | Tool::Paint | Tool::Eyedropper | Tool::Erase | Tool::Opening(_) | Tool::Visitor | Tool::Terrain) && !over_ui {
            let y = if ed.level == 0 { ed.house.terrain.height(p) } else { base };
            gizmos.circle(Isometry3d::new(Vec3::new(p.x, y + 0.03, p.y), Quat::from_rotation_x(FRAC_PI_2)), 0.15, preview);
        }
    }

    match ed.tool {
        Tool::Select => {
            if !over_ui {
                ed.hovered = hit.map(|h| h.0);
            }
            if click {
                ed.selected = hit.map(|h| h.0);
                if let (Some(_), Some(p)) = (ed.selected, p) {
                    ed.move_drag = Some((p, ed.house.clone()));
                }
            }
            if let (Some(sel), Some(p), Some((start, orig))) = (ed.selected, p, &ed.move_drag) {
                if mouse.pressed(MouseButton::Left) {
                    let mut h = orig.clone();
                    h.translate(sel, p - *start);
                    if h != ed.house {
                        ed.house = h;
                        ed.dirty = true;
                    }
                    if p != *start {
                        ed.measure = Some(format!("Δ {:.2} ; {:.2} m", p.x - start.x, p.y - start.y));
                    }
                }
            }
            if released {
                if let Some((_, orig)) = ed.move_drag.take() {
                    if orig != ed.house {
                        ed.push_undo(orig);
                        ed.message = "Élément déplacé".into();
                    }
                }
            }
        }
        Tool::Wall | Tool::Road => {
            let Some(p) = p else { return };
            if let Some(start) = ed.wall_start {
                draw_parts(&mut gizmos, &ed.segment(start, p).parts_closed(None), preview);
                ed.measure = Some(fmt_m(start.distance(p)));
            }
            if click {
                match ed.wall_start {
                    Some(start) if start.distance(p) > 0.05 => {
                        ed.add_segment(start, p);
                        ed.wall_start = Some(p);
                    }
                    Some(_) => {}
                    None => {
                        ed.wall_start = Some(p);
                        ed.drag_start = Some(p);
                    }
                }
            }
            // relâché loin du point d'appui : c'était un glisser, on pose un seul mur
            if released {
                if let (Some(press), Some(start)) = (ed.drag_start.take(), ed.wall_start) {
                    if press == start && start.distance(p) > 0.3 {
                        ed.add_segment(start, p);
                        ed.wall_start = None;
                        let what = if ed.tool == Tool::Road { "Route" } else { "Mur" };
                        ed.message = format!("{what} de {} posé", fmt_m(start.distance(p)));
                    }
                }
            }
        }
        Tool::Room | Tool::Floor | Tool::Roof | Tool::Pool => {
            let Some(p) = p else { return };
            let big_enough = |a: Vec2| {
                let s = (p - a).abs();
                s.x >= 0.5 && s.y >= 0.5
            };
            // fonctionne en glisser (appui → relâche) comme en deux clics (coin → coin opposé)
            let finish = match ed.drag_start {
                Some(start) if (click || released) && big_enough(start) => Some(start),
                _ => {
                    if click {
                        ed.drag_start = Some(p);
                    }
                    None
                }
            };
            if let Some(start) = finish {
                let House { walls, slabs, roofs, pools, .. } = ed.rect_result(start, p);
                ed.checkpoint();
                ed.house.walls.extend(walls);
                ed.house.slabs.extend(slabs);
                ed.house.roofs.extend(roofs);
                for pool in pools {
                    ed.house.add_pool(pool);
                }
                ed.drag_start = None;
                ed.message = format!("{} créé ({:.1} m²)", match ed.tool {
                    Tool::Room => "Pièce",
                    Tool::Floor => "Sol",
                    Tool::Pool => "Bassin",
                    _ => "Toit",
                }, (p - start).abs().element_product());
            } else if let Some(start) = ed.drag_start {
                let size = (p - start).abs();
                let color = if big_enough(start) { preview } else { Color::srgb(1.0, 0.6, 0.2) };
                draw_parts(&mut gizmos, &ed.rect_result(start, p).parts_closed(None), color);
                ed.measure = Some(format!("{} × {} = {:.1} m²", fmt_m(size.x), fmt_m(size.y), size.x * size.y));
            }
        }
        Tool::Opening(kind) => {
            let Some((ElemRef::Wall(wi) | ElemRef::Opening(wi, _), dist)) = hit else { return };
            let wall = &ed.house.walls[wi];
            let hp = origin + dir * dist;
            let along = (Vec2::new(hp.x, hp.z) - wall.a).dot(wall.dir());
            let step = if free { 0.05 } else { ed.grid.min(0.25) };
            let t = (along / step).round() * step;
            let (width, height, sill) = ed.op_dims[kind as usize];
            let ok = wall.fits(t, width, None);
            let op = Opening { kind, t, width, height, sill, color: ed.op_colors[kind as usize] };
            let mut w = wall.clone();
            w.openings.push(op.clone());
            let oi = w.openings.len() - 1;
            let preview_parts: Vec<Part> = House { walls: vec![w], ..default() }
                .parts_closed(None)
                .into_iter()
                .filter(|p| p.owner == ElemRef::Opening(0, oi))
                .collect();
            let color = if ok { Color::srgb(0.3, 1.0, 0.4) } else { Color::srgb(1.0, 0.3, 0.3) };
            draw_parts(&mut gizmos, &preview_parts, color);
            ed.measure = Some(format!("à {} du début du mur", fmt_m(t - width / 2.0)));
            if click {
                if ok {
                    ed.checkpoint();
                    ed.house.walls[wi].openings.push(op);
                } else {
                    ed.message = format!("Pas la place pour : {}", kind.name());
                }
            }
        }
        Tool::Prop(kind) => {
            let Some(p) = p else { return };
            let prop = ed.new_prop(kind, p);
            draw_parts(&mut gizmos, &prop_parts(&prop, ElemRef::Prop(0), 0.0, ed.house.prop_y(&prop)), preview);
            if click {
                ed.checkpoint();
                ed.house.props.push(prop);
            }
        }
        Tool::Visitor => {}
        Tool::Terrain => {
            if !mouse.pressed(MouseButton::Left) {
                ed.sculpting = false;
            }
            let Some(g) = ground else { return };
            // contour du pinceau, épousant le relief
            let r = ed.brush_radius;
            let ring: Vec<Vec3> = (0..=48)
                .map(|k| {
                    let q = g + Vec2::from_angle(k as f32 / 48.0 * TAU) * r;
                    Vec3::new(q.x, ed.house.terrain.height(q) + 0.1, q.y)
                })
                .collect();
            gizmos.linestrip(ring, preview);
            ed.measure = Some(format!("altitude {:.2} m", ed.house.terrain.height(g)));
            if click {
                ed.checkpoint();
                ed.brush_target = ed.house.terrain.height(g);
                ed.brush_seed = ed.brush_seed.wrapping_add(1);
                ed.sculpting = true;
            }
            if ed.sculpting {
                let amount = ed.brush_strength * time.delta_secs();
                let (mode, target, seed) = (ed.brush, ed.brush_target, ed.brush_seed);
                ed.house.sculpt(mode, g, r, amount, target, seed);
                ed.dirty = true;
            }
        }
        Tool::Paint | Tool::Eyedropper | Tool::Erase => {
            if over_ui {
                return;
            }
            ed.hovered = hit.map(|h| h.0);
            let (true, Some((e, _))) = (click, hit) else { return };
            match ed.tool {
                Tool::Paint => {
                    let (c, t) = (ed.paint_color, ed.paint_texture);
                    ed.checkpoint();
                    if let Some(col) = ed.house.color_mut(e) {
                        *col = c;
                    }
                    if let Some(tex) = ed.house.texture_mut(e) {
                        *tex = t;
                    }
                }
                Tool::Eyedropper => {
                    if let Some(c) = ed.house.color_mut(e).copied() {
                        ed.paint_color = c;
                        ed.paint_texture = ed.house.texture_mut(e).map_or(0, |t| *t);
                        ed.tool = Tool::Paint;
                        ed.message = "Couleur récupérée — cliquez pour peindre".into();
                    }
                }
                _ => {
                    ed.checkpoint();
                    ed.house.remove(e);
                    ed.selected = None;
                }
            }
        }
    }

    if ed.tool == Tool::Visitor && !over_ui {
        // le bonhomme se pose sur ce qui est sous le curseur, sinon sur le sol de l'étage
        let feet = hit.map(|(_, d)| origin + dir * d).or(ground3);
        if let Some(f) = feet {
            draw_figure(&mut gizmos, f);
            if click {
                ed.walk_start = Some(f);
                ed.walk = true;
                ed.tool = Tool::Select;
            }
        }
    }

    if let Some(h) = ed.hovered.filter(|h| Some(*h) != ed.selected) {
        let color = if ed.tool == Tool::Erase { Color::srgb(1.0, 0.25, 0.25) } else { Color::srgba(1.0, 1.0, 1.0, 0.7) };
        let parts: Vec<Part> = ed.parts.iter().filter(|p| p.owner == h).cloned().collect();
        draw_parts(&mut gizmos, &parts, color);
    }
}
