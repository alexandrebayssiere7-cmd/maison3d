//! Interface egui : panneau d'outils, propriétés de la sélection, raccourcis clavier, aide.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui::{Color32, RichText, Slider};

use crate::camera::CamState;
use crate::editor::{Editor, Tool};
use crate::model::*;
use crate::render::Library;
use crate::visit::Visitor;

/// Vignettes des textures pour egui : (id, nom), dans l'ordre du catalogue.
type Thumbs = [(egui::TextureId, &'static str)];

const ACCENT: Color32 = Color32::from_rgb(255, 200, 60);

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
    let thumbs = &thumbs[..];
    let ctx = egui.ctx_mut();
    let ed = &mut *ed;
    release_button_focus(ctx);
    shortcuts(ctx, ed, &cs);

    if ed.walk {
        visit_overlay(ctx, &visitor);
    } else {
    egui::SidePanel::left("outils").resizable(false).exact_width(285.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
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
        });
    });
    }

    // en visite : seulement le viseur et ses consignes
    if ed.walk {
        return;
    }
    egui::TopBottomPanel::bottom("statut").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(ed.tool.hint()).color(ACCENT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(c) = ed.cursor {
                    ui.monospace(format!("x {:6.2}  z {:6.2}", c.x, c.y));
                }
                ui.label(&ed.message);
            });
        });
    });

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

/// egui donne le focus clavier à tout bouton cliqué : ZQSD et les raccourcis seraient alors bloqués,
/// et Espace « recliquerait » le bouton. On ne garde le focus que pour les champs de saisie de texte.
fn release_button_focus(ctx: &egui::Context) {
    if let Some(id) = ctx.memory(|m| m.focused()) {
        if egui::text_edit::TextEditState::load(ctx, id).is_none() {
            ctx.memory_mut(|m| m.surrender_focus(id));
        }
    }
}

fn shortcuts(ctx: &egui::Context, ed: &mut Editor, cs: &CamState) {
    if ctx.wants_keyboard_input() {
        return;
    }
    use egui::Key;
    let (cmd, shift) = ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
    let pressed = |k: Key| ctx.input(|i| i.key_pressed(k));
    if cmd {
        if pressed(Key::Z) && shift || pressed(Key::Y) {
            ed.redo();
        } else if pressed(Key::Z) {
            ed.undo();
        } else if pressed(Key::S) {
            ed.save();
        } else if pressed(Key::O) {
            ed.load();
        } else if pressed(Key::D) {
            ed.duplicate_selected();
        }
        return;
    }
    if ed.walk {
        if pressed(Key::Escape) || pressed(Key::Tab) {
            ed.walk = false;
        }
        return;
    }
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
        if ed.walk {
            ed.walk = false;
        } else if ed.tool == Tool::Visitor {
            ed.set_tool(Tool::Select);
        } else {
            ed.set_tool(Tool::Visitor);
        }
    }
    // flèches : décalage d'un pas de grille, relatif à l'orientation de la caméra
    let arrows = [(Key::ArrowUp, Vec2::Y), (Key::ArrowDown, -Vec2::Y), (Key::ArrowLeft, -Vec2::X), (Key::ArrowRight, Vec2::X)];
    for (k, v) in arrows {
        if pressed(k) && ed.selected.is_some() {
            let (fwd, right) = cs.axes();
            let w = right * v.x + fwd * v.y;
            let axis = if w.x.abs() > w.z.abs() { Vec2::new(w.x.signum(), 0.0) } else { Vec2::new(0.0, w.z.signum()) };
            ed.nudge_selected(axis * ed.grid);
        }
    }
}

