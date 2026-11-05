//! Bureau : postes de travail, rangements, réunion, accueil, salle serveur.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::math::Vec3;

use super::{Piece, ball, bx, carcass, cyl, door, drawer, legs, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const STEEL: [f32; 3] = [0.55, 0.57, 0.6];
const SCREEN: [f32; 3] = [0.05, 0.08, 0.12];
const INSIDE: [f32; 3] = [0.3, 0.31, 0.33];

// listes longues hors du `match` pour que `slots` reste lisible
const COMPUTER_DESK: &[Slot] = &[
    Slot::textured("Plateau", "Bois clair"),
    Slot::plain("Piétement", METAL),
    Slot::plain("Moniteur", DARK),
    Slot::plain("Écran", SCREEN),
    Slot::plain("Clavier et souris", [0.2, 0.2, 0.22]),
    Slot::plain("Unité centrale", [0.12, 0.12, 0.14]),
    Slot::plain("Voyant", [0.2, 0.7, 1.0]),
];
const OFFICE_CHAIR: &[Slot] = &[
    Slot::plain("Tissu", [0.18, 0.2, 0.26]),
    Slot::textured("Piétement", "Chrome"),
    Slot::plain("Accoudoirs", DARK),
    Slot::plain("Roulettes", [0.08, 0.08, 0.09]),
];
const OFFICE_CABINET: &[Slot] = &[
    Slot::plain("Caisson", [0.9, 0.9, 0.88]),
    Slot::plain("Poignées", METAL),
    Slot::plain("Intérieur", [0.8, 0.8, 0.78]),
    Slot::plain("Classeurs bleus", [0.15, 0.3, 0.6]),
    Slot::plain("Classeurs rouges", [0.7, 0.15, 0.12]),
];
const WHITEBOARD: &[Slot] = &[
    Slot::plain("Tableau", WHITE),
    Slot::plain("Cadre", METAL),
    Slot::plain("Pied", [0.3, 0.3, 0.32]),
    Slot::plain("Roulettes", [0.08, 0.08, 0.09]),
    Slot::plain("Feutres", [0.1, 0.3, 0.8]),
];
const COPIER: &[Slot] = &[
    Slot::plain("Carrosserie", [0.88, 0.88, 0.86]),
    Slot::plain("Bacs", [0.55, 0.56, 0.6]),
    Slot::plain("Écran", [0.1, 0.35, 0.6]),
    Slot::plain("Papier", WHITE),
    Slot::plain("Pupitre", DARK),
    Slot::plain("Roulettes", [0.08, 0.08, 0.09]),
];
const WATER_COOLER: &[Slot] = &[
    Slot::plain("Carrosserie", WHITE),
    Slot::plain("Façade", DARK),
    Slot::plain("Robinet froid", [0.2, 0.4, 0.85]),
    Slot::plain("Robinet chaud", [0.85, 0.15, 0.15]),
    Slot::plain("Bonbonne", [0.35, 0.6, 0.9]),
];
const LOCKERS: &[Slot] = &[
    Slot::plain("Caisson", STEEL),
    Slot::plain("Portes", [0.3, 0.45, 0.65]),
    Slot::plain("Poignées", METAL),
    Slot::plain("Intérieur", INSIDE),
    Slot::plain("Vêtements", [0.45, 0.35, 0.25]),
    Slot::plain("Sac", [0.75, 0.2, 0.15]),
];
const RECEPTION_DESK: &[Slot] = &[
    Slot::textured("Façade", "Bois clair"),
    Slot::textured("Comptoir", "Marbre blanc"),
    Slot::plain("Bandeau", [0.2, 0.45, 0.75]),
    Slot::plain("Moniteur", DARK),
    Slot::plain("Écran", SCREEN),
];
const SERVER_RACK: &[Slot] = &[
    Slot::plain("Baie", [0.1, 0.1, 0.12]),
    Slot::plain("Poignée", METAL),
    Slot::plain("Intérieur", INSIDE),
    Slot::plain("Serveurs", [0.28, 0.29, 0.32]),
    Slot::plain("Voyants verts", [0.2, 0.95, 0.3]),
    Slot::plain("Voyants orange", [1.0, 0.6, 0.1]),
];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Desk => {
            const {
                &[
                    Slot::textured("Plateau", "Bois clair"),
                    Slot::plain("Piétement", METAL),
                    Slot::plain("Dossiers", [0.85, 0.7, 0.3]),
                ]
            }
        }
        PropKind::ComputerDesk => COMPUTER_DESK,
        PropKind::OfficeChair => OFFICE_CHAIR,
        PropKind::FilingCabinet => {
            const {
                &[
                    Slot::plain("Caisson", STEEL),
                    Slot::plain("Poignées", METAL),
                    Slot::plain("Intérieur", INSIDE),
                    Slot::plain("Dossiers", [0.85, 0.7, 0.3]),
                ]
            }
        }
        PropKind::OfficeCabinet => OFFICE_CABINET,
        PropKind::MeetingTable => {
            const {
                &[Slot::textured("Plateau", "Bois foncé"), Slot::plain("Piétement", METAL), Slot::plain("Prises", DARK)]
            }
        }
        PropKind::Cubicle => const { &[Slot::textured("Tissu", "Tissu gris"), Slot::plain("Cadre", METAL)] },
        PropKind::Whiteboard => WHITEBOARD,
        PropKind::Copier => COPIER,
        PropKind::WaterCooler => WATER_COOLER,
        PropKind::Lockers => LOCKERS,
        PropKind::ReceptionDesk => RECEPTION_DESK,
        PropKind::CoatRack => {
            const {
                &[
                    Slot::textured("Bois", "Bois foncé"),
                    Slot::textured("Patères", "Chrome"),
                    Slot::plain("Manteau", [0.45, 0.35, 0.25]),
                    Slot::plain("Chapeau", [0.2, 0.2, 0.22]),
                ]
            }
        }
        PropKind::ServerRack => SERVER_RACK,
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Desk => {
            desk(&mut v, 1.4);
            v.push(bx(-0.45, -0.12, 0.32, 0.24, 0.75, 0.82, 2));
        }
        PropKind::ComputerDesk => computer_desk(&mut v),
        PropKind::OfficeChair => office_chair(&mut v),
        PropKind::FilingCabinet => filing_cabinet(&mut v, open),
        PropKind::OfficeCabinet => office_cabinet(&mut v, open),
        PropKind::MeetingTable => meeting_table(&mut v),
        PropKind::Cubicle => cubicle(&mut v),
        PropKind::Whiteboard => whiteboard(&mut v),
        PropKind::Copier => copier(&mut v),
        PropKind::WaterCooler => water_cooler(&mut v),
        PropKind::Lockers => lockers(&mut v, open),
        PropKind::ReceptionDesk => reception_desk(&mut v),
        PropKind::CoatRack => coat_rack(&mut v),
        PropKind::ServerRack => server_rack(&mut v, open),
        _ => {}
    }
    v
}

