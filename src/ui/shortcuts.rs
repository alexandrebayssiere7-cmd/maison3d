//! Raccourcis clavier de l'éditeur et de la visite.

use bevy::prelude::*;
use bevy_egui::egui;
use egui::Key;

use crate::camera::CamState;
use crate::editor::{Editor, Tool};
use crate::model::{OpeningKind, PropKind};

/// Touche appuyée cette image.
type Pressed<'a> = &'a dyn Fn(Key) -> bool;

pub(super) fn shortcuts(ctx: &egui::Context, ed: &mut Editor, cs: &CamState) {
    if ctx.wants_keyboard_input() {
        return;
    }
    let (cmd, shift) = ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
    apply_shortcuts(ed, cmd, shift, cs.axes(), &|k| ctx.input(|i| i.key_pressed(k)));
}

/// `axes` : avant et droite de la caméra, pour décaler la sélection aux flèches.
fn apply_shortcuts(ed: &mut Editor, cmd: bool, shift: bool, axes: (Vec3, Vec3), pressed: Pressed) {
    if cmd {
        command_shortcuts(ed, shift, pressed);
        return;
    }
    if ed.walk {
        if pressed(Key::Escape) || pressed(Key::Tab) {
            ed.walk = false;
        }
        return;
    }
    tool_shortcuts(ed, pressed);
    edit_shortcuts(ed, pressed);
    view_shortcuts(ed, pressed);
    arrow_shortcuts(ed, axes, pressed);
}

/// Ctrl / Cmd + touche.
fn command_shortcuts(ed: &mut Editor, shift: bool, pressed: Pressed) {
    // en visite, remplacer la maison fausserait l'index de la voiture conduite ;
    // dupliquer y créerait une copie invisible qui referme les portes
    let edit = !ed.walk;
    if edit && (pressed(Key::Z) && shift || pressed(Key::Y)) {
        ed.redo();
    } else if edit && pressed(Key::Z) {
        ed.undo();
    } else if pressed(Key::S) {
        ed.save();
    } else if edit && pressed(Key::O) {
        ed.load();
    } else if edit && pressed(Key::D) {
        ed.duplicate_selected();
    }
}

fn tool_shortcuts(ed: &mut Editor, pressed: Pressed) {
    let tools = [
        (Key::V, Tool::Select),
        (Key::M, Tool::Wall),
        (Key::P, Tool::Room),
        (Key::L, Tool::Floor),
        (Key::T, Tool::Roof),
        (Key::O, Tool::Opening(OpeningKind::Door)),
        (Key::F, Tool::Opening(OpeningKind::Window)),
        (Key::B, Tool::Opening(OpeningKind::Bay)),
        (Key::N, Tool::Prop(PropKind::Table)),
        (Key::J, Tool::Pool),
        (Key::U, Tool::Road),
        (Key::K, Tool::Terrain),
        (Key::G, Tool::Paint),
        (Key::I, Tool::Eyedropper),
        (Key::X, Tool::Erase),
    ];
    for (k, t) in tools {
        if pressed(k) {
            // « N » garde le meuble déjà choisi
            if matches!(t, Tool::Prop(_)) && matches!(ed.tool, Tool::Prop(_)) {
                continue;
            }
            ed.set_tool(t);
        }
    }
    if pressed(Key::Escape) {
        ed.cancel();
        ed.selected = None;
        ed.walk = false;
        if ed.tool == Tool::Visitor {
            ed.tool = Tool::Select;
        }
    }
}

fn edit_shortcuts(ed: &mut Editor, pressed: Pressed) {
    if pressed(Key::Delete) || pressed(Key::Backspace) {
        ed.delete_selected();
    }
    if pressed(Key::R) {
        if matches!(ed.tool, Tool::Prop(_)) {
            ed.prop_rot = (ed.prop_rot + 45.0) % 360.0;
        } else {
            ed.rotate_selected();
        }
    }
}

fn view_shortcuts(ed: &mut Editor, pressed: Pressed) {
    if pressed(Key::PageUp) || pressed(Key::Plus) || pressed(Key::Equals) {
        ed.set_level(ed.level + 1);
    }
    if pressed(Key::PageDown) || pressed(Key::Minus) {
        ed.set_level(ed.level - 1);
    }
    if pressed(Key::H) {
        ed.hide_upper = !ed.hide_upper;
        ed.dirty = true;
    }
    if pressed(Key::F1) {
        ed.show_help = !ed.show_help;
    }
    if pressed(Key::Tab) {
        let t = if ed.tool == Tool::Visitor { Tool::Select } else { Tool::Visitor };
        ed.set_tool(t);
    }
}