fn file_section(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.horizontal(|ui| {
        if ui.button("📄 Nouveau").clicked() {
            ed.replace_house(House::default(), "Nouveau projet (Ctrl+Z pour revenir)".into());
        }
        if ui.button("✨ Exemple").on_hover_text("Charge une maison d'exemple").clicked() {
            ed.replace_house(demo_house(), "Maison d'exemple chargée".into());
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
        if ui.add_enabled(ed.can_redo(), egui::Button::new("⟳ Rétablir")).on_hover_text("Ctrl+Y / Ctrl+Maj+Z").clicked() {
            ed.redo();
        }
        if ui.button("❓").on_hover_text("Aide (F1)").clicked() {
            ed.show_help = !ed.show_help;
        }
    });
}

fn level_name(l: i32) -> String {
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
    ui.small(format!("Surface de sol de l'étage : {:.1} m²", ed.house.floor_area(ed.level)));
}

fn tool_button(ui: &mut egui::Ui, ed: &mut Editor, tool: Tool, label: &str, key: &str) {
    let selected = match (ed.tool, tool) {
        (Tool::Prop(_), Tool::Prop(_)) => true,
        (a, b) => a == b,
    };
    let text = RichText::new(label).size(14.0);
    let resp = ui.add_sized([128.0, 26.0], egui::Button::new(text).selected(selected)).on_hover_text(format!("Raccourci : {key}"));
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
            (Tool::Prop(PropKind::Table), "Mobilier", "N"),
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

fn color_row(ui: &mut egui::Ui, label: &str, c: &mut Rgb) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_rgb(c).changed()
    })
    .inner
}

/// Liste déroulante de vignettes ; renvoie vrai si le choix a changé.
fn texture_picker(ui: &mut egui::Ui, salt: &str, value: &mut usize, thumbs: &Thumbs) -> bool {
    let before = *value;
    let name = value.checked_sub(1).and_then(|i| thumbs.get(i)).map_or("Aucune (couleur unie)", |t| t.1);
    ui.horizontal(|ui| {
        ui.label("Texture");
        egui::ComboBox::from_id_salt(salt).selected_text(name).width(170.0).height(420.0).show_ui(ui, |ui| {
            ui.selectable_value(value, 0, "Aucune (couleur unie)");
            egui::Grid::new((salt, "grille")).spacing([3.0, 3.0]).show(ui, |ui| {
                for (i, (id, name)) in thumbs.iter().enumerate() {
                    let img = egui::Image::new(egui::load::SizedTexture::new(*id, egui::vec2(46.0, 46.0)));
                    if ui.add(egui::Button::image(img).selected(*value == i + 1)).on_hover_text(*name).clicked() {
                        *value = i + 1;
                    }
                    if i % 5 == 4 {
                        ui.end_row();
                    }
                }
            });
        });
    });
    *value != before
}

/// Couleur + texture ; choisir une texture repasse la teinte en blanc pour voir ses vraies couleurs.
fn finish_row(ui: &mut egui::Ui, salt: &str, color: &mut Rgb, texture: &mut usize, thumbs: &Thumbs) {
    if texture_picker(ui, salt, texture, thumbs) && *texture != 0 {
        *color = WHITE_TEX;
    }
    color_row(ui, if *texture == 0 { "Couleur" } else { "Teinte" }, color);
}