/// Plateau de `w` × 0,7 m à 75 cm (partie 0) sur piétement et voile de fond (partie 1).
fn desk(v: &mut Vec<Piece>, w: f32) {
    v.push(bx(0.0, 0.0, w, 0.7, 0.72, 0.75, 0));
    v.extend(legs(w / 2.0 - 0.05, 0.3, 0.72, 1));
    v.push(bx(0.0, -0.3, w - 0.15, 0.02, 0.35, 0.7, 1));
}

fn computer_desk(v: &mut Vec<Piece>) {
    desk(v, 1.4);
    // écran tourné vers l'avant, où l'on s'assoit
    v.push(bx(0.0, -0.2, 0.25, 0.18, 0.75, 0.77, 2));
    v.push(bx(0.0, -0.24, 0.05, 0.03, 0.77, 0.95, 2));
    v.push(bx(0.0, -0.21, 0.62, 0.03, 0.9, 1.27, 2));
    v.push(bx(0.0, -0.193, 0.58, 0.005, 0.92, 1.25, 3));
    v.push(bx(0.0, 0.12, 0.45, 0.15, 0.75, 0.77, 4));
    v.push(ball([0.32, 0.765, 0.12], [0.06, 0.03, 0.1], 4));
    // unité centrale au sol, sous le plateau
    v.push(bx(0.5, -0.05, 0.2, 0.45, 0.0, 0.45, 5));
    v.push(bx(0.5, 0.177, 0.02, 0.005, 0.38, 0.4, 6));
}