/// Flèches : décalage d'un pas de grille, relatif à l'orientation de la caméra.
fn arrow_shortcuts(ed: &mut Editor, (fwd, right): (Vec3, Vec3), pressed: Pressed) {
    let arrows =
        [(Key::ArrowUp, Vec2::Y), (Key::ArrowDown, -Vec2::Y), (Key::ArrowLeft, -Vec2::X), (Key::ArrowRight, Vec2::X)];
    for (k, v) in arrows {
        if pressed(k) && ed.selected.is_some() {
            let w = right * v.x + fwd * v.y;
            let axis = if w.x.abs() > w.z.abs() { Vec2::new(w.x.signum(), 0.0) } else { Vec2::new(0.0, w.z.signum()) };
            ed.nudge_selected(axis * ed.grid);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ElemRef, House};

    /// Appuie sur `keys` (avec ou sans Ctrl / Maj), caméra regardant vers -z.
    fn press(ed: &mut Editor, cmd: bool, shift: bool, keys: &[Key]) {
        apply_shortcuts(ed, cmd, shift, (Vec3::NEG_Z, Vec3::X), &|k| keys.contains(&k));
    }

    fn with_lamp() -> Editor {
        let mut ed = Editor::default();
        ed.apply([crate::command::Command::PlaceProp {
            kind: PropKind::Lamp,
            pos: Vec2::ZERO,
            level: 0,
            rot: 0.0,
            scale: None,
            tag: None,
        }])
        .unwrap();
        ed.selected = Some(ElemRef::Prop(0));
        ed
    }

    #[test]
    fn ctrl_shortcuts_undo_redo_duplicate_and_save() {
        let mut ed = with_lamp();
        press(&mut ed, true, false, &[Key::D]);
        assert_eq!(ed.house.props.len(), 2);
        press(&mut ed, true, false, &[Key::Z]);
        assert_eq!(ed.house.props.len(), 1);
        press(&mut ed, true, true, &[Key::Z]);
        assert_eq!(ed.house.props.len(), 2);
        let dir = std::env::temp_dir().join("maison3d_shortcuts");
        ed.file = dir.join("m.json").to_string_lossy().into_owned();
        press(&mut ed, true, false, &[Key::S]);
        assert!(ed.message.starts_with("Enregistré"), "{}", ed.message);
        ed.house = House::default();
        press(&mut ed, true, false, &[Key::O]);
        assert_eq!(ed.house.props.len(), 2);
        let opened = ed.message.clone();
        ed.walk = true;
        press(&mut ed, true, false, &[Key::Y]);
        assert_eq!(ed.message, opened, "pas de rétablissement en visite");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn ctrl_d_does_nothing_during_the_visit() {
        let mut ed = with_lamp();
        ed.walk = true;
        press(&mut ed, true, false, &[Key::D]);
        assert_eq!(ed.house.props.len(), 1, "pas de copie en visite");
        ed.walk = false;
        press(&mut ed, true, false, &[Key::Z]);
        assert_eq!(ed.house.props, [], "un seul annuler revient avant la lampe : aucun point ajouté");
    }

    #[test]
    fn letters_pick_tools_and_escape_leaves_the_visit() {
        let mut ed = Editor::default();
        press(&mut ed, false, false, &[Key::M]);
        assert_eq!(ed.tool, Tool::Wall);
        ed.set_tool(Tool::Prop(PropKind::Bed));
        press(&mut ed, false, false, &[Key::N]);
        assert_eq!(ed.tool, Tool::Prop(PropKind::Bed), "N garde le meuble choisi");
        press(&mut ed, false, false, &[Key::Tab]);
        assert_eq!(ed.tool, Tool::Visitor);
        press(&mut ed, false, false, &[Key::Tab]);
        assert_eq!(ed.tool, Tool::Select);
        ed.tool = Tool::Visitor;
        press(&mut ed, false, false, &[Key::Escape]);
        assert_eq!(ed.tool, Tool::Select);
        ed.walk = true;
        press(&mut ed, false, false, &[Key::M]);
        assert_eq!(ed.tool, Tool::Select, "en visite les lettres ne changent pas d outil");
        press(&mut ed, false, false, &[Key::Tab]);
        assert!(!ed.walk);
    }

    #[test]
    fn edit_and_view_keys() {
        let mut ed = with_lamp();
        press(&mut ed, false, false, &[Key::R]);
        assert_eq!(ed.house.props[0].rot, 45.0);
        press(&mut ed, false, false, &[Key::ArrowUp, Key::ArrowRight]);
        assert_eq!(ed.house.props[0].pos, Vec2::new(0.5, -0.5));
        press(&mut ed, false, false, &[Key::Delete]);
        assert_eq!(ed.house.props, []);
        ed.set_tool(Tool::Prop(PropKind::Lamp));
        press(&mut ed, false, false, &[Key::R]);
        assert_eq!(ed.prop_rot, 45.0);
        press(&mut ed, false, false, &[Key::PageUp]);
        assert_eq!(ed.level, 1);
        press(&mut ed, false, false, &[Key::Minus]);
        assert_eq!(ed.level, 0);
        let (hide, help) = (ed.hide_upper, ed.show_help);
        press(&mut ed, false, false, &[Key::H, Key::F1]);
        assert_eq!((ed.hide_upper, ed.show_help), (!hide, !help));
    }
}