fn tool_settings(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    ui.label(RichText::new("Réglages de l'outil").strong());
    match ed.tool {
        Tool::Wall | Tool::Room => {
            ui.add(Slider::new(&mut ed.wall_height, 0.5..=10.0).text("hauteur").suffix(" m"));
            ui.add(Slider::new(&mut ed.wall_thick, 0.05..=1.0).text("épaisseur").suffix(" m"));
            ui.label("Murs :");
            finish_row(ui, "t_mur", &mut ed.wall_color, &mut ed.wall_texture, thumbs);
            if ed.tool == Tool::Room {
                ui.label("Sol :");
                finish_row(ui, "t_sol", &mut ed.floor_color, &mut ed.floor_texture, thumbs);
            }
        }
        Tool::Floor => {
            finish_row(ui, "t_sol", &mut ed.floor_color, &mut ed.floor_texture, thumbs);
        }
        Tool::Roof => {
            egui::ComboBox::from_label("forme")
                .selected_text(ed.roof_kind.name())
                .show_ui(ui, |ui| {
                    for k in RoofKind::ALL {
                        ui.selectable_value(&mut ed.roof_kind, k, k.name());
                    }
                });
            if ed.roof_kind != RoofKind::Flat {
                ui.add(Slider::new(&mut ed.roof_height, 0.3..=8.0).text("hauteur").suffix(" m"));
            }
            ui.add(Slider::new(&mut ed.roof_overhang, 0.0..=2.0).text("débord").suffix(" m"));
            finish_row(ui, "t_toit", &mut ed.roof_color, &mut ed.roof_texture, thumbs);
        }
        Tool::Opening(k) => {
            let (w, h, s) = &mut ed.op_dims[k as usize];
            ui.add(Slider::new(w, 0.3..=6.0).text("largeur").suffix(" m"));
            ui.add(Slider::new(h, 0.3..=4.0).text("hauteur").suffix(" m"));
            if k == OpeningKind::Window {
                ui.add(Slider::new(s, 0.0..=2.5).text("allège").suffix(" m"));
            }
            color_row(ui, "Couleur", &mut ed.op_colors[k as usize]);
            if ui.button("Valeurs par défaut").clicked() {
                ed.op_dims[k as usize] = k.default_dims();
            }
        }
        Tool::Prop(current) => {
            egui::Grid::new("meubles").num_columns(3).spacing([4.0, 4.0]).show(ui, |ui| {
                for (i, k) in PropKind::ALL.into_iter().enumerate() {
                    if ui.add_sized([84.0, 22.0], egui::Button::new(k.name()).selected(k == current)).clicked() {
                        ed.set_tool(Tool::Prop(k));
                    }
                    if i % 3 == 2 {
                        ui.end_row();
                    }
                }
            });
            ui.add(Slider::new(&mut ed.prop_rot, 0.0..=359.0).text("rotation (R)").suffix("°"));
            finish_row(ui, "t_meuble", &mut ed.prop_color, &mut ed.prop_texture, thumbs);
        }
        Tool::Paint | Tool::Eyedropper => {
            ui.small("Le clic applique la couleur ET la texture ci-dessous.");
            finish_row(ui, "t_peinture", &mut ed.paint_color, &mut ed.paint_texture, thumbs);
            ui.horizontal_wrapped(|ui| {
                let palette: [Rgb; 12] = [
                    [0.95, 0.95, 0.93], [0.93, 0.89, 0.8], [0.85, 0.75, 0.6], [0.62, 0.48, 0.34],
                    [0.45, 0.3, 0.2], [0.65, 0.25, 0.18], [0.75, 0.2, 0.2], [0.95, 0.75, 0.3],
                    [0.4, 0.6, 0.35], [0.3, 0.45, 0.7], [0.5, 0.5, 0.52], [0.18, 0.18, 0.2],
                ];
                for c in palette {
                    let col = Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
                    ui.painter().rect_filled(rect, 3.0, col);
                    if resp.clicked() {
                        ed.paint_color = c;
                    }
                }
            });
        }
        Tool::Select => {
            ui.small("Cliquez sur un élément pour voir et modifier ses propriétés.");
        }
        Tool::Erase => {
            ui.small("Survolez : l'élément à supprimer s'affiche en rouge.");
        }
        Tool::Visitor => {
            ui.small("Cliquez dans la scène pour poser le bonhomme. Il tombe sur le sol le plus proche, se cogne aux murs, monte les escaliers.");
        }
    }
    ui.horizontal(|ui| {
        ui.label("Grille");
        for (v, label) in [(1.0, "1 m"), (0.5, "50 cm"), (0.25, "25 cm"), (0.1, "10 cm")] {
            ui.selectable_value(&mut ed.grid, v, label);
        }
    });
}