/// Piétement étoile à cinq branches sur roulettes, assise et dossier.
fn office_chair(v: &mut Vec<Piece>) {
    for k in 0..5 {
        let a = k as f32 * TAU / 5.0;
        let (s, c) = a.sin_cos();
        v.push(span(Shape::Cube, Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.3 * s, 0.07, 0.3 * c), 0.04, 1));
        v.push(Piece::new(Shape::Wheel, Vec3::new(0.3 * s, 0.03, 0.3 * c), Vec3::new(0.03, 0.06, 0.06), 3).yaw(a));
    }
    v.push(cyl(0.0, 0.0, 0.06, 0.1, 0.45, 1));
    v.push(bx(0.0, 0.02, 0.5, 0.5, 0.45, 0.53, 0));
    v.push(bx(0.0, -0.24, 0.06, 0.04, 0.45, 0.62, 1));
    v.push(bx(0.0, -0.25, 0.46, 0.06, 0.6, 1.1, 0));
    for x in [-0.27, 0.27] {
        v.push(bx(x, 0.0, 0.04, 0.04, 0.53, 0.68, 2));
        v.push(bx(x, 0.02, 0.06, 0.26, 0.68, 0.71, 2));
    }
}

/// Trois tiroirs ; des dossiers suspendus dans le grand tiroir du bas.
fn filing_cabinet(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.42, 0.55, 0.6, 0, 2));
    for (k, ys) in [(0.09, 0.3), (0.31, 0.44), (0.45, 0.57)].into_iter().enumerate() {
        v.extend(drawer(0.0, 0.275, 0.37, 0.5, ys, open * (0.18 + 0.08 * k as f32), (0, 1, 2)));
    }
    let z = 0.275 + open * 0.18 - 0.25;
    for k in 0..4 {
        v.push(bx(0.0, z - 0.15 + k as f32 * 0.1, 0.3, 0.015, 0.11, 0.28, 3));
    }
}

fn office_cabinet(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.9, 0.45, 1.95, 0, 2));
    for r in 0..4 {
        let y = 0.08 + r as f32 * 0.38;
        if r > 0 {
            v.push(bx(0.0, 0.0, 0.85, 0.4, y - 0.02, y, 0));
        }
        // une rangée de classeurs par étagère, couleur alternée
        for b in 0..5 {
            v.push(bx(-0.3 + b as f32 * 0.085, 0.02, 0.07, 0.3, y, y + 0.31, 3 + r % 2));
        }
    }
    v.extend(door(open, (-0.45, 0.225), 0.45, (0.09, 1.93), 1.0, (0, 1)));
    v.extend(door(open, (0.45, 0.225), 0.45, (0.09, 1.93), -1.0, (0, 1)));
}

/// Plateau ovale : un rectangle entre deux demi-disques.
fn meeting_table(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.3, 1.1, 0.72, 0.75, 0));
    for x in [-0.65, 0.65] {
        // un rien plus bas que le rectangle : pas de faces confondues
        v.push(cyl(x, 0.0, 1.1, 0.72, 0.749, 0));
        v.push(bx(x * 0.9, 0.0, 0.08, 0.6, 0.03, 0.72, 1));
        v.push(bx(x * 0.9, 0.0, 0.12, 0.8, 0.0, 0.03, 1));
    }
    v.push(bx(0.0, 0.0, 0.3, 0.12, 0.75, 0.76, 2));
}

/// Module de 1,6 m exactement (montants compris) : bout à bout, les modules forment une cloison.
fn cubicle(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.52, 0.05, 0.04, 1.17, 0));
    v.push(Piece::new(Shape::Glass, Vec3::new(0.0, 1.335, 0.0), Vec3::new(1.52, 0.27, 0.02), 0));
    for (y0, y1) in [(0.0, 0.04), (1.17, 1.2), (1.47, 1.5)] {
        v.push(bx(0.0, 0.0, 1.6, 0.07, y0, y1, 1));
    }
    for x in [-0.78, 0.78] {
        v.push(bx(x, 0.0, 0.04, 0.07, 0.0, 1.5, 1));
    }
}

