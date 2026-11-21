//! Interface egui : panneau d'outils, propriétés de la sélection, raccourcis clavier, aide.

mod overlay;
mod settings;
mod shortcuts;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui::{Color32, RichText, Slider};
use overlay::{help, visit_overlay};
use settings::{selection_section, tool_settings};
use shortcuts::shortcuts;

use crate::camera::CamState;
use crate::command::Command;
use crate::editor::{Editor, Tool};
use crate::menu::{start_world, worlds_dir};
use crate::model::{House, OpeningKind, PropKind, demo_house};
use crate::render::Library;
use crate::visit::Visitor;

/// Vignettes des textures pour egui : (id, nom), dans l'ordre du catalogue.
type Thumbs = [(egui::TextureId, &'static str)];

pub const ACCENT: Color32 = Color32::from_rgb(255, 200, 60);

/// Cadre commun à tous les éléments du HUD.
pub fn hud_frame() -> egui::Frame {
    egui::Frame::new().fill(Color32::from_black_alpha(170)).corner_radius(10.0).inner_margin(egui::vec2(14.0, 10.0))
}

pub fn ui_system(
    mut egui: EguiContexts,
    mut ed: ResMut<Editor>,
    cs: Res<CamState>,
    lib: Res<Library>,
    visitor: Res<Visitor>,
    mut thumbs: Local<Vec<(egui::TextureId, &'static str)>>,
) {
    if thumbs.is_empty() {
        *thumbs = lib.textures.iter().map(|(name, img, _)| (egui.add_image(img.clone()), *name)).collect();
    }
    let ctx = egui.ctx_mut();
    let ed = &mut *ed;
    release_button_focus(ctx);
    shortcuts(ctx, ed, &cs);
    // en visite : seulement le viseur et ses consignes
    if ed.walk {
        visit_overlay(ctx, &visitor);
        return;
    }
    editor_panels(ctx, ed, &thumbs);
}

/// Panneau d'outils, barre d'état, cote sous le pointeur et aide.
fn editor_panels(ctx: &egui::Context, ed: &mut Editor, thumbs: &Thumbs) {
    egui::SidePanel::left("outils").resizable(false).exact_width(285.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| side_panel(ui, ed, thumbs));
    });
    status_bar(ctx, ed);
    if let (Some(m), Some(pos)) = (&ed.measure, ctx.pointer_hover_pos()) {
        egui::Area::new(egui::Id::new("cote"))
            .fixed_pos(pos + egui::vec2(18.0, 14.0))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(RichText::new(m).strong()));
            });
    }

    let mut open = ed.show_help;
    egui::Window::new("Aide — prise en main").open(&mut open).default_width(430.0).show(ctx, help);
    ed.show_help = open;

    release_button_focus(ctx);
    // fin d'un geste d'édition de propriété : la prochaine modification crée un nouveau point d'annulation
    if !ctx.input(|i| i.pointer.any_down()) && !ctx.wants_keyboard_input() {
        ed.prop_edit_active = false;
    }
}

fn side_panel(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    ui.add_space(6.0);
    ui.heading(RichText::new("🏠 Maison 3D").strong());
    ui.add_space(4.0);
    file_section(ui, ed);
    ui.separator();
    level_section(ui, ed);
    ui.separator();
    tools_section(ui, ed);
    ui.separator();
    tool_settings(ui, ed, thumbs);
    if ed.selected.is_some() {
        ui.separator();
        selection_section(ui, ed, thumbs);
    }
    ui.separator();
    view_section(ui, ed);
}

/// Consigne de l'outil, message et coordonnées du curseur.
fn status_bar(ctx: &egui::Context, ed: &Editor) {
    // décalé de la moitié du panneau d'outils pour rester centré sur la vue 3D
    egui::Area::new(egui::Id::new("statut"))
        .anchor(egui::Align2::CENTER_BOTTOM, [142.0, -14.0])
        .interactable(false)
        .show(ctx, |ui| {
            hud_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(ed.tool.hint()).color(ACCENT));
                    if !ed.message.is_empty() {
                        ui.label(RichText::new("·").color(Color32::GRAY));
                        ui.label(RichText::new(&ed.message).color(Color32::from_gray(220)));
                    }
                    if let Some(c) = ed.cursor {
                        ui.label(
                            RichText::new(format!("x {:6.2}  z {:6.2}", c.x, c.y)).monospace().color(Color32::GRAY),
                        );
                    }
                });
            });
        });
}

/// egui donne le focus clavier à tout bouton cliqué : ZQSD et les raccourcis seraient alors bloqués,
/// et Espace « recliquerait » le bouton. On ne garde le focus que pour les champs de saisie de texte.
fn release_button_focus(ctx: &egui::Context) {
    if let Some(id) = ctx.memory(bevy_egui::egui::Memory::focused)
        && egui::text_edit::TextEditState::load(ctx, id).is_none()
    {
        ctx.memory_mut(|m| m.surrender_focus(id));
    }
}