fn selection_section(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    let Some(sel) = ed.selected else { return };
    let before = ed.house.clone();
    let h = &mut ed.house;
    let title = match sel {
        ElemRef::Wall(_) => "Mur".to_string(),
        ElemRef::Opening(w, o) => h.walls.get(w).and_then(|w| w.openings.get(o)).map_or("?", |o| o.kind.name()).to_string(),
        ElemRef::Slab(_) => "Sol".into(),
        ElemRef::Roof(_) => "Toit".into(),
        ElemRef::Prop(i) => h.props.get(i).map_or("?", |p| p.kind.name()).to_string(),
    };
    ui.label(RichText::new(format!("Sélection : {title}")).strong().color(ACCENT));
    match sel {
        ElemRef::Wall(i) => {
            let Some(w) = h.walls.get_mut(i) else { return };
            ui.label(format!("Longueur : {:.2} m  •  {} ouverture(s)", w.len(), w.openings.len()));
            ui.add(Slider::new(&mut w.height, 0.5..=10.0).text("hauteur").suffix(" m"));
            ui.add(Slider::new(&mut w.thick, 0.05..=1.0).text("épaisseur").suffix(" m"));
            finish_row(ui, "s_mur", &mut w.color, &mut w.texture, thumbs);
        }
        ElemRef::Opening(wi, oi) => {
            let Some(w) = h.walls.get_mut(wi) else { return };
            let (len, wall_h) = (w.len(), w.height);
            let Some(o) = w.openings.get_mut(oi) else { return };
            ui.add(Slider::new(&mut o.width, 0.3..=len.max(0.4)).text("largeur").suffix(" m"));
            ui.add(Slider::new(&mut o.height, 0.3..=wall_h).text("hauteur").suffix(" m"));
            ui.add(Slider::new(&mut o.sill, 0.0..=wall_h - 0.3).text("allège").suffix(" m"));
            ui.add(Slider::new(&mut o.t, 0.0..=len).text("position").suffix(" m"));
            color_row(ui, "Couleur", &mut o.color);
            // garde l'ouverture dans le mur
            o.width = o.width.min(len - 0.1);
            o.t = o.t.clamp(o.width / 2.0 + 0.05, len - o.width / 2.0 - 0.05);
        }
        ElemRef::Slab(i) => {
            let Some(s) = h.slabs.get_mut(i) else { return };
            let size = s.max - s.min;
            ui.label(format!("{:.2} × {:.2} m  =  {:.1} m²", size.x, size.y, size.x * size.y));
            finish_row(ui, "s_sol", &mut s.color, &mut s.texture, thumbs);
        }
        ElemRef::Roof(i) => {
            let Some(r) = h.roofs.get_mut(i) else { return };
            egui::ComboBox::from_label("forme ")
                .selected_text(r.kind.name())
                .show_ui(ui, |ui| {
                    for k in RoofKind::ALL {
                        ui.selectable_value(&mut r.kind, k, k.name());
                    }
                });
            if r.kind != RoofKind::Flat {
                ui.add(Slider::new(&mut r.height, 0.3..=8.0).text("hauteur").suffix(" m"));
            }
            ui.add(Slider::new(&mut r.overhang, 0.0..=2.0).text("débord").suffix(" m"));
            finish_row(ui, "s_toit", &mut r.color, &mut r.texture, thumbs);
        }
        ElemRef::Prop(i) => {
            let Some(p) = h.props.get_mut(i) else { return };
            ui.add(Slider::new(&mut p.rot, 0.0..=359.0).text("rotation").suffix("°"));
            finish_row(ui, "s_meuble", &mut p.color, &mut p.texture, thumbs);
            if ui.button("Couleur d'origine").clicked() {
                p.color = p.kind.default_color();
            }
        }
    }
    if ed.house != before {
        if !ed.prop_edit_active {
            ed.push_undo(before);
            ed.prop_edit_active = true;
        }
        ed.dirty = true;
    }
    ui.horizontal(|ui| {
        if !matches!(sel, ElemRef::Opening(..)) && ui.button("⧉ Dupliquer").on_hover_text("Ctrl+D").clicked() {
            ed.duplicate_selected();
        }
        if matches!(sel, ElemRef::Prop(_)) && ui.button("⟳ 45°").on_hover_text("R").clicked() {
            ed.rotate_selected();
        }
        if ui.button(RichText::new("🗑 Supprimer").color(Color32::from_rgb(255, 110, 110))).on_hover_text("Suppr").clicked() {
            ed.delete_selected();
        }
    });
}

fn view_section(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.label(RichText::new("Vue").strong());
    if ui.button("🚶 Poser le bonhomme et visiter (Tab)").clicked() {
        ed.set_tool(Tool::Visitor);
    }
    let h = ed.hour;
    ui.add(Slider::new(&mut ed.hour, 0.0..=24.0).text(format!("heure ({:02}h{:02})", h as u32, ((h.fract()) * 60.0) as u32)));
    ui.checkbox(&mut ed.show_grid, "Afficher la grille");
}

