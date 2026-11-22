//! Gestes du panneau de propriétés : chaque modification repasse par `Command::Set`.

use serde_json::{Map, Value};

use super::Editor;
use crate::command::{Command, EditError};
use crate::model::{ElemRef, House};

impl Editor {
    /// Après un geste du panneau sur l'élément sélectionné, déjà modifié dans `self.house` : la
    /// modification repasse par `Command::Set`, validée comme une commande. Un seul point
    /// d'annulation par geste ; un refus rend la maison `before` et le dit dans le message.
    /// Pendant un geste, `set` part de la maison du début du geste (le point d'annulation) :
    /// élargir puis rétrécir une route ne laisse pas le terrain aplani en plus.
    pub fn record_prop_edit(&mut self, before: House) -> Result<(), EditError> {
        let edited = std::mem::replace(&mut self.house, before);
        // le panneau ne modifie que la sélection : sans elle, rien à rejouer
        let Some(target) = self.selected.filter(|_| edited != self.house) else { return Ok(()) };
        let start = if self.prop_edit_active { self.undo.back() } else { None };
        let mut h = start.unwrap_or(&self.house).clone();
        let fields = elem_fields(&edited, target).map_err(|e| EditError::BadFields(e.to_string()));
        if let Err(e) = fields.and_then(|fields| h.apply(Command::Set { target: target.into(), fields })) {
            self.message = e.to_string();
            return Err(e);
        }
        let before = std::mem::replace(&mut self.house, h);
        if !self.prop_edit_active {
            self.push_undo(before);
            self.prop_edit_active = true;
        }
        self.dirty = true;
        self.message.clear();
        Ok(())
    }
}