fn file_section(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.horizontal(|ui| {
        if ui.button("📄 Nouveau").clicked() {
            start_world(ed, House::default(), &worlds_dir(), "Nouveau monde");
        }
        if ui.button("✨ Exemple").on_hover_text("Charge une maison d'exemple").clicked() {
            start_world(ed, demo_house(), &worlds_dir(), "Maison d'exemple");
        }
    });
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ed.file).desired_width(120.0));
        if ui.button("📂 Ouvrir").on_hover_text("Ctrl+O").clicked() {
            ed.load();
        }
        if ui.button("💾").on_hover_text("Enregistrer (Ctrl+S)").clicked() {
            ed.save();
        }
    });
    ui.horizontal(|ui| {
        if ui.add_enabled(ed.can_undo(), egui::Button::new("⟲ Annuler")).on_hover_text("Ctrl+Z").clicked() {
            ed.undo();
        }
        if ui.add_enabled(ed.can_redo(), egui::Button::new("⟳ Rétablir")).on_hover_text("Ctrl+Y / Ctrl+Maj+Z").clicked()
        {
            ed.redo();
        }
        if ui.button("❓").on_hover_text("Aide (F1)").clicked() {
            ed.show_help = !ed.show_help;
        }
    });
}

pub fn level_name(l: i32) -> String {
    match l {
        -1 => "Sous-sol".into(),
        0 => "Rez-de-chaussée".into(),
        n => format!("Étage {n}"),
    }
}

fn level_section(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.horizontal(|ui| {
        if ui.button(" − ").on_hover_text("Étage inférieur (Page↓ ou -)").clicked() {
            ed.set_level(ed.level - 1);
        }
        ui.label(RichText::new(level_name(ed.level)).strong().size(15.0));
        if ui.button(" + ").on_hover_text("Étage supérieur (Page↑ ou +)").clicked() {
            ed.set_level(ed.level + 1);
        }
    });
    if ui.checkbox(&mut ed.hide_upper, "Masquer les étages au-dessus (H)").changed() {
        ed.dirty = true;
    }
    if ui
        .button("+ Ajouter un étage au-dessus")
        .on_hover_text("Copie les murs (sans les portes) et les sols de cet étage, remonte le toit")
        .clicked()
        && ed.apply([Command::AddStorey { level: ed.level, min: None, max: None }]).is_ok()
    {
        ed.set_level(ed.level + 1);
        ed.message = "Étage ajouté (Ctrl+Z pour revenir)".into();
    }
    ui.small(format!("Surface de sol de l'étage : {:.1} m²", ed.house.floor_area(ed.level)));
}

fn tool_button(ui: &mut egui::Ui, ed: &mut Editor, tool: Tool, label: &str, key: &str) {
    let selected = match (ed.tool, tool) {
        (Tool::Prop(_), Tool::Prop(_)) => true,
        (a, b) => a == b,
    };
    let text = RichText::new(label).size(14.0);
    let mut resp = ui.add_sized([128.0, 26.0], egui::Button::new(text).selected(selected));
    if !key.is_empty() {
        resp = resp.on_hover_text(format!("Raccourci : {key}"));
    }
    if resp.clicked() {
        let t = match (tool, ed.tool) {
            (Tool::Prop(_), Tool::Prop(k)) => Tool::Prop(k),
            _ => tool,
        };
        ed.set_tool(t);
    }
}

fn tools_section(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.label(RichText::new("Outils").strong());
    egui::Grid::new("outils_grid").num_columns(2).spacing([6.0, 4.0]).show(ui, |ui| {
        let tools = [
            (Tool::Select, "👆 Sélection", "V"),
            (Tool::Wall, "Mur", "M"),
            (Tool::Room, "⬜ Pièce", "P"),
            (Tool::Floor, "Sol", "L"),
            (Tool::Roof, "🔺 Toit", "T"),
            (Tool::Opening(OpeningKind::Door), "🚪 Porte", "O"),
            (Tool::Opening(OpeningKind::Window), "▣ Fenêtre", "F"),
            (Tool::Opening(OpeningKind::Bay), "Baie vitrée", "B"),
            (Tool::Prop(PropKind::Table), "📚 Bibliothèque", "N"),
            (Tool::Pool, "Piscine", "J"),
            (Tool::Road, "Route", "U"),
            (Tool::Terrain, "Relief", "K"),
            (Tool::Paint, "🎨 Peinture", "G"),
            (Tool::Eyedropper, "💧 Pipette", "I"),
            (Tool::Erase, "✖ Gomme", "X"),
            (Tool::Visitor, "🚶 Visite", "Tab"),
        ];
        for (i, (t, label, key)) in tools.into_iter().enumerate() {
            tool_button(ui, ed, t, label, key);
            if i % 2 == 1 {
                ui.end_row();
            }
        }
    });
}

