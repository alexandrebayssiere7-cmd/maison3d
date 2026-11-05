//! Réglages de l'outil courant et propriétés de l'élément sélectionné.

use bevy_egui::egui;
use egui::{Color32, RichText, Slider};

use super::{ACCENT, Thumbs};
use crate::command::Command;
use crate::editor::{Editor, Tool};
use crate::model::{ElemRef, House, OpeningKind, PropKind, Rgb, RoofKind, WHITE_TEX};
use crate::terrain::Brush;

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

pub(super) fn tool_settings(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    ui.label(RichText::new("Réglages de l'outil").strong());
    match ed.tool {
        Tool::Wall | Tool::Room => wall_settings(ui, ed, thumbs),
        Tool::Floor => finish_row(ui, "t_sol", &mut ed.floor_color, &mut ed.floor_texture, thumbs),
        Tool::Roof => roof_settings(ui, ed, thumbs),
        Tool::Opening(k) => opening_settings(ui, ed, k),
        Tool::Prop(current) => prop_settings(ui, ed, current, thumbs),
        Tool::Paint | Tool::Eyedropper => paint_settings(ui, ed, thumbs),
        Tool::Pool => {
            ui.add(Slider::new(&mut ed.pool_depth, 0.5..=3.0).text("profondeur").suffix(" m"));
            finish_row(ui, "t_piscine", &mut ed.pool_color, &mut ed.pool_texture, thumbs);
        }
        Tool::Road => {
            ui.add(Slider::new(&mut ed.road_width, 1.0..=12.0).text("largeur").suffix(" m"));
            finish_row(ui, "t_route", &mut ed.road_color, &mut ed.road_texture, thumbs);
        }
        Tool::Terrain => terrain_settings(ui, ed),
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

fn wall_settings(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    ui.add(Slider::new(&mut ed.wall_height, 0.5..=10.0).text("hauteur").suffix(" m"));
    ui.add(Slider::new(&mut ed.wall_thick, 0.05..=1.0).text("épaisseur").suffix(" m"));
    ui.label("Murs :");
    finish_row(ui, "t_mur", &mut ed.wall_color, &mut ed.wall_texture, thumbs);
    if ed.tool == Tool::Room {
        ui.label("Sol :");
        finish_row(ui, "t_sol", &mut ed.floor_color, &mut ed.floor_texture, thumbs);
    }
}

fn roof_settings(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    egui::ComboBox::from_label("forme").selected_text(ed.roof_kind.name()).show_ui(ui, |ui| {
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

fn opening_settings(ui: &mut egui::Ui, ed: &mut Editor, k: OpeningKind) {
    let (w, h, s) = &mut ed.op_dims[k as usize];
    ui.add(Slider::new(w, 0.3..=6.0).text("largeur").suffix(" m"));
    ui.add(Slider::new(h, 0.3..=4.0).text("hauteur").suffix(" m"));
    if matches!(k, OpeningKind::Window | OpeningKind::Grid | OpeningKind::Fixed) {
        ui.add(Slider::new(s, 0.0..=2.5).text("allège").suffix(" m"));
    }
    color_row(ui, "Couleur", &mut ed.op_colors[k as usize]);
    if ui.button("Valeurs par défaut").clicked() {
        ed.op_dims[k as usize] = k.default_dims();
    }
}

fn prop_settings(ui: &mut egui::Ui, ed: &mut Editor, current: PropKind, thumbs: &Thumbs) {
    for (title, outdoor) in [("Intérieur", false), ("Jardin (suit le relief)", true)] {
        ui.small(title);
        egui::Grid::new(("meubles", outdoor)).num_columns(3).spacing([4.0, 4.0]).show(ui, |ui| {
            for (i, k) in PropKind::ALL.into_iter().filter(|k| k.outdoor() == outdoor).enumerate() {
                if ui.add_sized([84.0, 22.0], egui::Button::new(k.name()).selected(k == current)).clicked() {
                    ed.set_tool(Tool::Prop(k));
                }
                if i % 3 == 2 {
                    ui.end_row();
                }
            }
        });
    }
    ui.add(Slider::new(&mut ed.prop_rot, 0.0..=359.0).text("rotation (R)").suffix("°"));
    if current.has_height() {
        ui.add(Slider::new(&mut ed.prop_height, 0.4..=8.0).text("hauteur").suffix(" m"));
    }
    finish_row(ui, "t_meuble", &mut ed.prop_color, &mut ed.prop_texture, thumbs);
}

/// Teintes proposées d'un clic.
const PALETTE: [Rgb; 12] = [
    [0.95, 0.95, 0.93],
    [0.93, 0.89, 0.8],
    [0.85, 0.75, 0.6],
    [0.62, 0.48, 0.34],
    [0.45, 0.3, 0.2],
    [0.65, 0.25, 0.18],
    [0.75, 0.2, 0.2],
    [0.95, 0.75, 0.3],
    [0.4, 0.6, 0.35],
    [0.3, 0.45, 0.7],
    [0.5, 0.5, 0.52],
    [0.18, 0.18, 0.2],
];

fn paint_settings(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    ui.small("Le clic applique la couleur ET la texture ci-dessous.");
    finish_row(ui, "t_peinture", &mut ed.paint_color, &mut ed.paint_texture, thumbs);
    ui.horizontal_wrapped(|ui| {
        for c in PALETTE {
            let col = Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(rect, 3.0, col);
            if resp.clicked() {
                ed.paint_color = c;
            }
        }
    });
}

fn terrain_settings(ui: &mut egui::Ui, ed: &mut Editor) {
    ui.horizontal_wrapped(|ui| {
        for b in Brush::ALL {
            ui.selectable_value(&mut ed.brush, b, b.name());
        }
    });
    ui.add(Slider::new(&mut ed.brush_radius, 1.0..=30.0).text("taille").suffix(" m"));
    ui.add(Slider::new(&mut ed.brush_strength, 0.2..=10.0).text("force").suffix(" m/s"));
    ui.add_space(4.0);
    ui.label(RichText::new("Relief aléatoire").strong());
    ui.add(Slider::new(&mut ed.gen_amp, 1.0..=40.0).text("altitude").suffix(" m"));
    ui.add(Slider::new(&mut ed.gen_scale, 10.0..=100.0).text("taille des reliefs").suffix(" m"));
    ui.checkbox(&mut ed.gen_mountains, "Montagnes (crêtes) plutôt que collines");
    ui.horizontal(|ui| {
        if ui.button("Générer").on_hover_text("Un nouveau relief à chaque clic ; la maison reste à plat").clicked()
        {
            let seed =
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
            // refus déjà affiché dans le message par Editor::apply
            let _ = ed.apply([Command::GenerateTerrain {
                seed,
                amp: ed.gen_amp,
                scale: ed.gen_scale,
                mountains: ed.gen_mountains,
            }]);
            ed.message = "Relief généré (Ctrl+Z pour revenir)".into();
        }
        if ui.button("Tout aplanir").clicked() {
            // refus déjà affiché dans le message par Editor::apply
            let _ = ed.apply([Command::FlattenTerrain]);
        }
    });
}

pub(super) fn selection_section(ui: &mut egui::Ui, ed: &mut Editor, thumbs: &Thumbs) {
    let Some(sel) = ed.selected else { return };
    let before = ed.house.clone();
    let title = selection_title(&ed.house, sel);
    ui.label(RichText::new(format!("Sélection : {title}")).strong().color(ACCENT));
    if !selection_props(ui, &mut ed.house, sel, thumbs) {
        return;
    }
    ed.record_prop_edit(before);
    ui.horizontal(|ui| {
        if !matches!(sel, ElemRef::Opening(..)) && ui.button("⧉ Dupliquer").on_hover_text("Ctrl+D").clicked() {
            ed.duplicate_selected();
        }
        if matches!(sel, ElemRef::Prop(_)) && ui.button("⟳ 45°").on_hover_text("R").clicked() {
            ed.rotate_selected();
        }
        if ui
            .button(RichText::new("🗑 Supprimer").color(Color32::from_rgb(255, 110, 110)))
            .on_hover_text("Suppr")
            .clicked()
        {
            ed.delete_selected();
        }
    });
}

fn selection_title(h: &House, sel: ElemRef) -> String {
    match sel {
        ElemRef::Wall(_) => "Mur".to_string(),
        ElemRef::Opening(w, o) => {
            h.walls.get(w).and_then(|w| w.openings.get(o)).map_or("?", |o| o.kind.name()).to_string()
        }
        ElemRef::Slab(_) => "Sol".into(),
        ElemRef::Roof(_) => "Toit".into(),
        ElemRef::Prop(i) => h.props.get(i).map_or("?", |p| p.kind.name()).to_string(),
        ElemRef::Pool(_) => "Piscine".into(),
        ElemRef::Road(_) => "Route".into(),
    }
}

/// Curseurs et finitions de l'élément sélectionné, modifiés directement dans la maison ;
/// faux si l'élément n'existe plus.
fn selection_props(ui: &mut egui::Ui, h: &mut House, sel: ElemRef, thumbs: &Thumbs) -> bool {
    match sel {
        ElemRef::Wall(i) => {
            let Some(w) = h.walls.get_mut(i) else { return false };
            ui.label(format!("Longueur : {:.2} m  •  {} ouverture(s)", w.len(), w.openings.len()));
            ui.add(Slider::new(&mut w.height, 0.5..=10.0).text("hauteur").suffix(" m"));
            ui.add(Slider::new(&mut w.thick, 0.05..=1.0).text("épaisseur").suffix(" m"));
            finish_row(ui, "s_mur", &mut w.color, &mut w.texture, thumbs);
        }
        ElemRef::Opening(wi, oi) => {
            let Some(w) = h.walls.get_mut(wi) else { return false };
            let (len, wall_h) = (w.len(), w.height);
            let Some(o) = w.openings.get_mut(oi) else { return false };
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
            let Some(s) = h.slabs.get_mut(i) else { return false };
            let size = s.max - s.min;
            ui.label(format!("{:.2} × {:.2} m  =  {:.1} m²", size.x, size.y, size.x * size.y));
            finish_row(ui, "s_sol", &mut s.color, &mut s.texture, thumbs);
        }
        ElemRef::Roof(i) => {
            let Some(r) = h.roofs.get_mut(i) else { return false };
            egui::ComboBox::from_label("forme ").selected_text(r.kind.name()).show_ui(ui, |ui| {
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
            let Some(p) = h.props.get_mut(i) else { return false };
            ui.add(Slider::new(&mut p.rot, 0.0..=359.0).text("rotation").suffix("°"));
            if p.kind.has_height() {
                ui.add(Slider::new(&mut p.height, 0.4..=8.0).text("hauteur").suffix(" m"));
            }
            finish_row(ui, "s_meuble", &mut p.color, &mut p.texture, thumbs);
            if ui.button("Couleur d'origine").clicked() {
                p.color = p.kind.default_color();
            }
        }
        ElemRef::Pool(i) => {
            let Some(p) = h.pools.get_mut(i) else { return false };
            let size = p.max - p.min;
            ui.label(format!("{:.0} × {:.0} m", size.x, size.y));
            ui.add(Slider::new(&mut p.depth, 0.5..=3.0).text("profondeur").suffix(" m"));
            finish_row(ui, "s_piscine", &mut p.color, &mut p.texture, thumbs);
        }
        ElemRef::Road(i) => {
            let Some(r) = h.roads.get_mut(i) else { return false };
            ui.label(format!("Longueur : {:.2} m", r.a.distance(r.b)));
            ui.add(Slider::new(&mut r.width, 1.0..=12.0).text("largeur").suffix(" m"));
            finish_row(ui, "s_route", &mut r.color, &mut r.texture, thumbs);
        }
    }
    true
}
