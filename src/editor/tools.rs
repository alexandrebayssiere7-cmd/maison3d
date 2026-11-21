//! Outils de tracé : ce que chaque outil crée, aimantation, et réaction aux clics et glissers.

use bevy::prelude::*;

use super::{Editor, Tool};
use crate::command::Command;
use crate::model::{ElemRef, House, Opening, OpeningKind, Pool, Prop, PropKind, Road, Roof, Slab, Wall, room_walls};

impl Editor {
    pub(super) fn snap(&self, p: Vec2, free: bool) -> Vec2 {
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

    pub(super) fn new_wall(&self, a: Vec2, b: Vec2) -> Wall {
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

    pub(super) fn new_slab(&self, a: Vec2, b: Vec2) -> Slab {
        Slab { min: a.min(b), max: a.max(b), level: self.level, color: self.floor_color, texture: self.floor_texture }
    }

    pub(super) fn new_roof(&self, a: Vec2, b: Vec2) -> Roof {
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

    pub(super) fn new_prop(&self, kind: PropKind, pos: Vec2) -> Prop {
        Prop {
            color: self.prop_color,
            texture: self.prop_texture,
            height: self.prop_height,
            ..Prop::new(kind, pos, self.level, self.prop_rot)
        }
    }

    pub(super) fn new_road(&self, a: Vec2, b: Vec2) -> Road {
        Road { a, b, width: self.road_width, color: self.road_color, texture: self.road_texture }
    }

    /// Ce que l'outil mur ou route poserait entre `a` et `b`.
    pub(super) fn segment(&self, a: Vec2, b: Vec2) -> House {
        match self.tool {
            Tool::Road => House { roads: vec![self.new_road(a, b)], terrain: self.house.terrain.clone(), ..default() },
            _ => House { walls: vec![self.new_wall(a, b)], ..default() },
        }
    }

    pub(super) fn add_segment(&mut self, a: Vec2, b: Vec2) {
        let cmd = match self.tool {
            Tool::Road => Command::AddRoad(self.new_road(a, b)),
            _ => Command::AddWall(self.new_wall(a, b)),
        };
        // refus déjà affiché dans le message par Editor::apply
        let _ = self.apply([cmd]);
    }

    /// Ce que l'outil rectangle en cours créerait entre `a` et `b`.
    pub(super) fn rect_result(&self, a: Vec2, b: Vec2) -> House {
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
                h.pools.push(Pool {
                    min,
                    max,
                    depth: self.pool_depth,
                    color: self.pool_color,
                    texture: self.pool_texture,
                });
                h.terrain = self.house.terrain.clone();
            }
            _ => {}
        }
        h
    }

    /// Outil sélection : clic = sélectionner, glisser = déplacer (une seule commande au relâché).
    /// Bouton non enfoncé = relâché : l'événement se perd hors de la fenêtre ou avec Alt.
    pub(super) fn select_input(&mut self, p: Option<Vec2>, hit: Option<ElemRef>, click: bool, held: bool) {
        if click {
            self.selected = hit;
            if let (Some(_), Some(p)) = (self.selected, p) {
                self.move_drag = Some((p, self.house.clone(), Vec2::ZERO));
            }
        }
        if let (Some(sel), Some(p), Some((start, orig, by)), true) = (self.selected, p, &mut self.move_drag, held) {
            // aperçu en direct, refait seulement si le décalage change ; la vraie commande part au relâché
            if p - *start != *by {
                *by = p - *start;
                let mut h = orig.clone();
                h.translate(sel, *by);
                if h != self.house {
                    self.house = h;
                    self.dirty = true;
                }
            }
            if p != *start {
                self.measure = Some(format!("Δ {:.2} ; {:.2} m", p.x - start.x, p.y - start.y));
            }
        }
        if !held
            && let (Some((_, orig, by)), Some(target)) = (self.move_drag.take(), self.selected)
            && orig != self.house
        {
            self.house = orig;
            if self.apply([Command::Move { target, by }]).is_ok() {
                self.message = "Élément déplacé".into();
            }
        }
    }

    /// Outils mur et route : glisser pose un segment, des clics successifs les enchaînent.
    pub(super) fn segment_input(&mut self, p: Vec2, click: bool, released: bool) {
        if let Some(start) = self.wall_start {
            self.measure = Some(fmt_m(start.distance(p)));
        }
        if click {
            match self.wall_start {
                Some(start) if start.distance(p) > 0.05 => {
                    self.add_segment(start, p);
                    self.wall_start = Some(p);
                }
                Some(_) => {}
                None => {
                    self.wall_start = Some(p);
                    self.drag_start = Some(p);
                }
            }
        }
        // relâché loin du point d'appui : c'était un glisser, on pose un seul mur
        if released
            && let (Some(press), Some(start)) = (self.drag_start.take(), self.wall_start)
            && press == start
            && start.distance(p) > 0.3
        {
            self.add_segment(start, p);
            self.wall_start = None;
            let what = if self.tool == Tool::Road { "Route" } else { "Mur" };
            self.message = format!("{what} de {} posé", fmt_m(start.distance(p)));
        }
    }

    /// Outils rectangle : fonctionne en glisser (appui → relâche) comme en deux clics (coin → coin opposé).
    pub(super) fn rect_input(&mut self, p: Vec2, click: bool, released: bool) {
        let start = match self.drag_start {
            Some(start) if (click || released) && big_enough(start, p) => start,
            _ => {
                if click {
                    self.drag_start = Some(p);
                }
                if let Some(start) = self.drag_start {
                    let size = (p - start).abs();
                    self.measure = Some(format!("{} × {} = {:.1} m²", fmt_m(size.x), fmt_m(size.y), size.x * size.y));
                }
                return;
            }
        };
        let House { walls, slabs, roofs, pools, .. } = self.rect_result(start, p);
        let cmds = walls
            .into_iter()
            .map(Command::AddWall)
            .chain(slabs.into_iter().map(Command::AddSlab))
            .chain(roofs.into_iter().map(Command::AddRoof))
            .chain(pools.into_iter().map(Command::AddPool));
        self.drag_start = None;
        if self.apply(cmds).is_ok() {
            let what = match self.tool {
                Tool::Room => "Pièce",
                Tool::Floor => "Sol",
                Tool::Pool => "Bassin",
                _ => "Toit",
            };
            self.message = format!("{what} créé ({:.1} m²)", (p - start).abs().element_product());
        }
    }

    /// L'ouverture que l'outil poserait sur le mur `wi` au point visé, et si elle y tient.
    pub(super) fn opening_at(&self, wi: usize, kind: OpeningKind, at: Vec2, free: bool) -> Option<(Opening, bool)> {
        // `parts` date de l'image précédente : le mur a pu être annulé entre-temps
        let wall = self.house.walls.get(wi)?;
        let along = (at - wall.a).dot(wall.dir());
        let step = if free { 0.05 } else { self.grid.min(0.25) };
        let t = (along / step).round() * step;
        let (width, height, sill) = self.op_dims[kind as usize];
        let op = Opening { width, height, sill, color: self.op_colors[kind as usize], ..Opening::new(kind, t) };
        Some((op, wall.fits(t, width, None)))
    }

    /// Pinceau de relief : un point d'annulation par geste, sculpte tant que le bouton est tenu.
    pub(super) fn sculpt_input(&mut self, g: Option<Vec2>, click: bool, held: bool, dt: f32) {
        if !held {
            self.sculpting = false;
        }
        let Some(g) = g else { return };
        self.measure = Some(format!("altitude {:.2} m", self.house.terrain.height(g)));
        if click {
            self.checkpoint();
            self.brush_target = self.house.terrain.height(g);
            self.brush_seed = self.brush_seed.wrapping_add(1);
            self.sculpting = true;
        }
        if self.sculpting {
            let (mode, target, seed) = (self.brush, self.brush_target, self.brush_seed);
            self.house.sculpt(mode, g, self.brush_radius, self.brush_strength * dt, target, seed);
            self.dirty = true;
        }
    }

    /// Clic sur la partie `part` d'un élément avec la peinture, la pipette ou la gomme.
    pub(super) fn pick_click(&mut self, e: ElemRef, part: usize) {
        match self.tool {
            Tool::Paint => {
                let (color, texture) = (Some(self.paint_color), Some(self.paint_texture));
                // refus déjà affiché dans le message par Editor::apply
                let _ = self.apply([Command::Paint { target: Some(e), targets: vec![], part, color, texture }]);
            }
            Tool::Eyedropper => {
                if let Some((c, t)) = self.house.finish_mut(e, part) {
                    (self.paint_color, self.paint_texture) = (*c, *t);
                    self.tool = Tool::Paint;
                    self.message = "Couleur récupérée — cliquez pour peindre".into();
                }
            }
            _ => {
                // refus déjà affiché dans le message par Editor::apply
                let _ = self.apply([Command::Remove { target: e }]);
                self.selected = None;
            }
        }
    }
}

/// Le rectangle tracé est-il assez grand pour être posé ?
pub(super) fn big_enough(a: Vec2, p: Vec2) -> bool {
    let s = (p - a).abs();
    s.x >= 0.5 && s.y >= 0.5
}

pub(super) fn fmt_m(v: f32) -> String {
    format!("{v:.2} m")
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

    #[test]
    fn drag_moves_with_a_single_undo_point() {
        let mut ed = with_table();
        let table = Some(ElemRef::Prop(0));
        ed.select_input(Some(v(0.0, 0.0)), table, true, true);
        ed.select_input(Some(v(1.0, 0.0)), None, false, true);
        ed.select_input(Some(v(2.0, 1.0)), None, false, true);
        assert_eq!(ed.house.props[0].pos, v(2.0, 1.0), "aperçu en direct");
        assert!(!ed.can_undo(), "rien n'est encore dans l'historique");
        assert_eq!(ed.measure.as_deref(), Some("Δ 2.00 ; 1.00 m"));
        ed.select_input(Some(v(2.0, 1.0)), None, false, false);
        assert_eq!(ed.house.props[0].pos, v(2.0, 1.0));
        assert_eq!(ed.message, "Élément déplacé");
        ed.undo();
        assert_eq!(ed.house.props[0].pos, Vec2::ZERO);
        assert!(!ed.can_undo());
    }

    #[test]
    fn drag_released_without_a_point_still_moves() {
        let mut ed = with_table();
        ed.select_input(Some(v(0.0, 0.0)), Some(ElemRef::Prop(0)), true, true);
        ed.select_input(Some(v(2.0, 1.0)), None, false, true);
        // bouton relâché hors de la fenêtre : pas de point, et l'événement de relâché est perdu
        ed.select_input(None, None, false, false);
        assert!(ed.can_undo(), "le déplacement est une commande annulable");
        assert_eq!(ed.house.props[0].pos, v(2.0, 1.0));
        ed.undo();
        assert_eq!(ed.house.props[0].pos, Vec2::ZERO);
    }

    /// Lance `act` au milieu d'un glisser de la table puis relâche plus loin : la maison doit être
    /// celle que `act` donne sans glisser, avec un seul point d'annulation vers l'origine.
    fn act_during_drag(act: fn(&mut Editor)) {
        let mut expected = with_table();
        expected.selected = Some(ElemRef::Prop(0));
        act(&mut expected);
        let mut ed = with_table();
        ed.select_input(Some(Vec2::ZERO), Some(ElemRef::Prop(0)), true, true);
        ed.select_input(Some(v(2.0, 1.0)), None, false, true);
        act(&mut ed);
        ed.select_input(Some(v(3.0, 1.0)), None, false, true);
        ed.select_input(Some(v(3.0, 1.0)), None, false, false);
        assert_eq!(ed.house, expected.house, "le glisser est abandonné avant l'action");
        ed.undo();
        assert_eq!(ed.house, with_table().house, "une annulation remet la table à sa place d'origine");
        assert!(!ed.can_undo(), "un seul point d'annulation");
    }

    #[test]
    fn delete_during_a_drag_removes_the_element() {
        act_during_drag(Editor::delete_selected);
    }

    #[test]
    fn duplicate_during_a_drag() {
        act_during_drag(Editor::duplicate_selected);
    }

    #[test]
    fn rotate_during_a_drag() {
        act_during_drag(Editor::rotate_selected);
    }

    #[test]
    fn nudge_during_a_drag() {
        act_during_drag(|ed| ed.nudge_selected(v(0.0, 1.0)));
    }

    #[test]
    fn refused_command_during_a_drag_still_abandons_it() {
        let mut ed = with_table();
        ed.select_input(Some(Vec2::ZERO), Some(ElemRef::Prop(0)), true, true);
        ed.select_input(Some(v(2.0, 1.0)), None, false, true);
        let err = ed.apply([Command::Remove { target: ElemRef::Wall(3) }]).unwrap_err();
        assert!(matches!(err, crate::editor::EditError::NotFound(ElemRef::Wall(3), _)), "{err}");
        ed.select_input(Some(v(3.0, 1.0)), None, false, false);
        assert_eq!(ed.house, with_table().house);
        assert!(!ed.can_undo());
    }

    #[test]
    fn holding_still_during_a_drag_rebuilds_nothing() {
        let mut ed = with_table();
        ed.select_input(Some(Vec2::ZERO), Some(ElemRef::Prop(0)), true, true);
        ed.dirty = false;
        ed.select_input(Some(Vec2::ZERO), None, false, true);
        assert!(!ed.dirty, "pas encore bougé");
        ed.select_input(Some(v(1.0, 0.0)), None, false, true);
        assert!(ed.dirty && ed.house.props[0].pos == v(1.0, 0.0));
        ed.dirty = false;
        // marqueur : un aperçu recalculé repartirait de la maison d'origine et l'effacerait
        ed.house.props[0].color = [0.0; 3];
        ed.select_input(Some(v(1.0, 0.0)), None, false, true);
        assert!(!ed.dirty, "même décalage : l'aperçu n'est pas recalculé");
        assert_eq!(ed.house.props[0].color, [0.0; 3]);
    }

    #[test]
    fn click_without_moving_only_selects() {
        let mut ed = with_table();
        ed.select_input(Some(Vec2::ZERO), Some(ElemRef::Prop(0)), true, true);
        ed.select_input(Some(Vec2::ZERO), None, false, false);
        assert_eq!(ed.selected, Some(ElemRef::Prop(0)));
        assert!(!ed.can_undo());
        // clic dans le vide : désélectionne
        ed.select_input(Some(v(5.0, 5.0)), None, true, true);
        assert_eq!(ed.selected, None);
    }

    #[test]
    fn wall_tool_chains_clicks_and_drags_single_walls() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Wall);
        for p in [v(0.0, 0.0), v(4.0, 0.0), v(4.0, 3.0)] {
            ed.segment_input(p, true, false);
            ed.segment_input(p, false, true);
        }
        assert_eq!(ed.house.walls.len(), 2, "deux clics après le premier : deux murs enchaînés");
        assert_eq!(ed.house.walls[1].a, v(4.0, 0.0));
        ed.segment_input(v(4.0, 3.0), true, false);
        assert_eq!(ed.house.walls.len(), 2, "re-cliquer au même point ne pose rien");
        ed.cancel();