/// Viseur, consignes et action possible pendant la visite.
fn visit_overlay(ctx: &egui::Context, v: &Visitor) {
    let center = ctx.screen_rect().center();
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("viseur")));
    let aim = if v.target.is_some() { ACCENT } else { Color32::from_white_alpha(200) };
    painter.circle_stroke(center, 5.0, egui::Stroke::new(2.0_f32, aim));
    // actions liées à l'objet visé en évidence, les autres en discret
    for (i, a) in v.actions.iter().enumerate() {
        let strong = a.starts_with("E :") || a.contains("relever");
        let color = if strong { ACCENT } else { Color32::from_white_alpha(170) };
        let size = if strong { 18.0 } else { 14.0 };
        let pos = center + egui::vec2(0.0, 30.0 + 22.0 * i as f32);
        painter.text(pos + egui::vec2(1.0, 1.0), egui::Align2::CENTER_CENTER, *a, egui::FontId::proportional(size), Color32::BLACK);
        painter.text(pos, egui::Align2::CENTER_CENTER, *a, egui::FontId::proportional(size), color);
    }
    egui::Area::new(egui::Id::new("visite"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 10.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label(RichText::new("Visite").strong().color(ACCENT));
                ui.label("Souris : regarder  •  ZQSD / WASD : marcher  •  Maj : courir  •  Espace : sauter");
                ui.label("E : ouvrir / s'asseoir / s'allonger  •  C : s'asseoir  •  X : s'allonger  •  V : 1re / 3e personne");
                ui.label("Échap ou Tab : revenir à l'édition");
            });
        });
}

fn help(ui: &mut egui::Ui) {
    let section = |ui: &mut egui::Ui, title: &str, lines: &[&str]| {
        ui.label(RichText::new(title).strong().color(ACCENT));
        for l in lines {
            ui.label(format!("  • {l}"));
        }
        ui.add_space(4.0);
    };
    section(ui, "Démarrer en 30 secondes", &[
        "Outil Pièce (P) : glissez un rectangle (ou cliquez 2 coins) → 4 murs + un sol",
        "Outil Mur (M) : glissez pour un mur, ou cliquez coin après coin",
        "Porte (O) / Fenêtre (F) : survolez un mur, cliquez",
        "Montez d'un étage (+), outil Toit (T), glissez au-dessus des murs",
        "Mobilier (N) : choisissez un meuble, cliquez pour le poser",
        "Tab : posez le bonhomme où vous voulez et visitez (E ouvre portes et placards)",
        "Peinture (G) : choisissez une couleur et/ou une des 55 textures",
    ]);
    section(ui, "Caméra", &[
        "ZQSD / WASD : avancer vers là où vous regardez  •  Espace / C : monter / descendre",
        "A/E (Q/E en QWERTY) : pivoter  •  Maj : aller plus vite",
        "Flèches (sans sélection) : pivoter / incliner",
        "Clic droit + glisser, ou Alt (⌥) + clic gauche + glisser : tourner",
        "Trackpad : glisser à deux doigts = tourner, pincer = zoom",
        "Souris : molette = zoom, clic molette ou Maj + clic droit = déplacer la vue",
    ]);
    section(ui, "Édition", &[
        "Ctrl+Z / Ctrl+Y : annuler / rétablir",
        "Sélection (V) : clic pour choisir, glisser pour déplacer, flèches pour décaler",
        "Suppr : effacer  •  Ctrl+D : dupliquer  •  R : pivoter un meuble",
        "Échap ou clic droit : annuler le tracé en cours",
        "Maj enfoncée : placement libre (sans grille)",
        "Les extrémités de murs s'aimantent entre elles",
    ]);
    section(ui, "Étages et fichiers", &[
        "Page↑ / Page↓ (ou + / -) : changer d'étage  •  H : masquer les étages au-dessus",
        "Ctrl+S / Ctrl+O : enregistrer / ouvrir (fichier JSON)",
    ]);
}