fn whiteboard(v: &mut Vec<Piece>) {
    v.push(bx(0.0, -0.005, 1.54, 0.035, 0.83, 1.87, 1));
    v.push(bx(0.0, 0.003, 1.5, 0.03, 0.85, 1.85, 0));
    v.push(bx(0.0, 0.04, 0.6, 0.06, 0.82, 0.84, 1));
    for x in [-0.79, 0.79] {
        v.push(bx(x, 0.0, 0.04, 0.04, 0.07, 1.9, 2));
        v.push(bx(x, 0.0, 0.05, 0.6, 0.04, 0.07, 2));
        for z in [-0.27, 0.27] {
            v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.025, z), Vec3::new(0.025, 0.05, 0.05), 3));
        }
    }
    v.push(span(Shape::Cylinder, Vec3::new(-0.15, 0.85, 0.045), Vec3::new(-0.02, 0.85, 0.045), 0.02, 4));
    v.push(span(Shape::Cylinder, Vec3::new(0.05, 0.85, 0.035), Vec3::new(0.18, 0.85, 0.05), 0.02, 4));
}

/// Corps à bacs sur roulettes, sortie papier à gauche, pupitre tactile à l'avant.
fn copier(v: &mut Vec<Piece>) {
    for (x, z) in [(-0.26, -0.27), (0.26, -0.27), (-0.26, 0.27), (0.26, 0.27)] {
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.025, z), Vec3::new(0.03, 0.05, 0.05), 5));
    }
    v.push(bx(0.0, 0.0, 0.6, 0.62, 0.05, 1.05, 0));
    for k in 0..3 {
        let y = 0.1 + k as f32 * 0.2;
        v.push(bx(0.0, 0.315, 0.56, 0.012, y, y + 0.17, 1));
    }
    v.push(bx(0.0, -0.04, 0.6, 0.46, 1.05, 1.1, 0));
    v.push(bx(-0.36, 0.0, 0.12, 0.4, 0.82, 0.84, 1));
    v.push(bx(-0.36, 0.0, 0.1, 0.3, 0.84, 0.85, 3));
    v.push(bx(0.12, 0.27, 0.3, 0.08, 1.05, 1.1, 4));
    v.push(bx(0.08, 0.27, 0.16, 0.06, 1.1, 1.105, 2));
}

/// Bonbonne retournée sur le corps, robinets froid et chaud à l'avant.
fn water_cooler(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 0.32, 0.32, 0.0, 1.0, 0));
    v.push(bx(0.0, 0.155, 0.22, 0.02, 0.6, 0.9, 1));
    v.push(bx(0.0, 0.18, 0.18, 0.06, 0.62, 0.64, 1));
    v.push(bx(-0.05, 0.18, 0.03, 0.04, 0.78, 0.82, 2));
    v.push(bx(0.05, 0.18, 0.03, 0.04, 0.78, 0.82, 3));
    v.push(cyl(0.0, 0.0, 0.08, 1.0, 1.06, 4));
    v.push(ball([0.0, 1.1, 0.0], [0.27, 0.1, 0.27], 4));
    v.push(cyl(0.0, 0.0, 0.27, 1.1, 1.42, 4));
}

/// Quatre casiers de 30 cm à porte, tablette haute ; un manteau et un sac dedans.
fn lockers(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(1.2, 0.5, 1.8, 0, 3));
    for k in 0..4 {
        let x = -0.6 + k as f32 * 0.3;
        if k > 0 {
            v.push(bx(x, 0.0, 0.02, 0.48, 0.08, 1.78, 0));
        }
        v.push(bx(x + 0.15, 0.0, 0.27, 0.45, 1.45, 1.47, 0));
        v.extend(door(open, (x, 0.25), 0.3, (0.09, 1.76), 1.0, (1, 2)));
    }
    v.push(bx(-0.45, -0.05, 0.2, 0.3, 0.75, 1.4, 4));
    v.push(bx(0.15, 0.0, 0.2, 0.3, 0.08, 0.36, 5));
}