fn view_section(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.label(RichText::new("Vue").strong());
    if ui.button("🚶 Poser le bonhomme et visiter (Tab)").clicked() {
        ed.set_tool(Tool::Visitor);
    }
    let h = ed.hour;
    ui.add(Slider::new(&mut ed.hour, 0.0..=24.0).text(format!(
        "heure ({:02}h{:02})",
        h as u32,
        ((h.fract()) * 60.0) as u32
    )));
    ui.checkbox(&mut ed.show_grid, "Afficher la grille");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ElemRef;

    /// Une image egui sans fenêtre ni rendu.
    fn frame(mut f: impl FnMut(&egui::Context)) {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))),
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| f(ctx));
    }

    const THUMBS: [(egui::TextureId, &str); 6] = [(egui::TextureId::Managed(0), "Brique"); 6];

    #[test]
    fn panels_draw_for_every_tool() {
        let mut ed = Editor::default();
        ed.measure = Some("1 m".into());
        ed.message = "ok".into();
        ed.cursor = Some(Vec2::ONE);
        let tools = [
            Tool::Select,
            Tool::Wall,
            Tool::Room,
            Tool::Floor,
            Tool::Roof,
            Tool::Opening(OpeningKind::Window),
            Tool::Opening(OpeningKind::Door),
            Tool::Prop(PropKind::Stairs),
            Tool::Pool,
            Tool::Road,
            Tool::Terrain,
            Tool::Paint,
            Tool::Eyedropper,
            Tool::Erase,
            Tool::Visitor,
        ];
        for t in tools {
            ed.set_tool(t);
            frame(|ctx| editor_panels(ctx, &mut ed, &THUMBS));
            assert_eq!(ed.tool, t);
        }
        ed.roof_kind = crate::model::RoofKind::Flat;
        ed.set_tool(Tool::Roof);
        frame(|ctx| editor_panels(ctx, &mut ed, &THUMBS));
        assert!(ed.show_help, "l aide reste ouverte");
        assert!(!ed.prop_edit_active);
    }

    #[test]
    fn panels_draw_every_kind_of_selection() {
        let mut ed = Editor::default();
        ed.house = demo_house();
        ed.house.add_storey(0, None);
        let mut targets: Vec<ElemRef> = vec![ElemRef::Wall(0), ElemRef::Opening(0, 0), ElemRef::Slab(0)];
        targets.extend([ElemRef::Roof(0), ElemRef::Prop(0), ElemRef::Pool(0), ElemRef::Road(0)]);
        let stairs = ed.house.props.iter().position(|p| p.kind.dim().is_some());
        targets.extend(stairs.map(ElemRef::Prop));
        targets.push(ElemRef::Wall(999));
        for t in targets {
            ed.selected = Some(t);
            let props = ed.house.props.len();
            frame(|ctx| editor_panels(ctx, &mut ed, &THUMBS));
            assert_eq!((ed.selected, ed.house.props.len()), (Some(t), props));
        }
    }

    /// Un éditeur avec un mur de `len` m de haut de `height` m, sélectionné.
    fn selected_wall(len: f32, height: f32) -> Editor {
        let mut ed = Editor::default();
        let (a, b, color) = (Vec2::ZERO, Vec2::new(len, 0.0), [1.0; 3]);
        ed.house.walls.push(crate::model::Wall {
            a,
            b,
            level: 0,
            height,
            thick: 0.2,
            color,
            openings: vec![],
            texture: 0,
        });
        ed.selected = Some(ElemRef::Wall(0));
        ed
    }

    #[test]
    fn selecting_keeps_values_beyond_the_sliders() {
        // un mur de 12 m de haut, accepté par la CLI, au-delà du curseur (10 m)
        let mut ed = selected_wall(4.0, 12.0);
        let before = ed.house.clone();
        frame(|ctx| editor_panels(ctx, &mut ed, &THUMBS));
        assert_eq!(ed.house, before, "sélectionner ne modifie rien");
        assert!(!ed.can_undo());
    }

    #[test]
    fn opening_as_wide_as_its_wall_does_not_crash_the_panel() {
        // l état laissé par le curseur de largeur poussé au bout
        for len in [2.0, 8.05] {
            let mut ed = selected_wall(len, 3.0);
            let mut o = crate::model::Opening::new(OpeningKind::Window, len / 2.0);
            o.width = len;
            ed.house.walls[0].openings.push(o);
            ed.selected = Some(ElemRef::Opening(0, 0));
            frame(|ctx| editor_panels(ctx, &mut ed, &THUMBS));
        }
    }

    #[test]
    fn visit_overlay_shows_speed_and_actions() {
        let mut v = Visitor::default();
        v.speed_kmh = Some(42.0);
        v.actions = vec!["E : descendre", "V : vue"];
        frame(|ctx| visit_overlay(ctx, &v));
        v.target = Some(ElemRef::Wall(0));
        v.speed_kmh = None;
        frame(|ctx| visit_overlay(ctx, &v));
        assert_eq!([level_name(-1), level_name(0), level_name(2)], ["Sous-sol", "Rez-de-chaussée", "Étage 2"]);
        frame(release_button_focus);
    }
}