/// Tous les champs de l'élément `e` de `h` ; aucun s'il n'existe pas (`set` le refusera).
fn elem_fields(h: &House, e: ElemRef) -> Result<Map<String, Value>, serde_json::Error> {
    let v = match e {
        ElemRef::Wall(i) => h.walls.get(i).map(serde_json::to_value),
        ElemRef::Opening(w, o) => h.walls.get(w).and_then(|w| w.openings.get(o)).map(serde_json::to_value),
        ElemRef::Slab(i) => h.slabs.get(i).map(serde_json::to_value),
        ElemRef::Roof(i) => h.roofs.get(i).map(serde_json::to_value),
        ElemRef::Prop(i) => h.props.get(i).map(serde_json::to_value),
        ElemRef::Pool(i) => h.pools.get(i).map(serde_json::to_value),
        ElemRef::Road(i) => h.roads.get(i).map(serde_json::to_value),
    };
    v.unwrap_or(Ok(Value::Null)).map(|v| if let Value::Object(m) = v { m } else { Map::new() })
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::command::{Command, EditError};
    use crate::editor::Editor;
    use crate::model::{ElemRef, House, OpeningKind, demo_house};

    /// Un mur de 4 m sur 2,5 m de haut, une fenêtre au milieu, sans historique.
    fn wall_with_window() -> Editor {
        let mut ed = Editor::default();
        let mut w = ed.new_wall(Vec2::ZERO, Vec2::new(4.0, 0.0));
        w.height = 2.5;
        let place = Command::PlaceOpening { wall: 0, kind: OpeningKind::Window, t: 2.0 };
        assert!(ed.apply([Command::AddWall(w), place]).is_ok());
        ed.undo.clear();
        ed.selected = Some(ElemRef::Opening(0, 0));
        ed
    }

    #[test]
    fn refused_panel_edit_leaves_the_house_intact() {
        let mut ed = wall_with_window();
        let before = ed.house.clone();
        // allège 2,2 + hauteur 1,2 = 3,4 m sur un mur de 2,5 m
        let o = &mut ed.house.walls[0].openings[0];
        (o.sill, o.height) = (2.2, 1.2);
        let res = ed.record_prop_edit(before.clone());
        assert!(matches!(res, Err(EditError::OpeningTooTall { .. })), "{res:?}");
        assert_eq!(ed.house, before, "la maison reste celle d avant le geste");
        assert!(!ed.can_undo() && !ed.message.is_empty());
        assert!(ed.house.validate().is_ok(), "le monde se recharge toujours");
    }

    #[test]
    fn panel_cannot_push_an_opening_out_of_its_wall() {
        let mut ed = wall_with_window();
        let before = ed.house.clone();
        // le curseur de largeur poussé au bout : toute la longueur du mur
        ed.house.walls[0].openings[0].width = 4.0;
        let res = ed.record_prop_edit(before.clone());
        assert!(matches!(res, Err(EditError::OpeningOutsideWall { .. })), "{res:?}");
        assert_eq!(ed.house, before);
    }

    #[test]
    fn panel_edits_make_one_undo_point_per_gesture() {
        let mut ed = wall_with_window();
        for sill in [1.0, 1.1] {
            let before = ed.house.clone();
            ed.house.walls[0].openings[0].sill = sill;
            assert!(ed.record_prop_edit(before).is_ok());
        }
        assert!(ed.prop_edit_active && ed.dirty);
        ed.undo();
        assert_eq!(ed.house.walls[0].openings[0].sill, 0.9);
        assert!(!ed.can_undo(), "un seul point d annulation");
    }

    #[test]
    fn accepted_panel_edit_goes_through_set() {
        let mut ed = wall_with_window();
        let before = ed.house.clone();
        ed.house.walls[0].openings[0].sill = 1.0;
        assert!(ed.record_prop_edit(before.clone()).is_ok());
        assert_eq!(ed.house.walls[0].openings[0].sill, 1.0);
        assert!(ed.can_undo() && ed.dirty && ed.house.validate().is_ok());
        ed.undo();
        assert_eq!(ed.house, before);
    }

    #[test]
    fn road_width_from_the_panel_regrades_like_set() {
        let mut ed = Editor { house: demo_house(), ..Default::default() };
        ed.selected = Some(ElemRef::Road(0));
        let fields = serde_json::from_str(r#"{"width": 8}"#).unwrap();
        let mut expected = ed.house.clone();
        assert!(expected.apply(Command::Set { target: ElemRef::Road(0).into(), fields }).is_ok());
        let before = ed.house.clone();
        ed.house.roads[0].width = 8.0;
        assert!(ed.record_prop_edit(before).is_ok());
        assert_eq!(ed.house, expected, "le terrain est retassé comme par set");
    }

    /// Une image du panneau : `edit` modifie la maison, puis le geste est rejoué.
    fn frame(ed: &mut Editor, edit: impl FnOnce(&mut House)) -> Result<(), EditError> {
        let before = ed.house.clone();
        edit(&mut ed.house);
        ed.record_prop_edit(before)
    }

    #[test]
    fn widening_then_narrowing_a_road_in_one_gesture_does_not_flatten_more() {
        let mut ed = Editor { house: demo_house(), ..Default::default() };
        let relief = Command::GenerateTerrain { seed: 7, amp: 8.0, scale: 20.0, mountains: false };
        assert!(ed.house.apply(relief).is_ok());
        ed.selected = Some(ElemRef::Road(0));
        let mut expected = ed.house.clone();
        let fields = serde_json::from_str(r#"{"width": 4}"#).unwrap();
        assert!(expected.apply(Command::Set { target: ElemRef::Road(0).into(), fields }).is_ok());
        assert!(frame(&mut ed, |h| h.roads[0].width = 12.0).is_ok());
        assert_ne!(ed.house.terrain, expected.terrain, "12 m de large aplanit plus");
        assert!(frame(&mut ed, |h| h.roads[0].width = 4.0).is_ok());
        assert_eq!(ed.house, expected, "revenir à 4 m dans le même geste rend le terrain d avant");
    }

    #[test]
    fn a_command_during_a_gesture_is_kept() {
        let mut ed = wall_with_window();
        assert!(frame(&mut ed, |h| h.walls[0].openings[0].sill = 1.0).is_ok());
        let place = Command::PlaceOpening { wall: 0, kind: OpeningKind::Door, t: 0.6 };
        assert!(ed.apply([place]).is_ok());
        assert!(frame(&mut ed, |h| h.walls[0].openings[0].sill = 1.1).is_ok());
        assert_eq!(ed.house.walls[0].openings.len(), 2, "la porte posée en plein geste reste");
        assert_eq!(ed.house.walls[0].openings[0].sill, 1.1);
        ed.undo();
        assert!(!ed.prop_edit_active, "annuler clôt le geste");
        assert_eq!(ed.house.walls[0].openings.len(), 2);
    }

    #[test]
    fn accepted_edit_clears_the_previous_refusal() {
        let mut ed = wall_with_window();
        let res = frame(&mut ed, |h| h.walls[0].openings[0].sill = 2.2);
        assert!(matches!(res, Err(EditError::OpeningTooTall { .. })), "{res:?}");
        assert_ne!(ed.message, "");
        assert!(frame(&mut ed, |h| h.walls[0].openings[0].sill = 1.0).is_ok());
        assert!(ed.message.is_empty(), "{}", ed.message);
    }

    #[test]
    fn every_kind_of_selection_can_be_edited() {
        let mut ed = Editor { house: demo_house(), ..Default::default() };
        ed.house.add_storey(0, None);
        let targets = [ElemRef::Wall(0), ElemRef::Opening(0, 0), ElemRef::Slab(0), ElemRef::Roof(0)];
        for t in targets.into_iter().chain([ElemRef::Prop(0), ElemRef::Pool(0), ElemRef::Road(0)]) {
            ed.selected = Some(t);
            let before = ed.house.clone();
            let paint = Command::Paint {
                target: Some(t.into()),
                targets: vec![],
                part: 0,
                color: Some([0.1, 0.2, 0.3]),
                texture: None,
            };
            assert!(ed.house.apply(paint).is_ok(), "{t:?}");
            assert!(ed.record_prop_edit(before.clone()).is_ok(), "{t:?} : {}", ed.message);
            assert_ne!(ed.house, before, "{t:?}");
        }
    }

    #[test]
    fn panel_edit_without_change_or_target_changes_nothing() {
        let mut ed = wall_with_window();
        let before = ed.house.clone();
        assert!(ed.record_prop_edit(before.clone()).is_ok());
        ed.selected = None;
        ed.house.walls[0].height = 3.0;
        assert!(ed.record_prop_edit(before.clone()).is_ok());
        assert_eq!(ed.house, before);
        ed.selected = Some(ElemRef::Wall(5));
        ed.house.walls[0].height = 3.0;
        let res = ed.record_prop_edit(before.clone());
        assert!(matches!(res, Err(EditError::NotFound(ElemRef::Wall(5), 1))), "{res:?}");
        assert_eq!(ed.house, before);
        assert!(!ed.can_undo());
    }
}
