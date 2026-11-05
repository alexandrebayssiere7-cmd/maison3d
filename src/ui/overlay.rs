//! Viseur et consignes de la visite, fenêtre d'aide.

use bevy_egui::egui;
use egui::{Color32, RichText};

use super::{ACCENT, hud_frame};
use crate::visit::Visitor;

/// Touche de clavier dessinée, suivie de ce qu'elle fait.
fn key_hint(ui: &mut egui::Ui, key: &str, label: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(Color32::from_white_alpha(30))
            .stroke(egui::Stroke::new(1.0_f32, Color32::from_white_alpha(70)))
            .corner_radius(4.0)
            .inner_margin(egui::vec2(6.0, 1.0))
            .show(ui, |ui| ui.label(RichText::new(key).strong().color(Color32::WHITE)));
        ui.label(RichText::new(label).color(Color32::from_gray(210)));
    });
}

/// Viseur, consignes et action possible pendant la visite.
pub(super) fn visit_overlay(ctx: &egui::Context, v: &Visitor) {
    let center = ctx.screen_rect().center();
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("viseur")));
    let aim = if v.target.is_some() { ACCENT } else { Color32::from_white_alpha(200) };
    painter.circle_stroke(center, 5.0, egui::Stroke::new(2.0_f32, aim));
    if let Some(kmh) = v.speed_kmh {
        egui::Area::new(egui::Id::new("vitesse"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -30.0])
            .interactable(false)
            .show(ctx, |ui| {
                hud_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{kmh:.0}")).size(44.0).strong().color(Color32::WHITE));
                        ui.label(RichText::new("km/h").size(16.0).color(ACCENT));
                    });
                });
            });
    }
    // actions possibles sous le viseur : celles liées à l'objet visé en évidence
    if !v.actions.is_empty() {
        egui::Area::new(egui::Id::new("actions"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, center.y + 26.0])
            .interactable(false)
            .show(ctx, |ui| {
                hud_frame().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        for a in &v.actions {
                            let strong = a.starts_with("E :") || a.contains("relever") || v.speed_kmh.is_some();
                            let color = if strong { ACCENT } else { Color32::from_gray(190) };
                            ui.label(RichText::new(*a).size(if strong { 17.0 } else { 14.0 }).color(color));
                        }
                    });
                });
            });
    }
    egui::Area::new(egui::Id::new("visite")).anchor(egui::Align2::LEFT_BOTTOM, [14.0, -14.0]).interactable(false).show(
        ctx,
        |ui| {
            hud_frame().show(ui, |ui| {
                ui.label(RichText::new("VISITE").strong().color(ACCENT));
                for (k, l) in [
                    ("Souris", "regarder"),
                    ("ZQSD", "marcher"),
                    ("Maj", "courir"),
                    ("Espace", "sauter"),
                    ("E", "ouvrir / s'asseoir / s'allonger"),
                    ("C", "s'asseoir"),
                    ("X", "s'allonger"),
                    ("V", "1re / 3e personne"),
                    ("Échap", "revenir à l'édition"),
                ] {
                    key_hint(ui, k, l);
                }
            });
        },
    );
}

pub(super) fn help(ui: &mut egui::Ui) {
    let section = |ui: &mut egui::Ui, title: &str, lines: &[&str]| {
        ui.label(RichText::new(title).strong().color(ACCENT));
        for l in lines {
            ui.label(format!("  • {l}"));
        }
        ui.add_space(4.0);
    };
    section(
        ui,
        "Démarrer en 30 secondes",
        &[
            "Outil Pièce (P) : glissez un rectangle (ou cliquez 2 coins) → 4 murs + un sol",
            "Outil Mur (M) : glissez pour un mur, ou cliquez coin après coin",
            "Porte (O) / Fenêtre (F) : survolez un mur, cliquez",
            "Montez d'un étage (+), outil Toit (T), glissez au-dessus des murs",
            "Mobilier (N) : choisissez un meuble, cliquez pour le poser",
            "Tab : posez le bonhomme où vous voulez et visitez (E ouvre portes et placards)",
            "Peinture (G) : choisissez une couleur et/ou une des 55 textures",
        ],
    );
    section(
        ui,
        "Extérieur",
        &[
            "Relief (K) : pinceau monter / creuser / aplanir / lisser / aléatoire, ou « Générer » des collines et montagnes",
            "Piscine (J) : glissez un rectangle, elle se creuse dans le terrain",
            "Route (U) : comme les murs, point après point ; le terrain est terrassé dessous",
            "Mobilier (N) → Jardin : arbres, fleurs, haies… posés sur le relief",
            "« + Ajouter un étage » : recopie les murs et sols de l'étage courant au-dessus",
        ],
    );
    section(
        ui,
        "Caméra",
        &[
            "ZQSD / WASD : avancer vers là où vous regardez  •  Espace / C : monter / descendre",
            "A/E (Q/E en QWERTY) : pivoter  •  Maj : aller plus vite",
            "Flèches (sans sélection) : pivoter / incliner",
            "Clic droit + glisser, ou Alt (⌥) + clic gauche + glisser : tourner",
            "Trackpad : glisser à deux doigts = tourner, pincer = zoom",
            "Souris : molette = zoom, clic molette ou Maj + clic droit = déplacer la vue",
        ],
    );
    section(
        ui,
        "Édition",
        &[
            "Ctrl+Z / Ctrl+Y : annuler / rétablir",
            "Sélection (V) : clic pour choisir, glisser pour déplacer, flèches pour décaler",
            "Suppr : effacer  •  Ctrl+D : dupliquer  •  R : pivoter un meuble",
            "Échap ou clic droit : annuler le tracé en cours",
            "Maj enfoncée : placement libre (sans grille)",
            "Les extrémités de murs s'aimantent entre elles",
        ],
    );
    section(
        ui,
        "Étages et fichiers",
        &[
            "Page↑ / Page↓ (ou + / -) : changer d'étage  •  H : masquer les étages au-dessus",
            "Ctrl+S / Ctrl+O : enregistrer / ouvrir (fichier JSON)",
        ],
    );
}