        ed.segment_input(v(0.0, 5.0), true, false);
        ed.segment_input(v(3.0, 5.0), false, false);
        assert_eq!(ed.measure.as_deref(), Some("3.00 m"));
        ed.segment_input(v(3.0, 5.0), false, true);
        assert_eq!(ed.house.walls.len(), 3);
        assert_eq!(ed.message, "Mur de 3.00 m posé");
        assert_eq!(ed.wall_start, None, "un glisser ne s'enchaîne pas");
    }

    #[test]
    fn road_tool_lays_roads() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Road);
        ed.segment_input(v(0.0, 0.0), true, false);
        assert_eq!(ed.segment(v(0.0, 0.0), v(0.0, 9.0)).roads.len(), 1);
        ed.segment_input(v(0.0, 9.0), false, true);
        assert_eq!((ed.house.roads.len(), ed.house.walls.len()), (1, 0));
        assert!(ed.message.starts_with("Route"));
    }

    #[test]
    fn rect_tools_by_drag_or_two_clicks() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Room);
        ed.rect_input(v(0.0, 0.0), true, false);
        ed.rect_input(v(4.0, 3.0), false, false);
        assert_eq!(ed.measure.as_deref(), Some("4.00 m × 3.00 m = 12.0 m²"));
        ed.rect_input(v(4.0, 3.0), false, true);
        assert_eq!((ed.house.walls.len(), ed.house.slabs.len()), (4, 1));
        assert_eq!(ed.message, "Pièce créé (12.0 m²)");

        for (tool, what) in [(Tool::Floor, "Sol"), (Tool::Roof, "Toit"), (Tool::Pool, "Bassin")] {
            ed.set_tool(tool);
            ed.rect_input(v(10.0, 10.0), true, false);
            ed.rect_input(v(10.0, 10.0), false, true);
            ed.rect_input(v(13.0, 12.0), true, false);
            assert!(ed.message.starts_with(what), "{tool:?} : {}", ed.message);
        }
        assert_eq!((ed.house.slabs.len(), ed.house.roofs.len(), ed.house.pools.len()), (2, 1, 1));
    }

    #[test]
    fn rect_too_small_waits_for_the_opposite_corner() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Floor);
        ed.rect_input(v(0.0, 0.0), true, false);
        ed.rect_input(v(0.2, 3.0), false, true);
        assert_eq!(ed.house.slabs, []);
        assert_eq!(ed.drag_start, Some(Vec2::ZERO));
        assert!(!big_enough(Vec2::ZERO, v(0.2, 3.0)) && big_enough(Vec2::ZERO, v(-1.0, 1.0)));
    }

    #[test]
    fn pool_preview_snaps_to_whole_meters() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Pool);
        let h = ed.rect_result(v(0.4, 0.4), v(0.6, 2.7));
        assert_eq!((h.pools[0].min, h.pools[0].max), (v(0.0, 0.0), v(1.0, 3.0)));
        ed.set_tool(Tool::Select);
        assert_eq!(ed.rect_result(Vec2::ZERO, Vec2::ONE), House::default());
    }

    #[test]
    fn opening_tool_snaps_along_the_wall_and_checks_room() {
        let mut ed = Editor::default();
        ed.house.walls.push(ed.new_wall(v(0.0, 0.0), v(4.0, 0.0)));
        let (op, ok) = ed.opening_at(0, OpeningKind::Door, v(2.1, 0.7), false).unwrap();
        assert_eq!((op.t, ok), (2.0, true));
        let (op, _) = ed.opening_at(0, OpeningKind::Door, v(2.12, 0.0), true).unwrap();
        assert!((op.t - 2.1).abs() < 1e-5);
        let (_, ok) = ed.opening_at(0, OpeningKind::Bay, v(0.2, 0.0), false).unwrap();
        assert!(!ok, "dépasse du mur");
        assert!(ed.opening_at(5, OpeningKind::Door, Vec2::ZERO, false).is_none());
    }

    #[test]
    fn sculpting_is_one_undo_point_per_gesture() {
        let mut ed = Editor { brush_strength: 10.0, ..Editor::default() };
        ed.sculpt_input(Some(Vec2::ZERO), true, true, 0.1);
        ed.sculpt_input(Some(Vec2::ZERO), false, true, 0.1);
        assert!((ed.house.terrain.height(Vec2::ZERO) - 2.0).abs() < 0.2);
        ed.sculpt_input(None, false, false, 0.1);
        let h = ed.house.terrain.height(Vec2::ZERO);
        ed.sculpt_input(Some(Vec2::ZERO), false, false, 0.1);
        assert_eq!(ed.house.terrain.height(Vec2::ZERO), h, "bouton relâché : on ne sculpte plus");
        ed.undo();
        assert_eq!(ed.house.terrain.height(Vec2::ZERO), 0.0);
    }

    #[test]
    fn paint_eyedropper_and_erase() {
        let mut ed = with_table();
        ed.set_tool(Tool::Eyedropper);
        ed.house.props[0].texture = 4;
        ed.pick_click(ElemRef::Prop(0), 0);
        assert_eq!((ed.tool, ed.paint_color, ed.paint_texture), (Tool::Paint, ed.house.props[0].color, 4));
        ed.paint_color = [0.0, 0.0, 1.0];
        ed.house.props.push(ed.new_prop(PropKind::Chair, Vec2::X));
        ed.pick_click(ElemRef::Prop(1), 0);
        assert_eq!((ed.house.props[1].color, ed.house.props[1].texture), ([0.0, 0.0, 1.0], 4));
        // les pieds (partie 1) se peignent à part
        ed.paint_color = [1.0, 0.0, 0.0];
        ed.pick_click(ElemRef::Prop(1), 1);
        assert_eq!((ed.house.props[1].color, ed.house.props[1].finish(1).color), ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]));
        ed.set_tool(Tool::Eyedropper);
        ed.pick_click(ElemRef::Prop(1), 1);
        assert_eq!(ed.paint_color, [1.0, 0.0, 0.0], "la pipette lit la partie visée");
        ed.set_tool(Tool::Erase);
        ed.selected = Some(ElemRef::Prop(0));
        ed.pick_click(ElemRef::Prop(0), 0);
        assert_eq!((ed.house.props.len(), ed.selected), (1, None));
    }

    #[test]
    fn snap_prefers_wall_ends_then_grid() {
        let mut ed = Editor::default();
        ed.house.walls.push(ed.new_wall(v(1.13, 2.07), v(5.0, 2.07)));
        assert_eq!(ed.snap(v(1.2, 2.0), false), v(1.13, 2.07));
        assert_eq!(ed.snap(v(3.2, 0.9), false), v(3.0, 1.0));
        assert!((ed.snap(v(3.22, 0.91), true) - v(3.2, 0.9)).length() < 1e-5);
        ed.set_level(1);
        assert_eq!(ed.snap(v(1.2, 2.0), false), v(1.0, 2.0), "les murs des autres étages n'aimantent pas");
    }
}