/// Comptoir haut côté public (avant), plan de travail et écran côté hôtesse (arrière).
fn reception_desk(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.35, 2.0, 0.06, 0.0, 1.1, 0));
    for x in [-0.98, 0.98] {
        v.push(bx(x, -0.02, 0.04, 0.74, 0.0, 1.1, 0));
    }
    v.push(bx(0.0, 0.3, 2.04, 0.3, 1.1, 1.14, 1));
    v.push(bx(0.0, -0.05, 1.92, 0.66, 0.72, 0.75, 1));
    v.push(bx(0.0, 0.385, 1.8, 0.012, 0.75, 0.9, 2));
    v.push(bx(0.45, -0.05, 0.25, 0.18, 0.75, 0.77, 3));
    v.push(bx(0.45, -0.03, 0.05, 0.03, 0.77, 0.9, 3));
    v.push(bx(0.45, -0.05, 0.55, 0.03, 0.88, 1.2, 3));
    v.push(bx(0.45, -0.067, 0.51, 0.005, 0.9, 1.18, 4));
}

/// Mât sur quatre pieds en croix, quatre patères ; un manteau et un chapeau accrochés.
fn coat_rack(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, 0.0, 0.04, 0.0, 1.8, 0));
    v.push(ball([0.0, 1.81, 0.0], [0.06, 0.06, 0.06], 0));
    for k in 0..4 {
        let (s, c) = (k as f32 * FRAC_PI_2).sin_cos();
        let (fs, fc) = ((k as f32 + 0.5) * FRAC_PI_2).sin_cos();
        v.push(span(Shape::Cylinder, Vec3::new(0.0, 0.25, 0.0), Vec3::new(0.28 * fs, 0.015, 0.28 * fc), 0.03, 0));
        v.push(span(Shape::Cylinder, Vec3::new(0.0, 1.62, 0.0), Vec3::new(0.14 * s, 1.72, 0.14 * c), 0.015, 1));
    }
    // manteau : épaules arrondies, buste, pans qui s'évasent
    v.push(ball([0.18, 1.64, 0.0], [0.12, 0.08, 0.36], 2));
    v.push(bx(0.18, 0.0, 0.12, 0.36, 1.25, 1.64, 2));
    v.push(bx(0.19, 0.0, 0.14, 0.42, 0.92, 1.25, 2));
    v.push(cyl(-0.16, 0.0, 0.28, 1.72, 1.74, 3));
    v.push(cyl(-0.16, 0.0, 0.17, 1.74, 1.86, 3));
}

/// Serveurs empilés, deux voyants chacun, derrière une porte vitrée.
fn server_rack(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.6, 1.0, 2.0, 0, 2));
    for k in 0..9 {
        let y = 0.12 + k as f32 * 0.2;
        v.push(bx(0.0, 0.0, 0.54, 0.9, y, y + 0.17, 3));
        v.push(bx(-0.2, 0.452, 0.03, 0.01, y + 0.07, y + 0.1, 4));
        v.push(bx(-0.15, 0.452, 0.03, 0.01, y + 0.07, y + 0.1, 5));
    }
    let [mut pane, handle] = door(open, (-0.3, 0.5), 0.6, (0.02, 1.98), 1.0, (0, 1));
    pane.shape = Shape::Glass;
    v.extend([pane, handle]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Category, LEVEL_H};

    #[test]
    fn only_storage_with_doors_or_drawers_opens() {
        let opening = [PropKind::FilingCabinet, PropKind::OfficeCabinet, PropKind::Lockers, PropKind::ServerRack];
        for k in Category::Office.kinds() {
            assert_eq!(k.openable(), opening.contains(&k), "{k:?}");
        }
    }
    #[test]
    fn cubicle_modules_line_up_every_1_6_m() {
        let s = PropKind::Cubicle.size();
        assert!((s.x - 1.6).abs() < 1e-4 && s.y < 0.1, "{s}");
    }
    #[test]
    fn server_rack_door_is_glass_and_swings() {
        let glass =
            |open: f32| shapes(PropKind::ServerRack, open, LEVEL_H).into_iter().find(|p| p.shape == Shape::Glass);
        let (shut, open) = (glass(0.0).expect("porte vitrée"), glass(1.0).expect("porte vitrée"));
        assert!(shut.rot.angle_between(open.rot) > 1.0);
    }
    #[test]
    fn office_chair_rolls_on_five_casters() {
        let pieces = shapes(PropKind::OfficeChair, 0.0, LEVEL_H);
        assert_eq!(pieces.iter().filter(|p| p.shape == Shape::Wheel).count(), 5);
        let s = PropKind::OfficeChair.size();
        assert!((s.x - s.y).abs() < 0.05, "piétement rond : {s}");
    }
}
