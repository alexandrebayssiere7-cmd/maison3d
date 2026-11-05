//! Signalisation : feux, panneaux sur poteau (face vers l'avant, +z), marquages au sol et chantier.
//! Les marquages restent sous 2 cm pour ne pas scintiller avec la chaussée.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::math::{Quat, Vec3};

use crate::model::shapes::{Piece, ball, bx, cyl, span};
use crate::model::{PropKind, Shape, Slot, WHITE};

const RED: [f32; 3] = [0.8, 0.08, 0.08];
const BLACK: [f32; 3] = [0.05, 0.05, 0.06];
const PAINT: Slot = Slot::plain("Peinture", WHITE);

// Parties colorables de chaque sorte, la partie 0 en tête ; le poteau est toujours la dernière.
const TRAFFIC_LIGHT: &[Slot] = &[
    Slot::plain("Boîtier", [0.12, 0.13, 0.14]),
    Slot::plain("Feu rouge", [0.9, 0.1, 0.08]),
    Slot::plain("Feu orange", [1.0, 0.6, 0.05]),
    Slot::plain("Feu vert", [0.1, 0.85, 0.3]),
    Slot::plain("Mât", [0.55, 0.57, 0.6]),
];
const STOP_SIGN: &[Slot] = &[Slot::plain("Panneau", RED), Slot::plain("Bordure et texte", WHITE), POST];
const YIELD_SIGN: &[Slot] = &[Slot::plain("Bordure", RED), Slot::plain("Fond", WHITE), POST];
const NO_ENTRY_SIGN: &[Slot] = &[Slot::plain("Disque", RED), Slot::plain("Barre", WHITE), POST];
const SPEED_LIMIT_SIGN: &[Slot] =
    &[Slot::plain("Couronne", RED), Slot::plain("Fond", WHITE), Slot::plain("Chiffres", BLACK), POST];
const PEDESTRIAN_SIGN: &[Slot] =
    &[Slot::plain("Fond", [0.1, 0.3, 0.7]), Slot::plain("Triangle", WHITE), Slot::plain("Piéton", BLACK), POST];
const TRAFFIC_CONE: &[Slot] = &[
    Slot::plain("Cône", [0.95, 0.4, 0.05]),
    Slot::plain("Bandes réfléchissantes", WHITE),
    Slot::plain("Socle", [0.1, 0.1, 0.1]),
];
const WORK_BARRIER: &[Slot] = &[
    Slot::plain("Bandes rouges", [0.85, 0.1, 0.08]),
    Slot::plain("Bandes blanches", WHITE),
    Slot::plain("Montants", [0.6, 0.6, 0.62]),
    Slot::plain("Feu clignotant", [1.0, 0.6, 0.05]),
];
const SPEED_BUMP: &[Slot] =
    &[Slot::plain("Bandes noires", [0.08, 0.08, 0.09]), Slot::plain("Bandes jaunes", [0.95, 0.8, 0.1])];
const POST: Slot = Slot::plain("Poteau", [0.6, 0.62, 0.65]);

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::TrafficLight => TRAFFIC_LIGHT,
        PropKind::StopSign => STOP_SIGN,
        PropKind::YieldSign => YIELD_SIGN,
        PropKind::NoEntrySign => NO_ENTRY_SIGN,
        PropKind::SpeedLimitSign => SPEED_LIMIT_SIGN,
        PropKind::PedestrianSign => PEDESTRIAN_SIGN,
        PropKind::Crosswalk => const { &[Slot::plain("Bandes", WHITE)] },
        PropKind::RoadDash | PropKind::ParkingSpot => const { &[PAINT] },
        PropKind::TrafficCone => TRAFFIC_CONE,
        PropKind::WorkBarrier => WORK_BARRIER,
        PropKind::SpeedBump => SPEED_BUMP,
        _ => &[],
    }
}

pub(super) fn shapes(kind: PropKind, height: f32) -> Vec<Piece> {
    match kind {
        PropKind::TrafficLight => traffic_light(),
        PropKind::StopSign => stop_sign(),
        PropKind::YieldSign => yield_sign(),
        PropKind::NoEntrySign => vec![post(2.3, 2), disc(2.0, 0.05, 0.6, 0), plate(0.0, 2.0, 0.44, 0.1, 0.058, 1)],
        PropKind::SpeedLimitSign => speed_limit_sign(),
        PropKind::PedestrianSign => pedestrian_sign(),
        PropKind::Crosswalk => crosswalk(height),
        PropKind::RoadDash => vec![bx(0.0, 0.0, 3.0, 0.15, 0.0, 0.01, 0)],
        PropKind::ParkingSpot => vec![
            bx(-1.25, 0.0, 0.12, 5.0, 0.0, 0.01, 0),
            bx(1.25, 0.0, 0.12, 5.0, 0.0, 0.01, 0),
            bx(0.0, -2.44, 2.38, 0.12, 0.0, 0.01, 0),
        ],
        PropKind::TrafficCone => traffic_cone(),
        PropKind::WorkBarrier => work_barrier(),
        PropKind::SpeedBump => speed_bump(),
        _ => Vec::new(),
    }
}

/// Poteau de panneau de diamètre 6 cm, haut de `top`.
fn post(top: f32, slot: usize) -> Piece {
    cyl(0.0, 0.0, 0.06, 0.0, top, slot)
}

/// Disque de diamètre `d` centré en (0, y, z), tourné vers l'avant.
fn disc(y: f32, z: f32, d: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cylinder, Vec3::new(0.0, y, z), Vec3::new(d, 0.01, d), slot)
        .turn(Quat::from_rotation_x(FRAC_PI_2))
}

/// Plaque fine de `w` × `h` centrée en (x, y, z) : barres, lettres, chiffres.
fn plate(x: f32, y: f32, w: f32, h: f32, z: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cube, Vec3::new(x, y, z), Vec3::new(w, h, 0.005), slot)
}

/// Octogone régulier de `w` entre côtés opposés : quatre bandes tournées de 45°, dont l'union
/// donne exactement l'octogone.
fn octagon(y: f32, z: f32, w: f32, slot: usize) -> [Piece; 4] {
    let side = w * (PI / 8.0).tan();
    [0.0, 1.0, 2.0, 3.0].map(|k| {
        Piece::new(Shape::Cube, Vec3::new(0.0, y, z), Vec3::new(w, side, 0.01), slot)
            .turn(Quat::from_rotation_z(k * FRAC_PI_4))
    })
}

/// Triangle équilatéral de côté `w` (cône aplati en z) centré en (0, y, z), tourné de `turn`
/// autour de l'axe avant (π : pointe en bas).
fn triangle(y: f32, z: f32, w: f32, turn: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cone, Vec3::new(0.0, y, z), Vec3::new(w, w * 3f32.sqrt() / 2.0, 0.01), slot)
        .turn(Quat::from_rotation_z(turn))
}

/// Feu tricolore sur mât : rouge en haut, orange, vert en bas, chacun sous sa visière.
fn traffic_light() -> Vec<Piece> {
    let mut v = vec![
        cyl(0.0, 0.0, 0.22, 0.0, 0.1, 4),
        cyl(0.0, 0.0, 0.1, 0.1, 3.3, 4),
        bx(0.0, 0.07, 0.48, 0.02, 2.3, 3.3, 0),
        bx(0.0, 0.18, 0.32, 0.2, 2.4, 3.2, 0),
    ];
    for (slot, y) in [(1, 3.06), (2, 2.8), (3, 2.54)] {
        v.push(disc(y, 0.285, 0.2, slot));
        v.push(bx(0.0, 0.34, 0.24, 0.1, y + 0.1, y + 0.115, 0));
    }
    v
}

/// Panneau stop : octogone rouge bordé de blanc, lettres stylisées.
fn stop_sign() -> Vec<Piece> {
    let mut v = vec![post(2.3, 2), bx(0.0, 0.035, 0.1, 0.02, 1.9, 2.1, 2)];
    v.extend(octagon(2.0, 0.05, 0.64, 1));
    v.extend(octagon(2.0, 0.06, 0.58, 0));
    v.extend([-0.18, -0.06, 0.06, 0.18].map(|x| plate(x, 2.0, 0.07, 0.14, 0.068, 1)));
    v
}

/// Cédez le passage : triangle pointe en bas, bordure rouge et fond blanc de même centre.
fn yield_sign() -> Vec<Piece> {
    let (tip, big, small) = (1.7, 0.8, 0.5);
    let h = |w: f32| w * 3f32.sqrt() / 2.0;
    // le centre du cercle inscrit est aux deux tiers depuis la pointe
    let inner = tip + 2.0 * (h(big) - h(small)) / 3.0 + h(small) / 2.0;
    vec![post(2.3, 2), triangle(tip + h(big) / 2.0, 0.05, big, PI, 0), triangle(inner, 0.06, small, PI, 1)]
}

/// Limitation à 50 : couronne rouge, fond blanc, chiffres noirs en segments.
fn speed_limit_sign() -> Vec<Piece> {
    let mut v = vec![post(2.3, 3), disc(2.0, 0.05, 0.6, 0), disc(2.0, 0.056, 0.46, 1)];
    // segments (x, y, largeur, hauteur) du 5 puis du 0, autour du centre du panneau
    let digits = [
        (-0.075, 0.095, 0.11, 0.03),
        (-0.115, 0.055, 0.03, 0.11),
        (-0.075, 0.0, 0.11, 0.03),
        (-0.035, -0.055, 0.03, 0.11),
        (-0.075, -0.095, 0.11, 0.03),
        (0.035, 0.0, 0.03, 0.22),
        (0.115, 0.0, 0.03, 0.22),
        (0.075, 0.095, 0.11, 0.03),
        (0.075, -0.095, 0.11, 0.03),
    ];
    v.extend(digits.map(|(x, y, w, h)| plate(x, 2.0 + y, w, h, 0.064, 2)));
    v
}

/// Panneau carré bleu de passage piéton : triangle blanc et silhouette noire qui marche.
fn pedestrian_sign() -> Vec<Piece> {
    let z = 0.068;
    let limb =
        |a: [f32; 2], b: [f32; 2]| span(Shape::Cube, Vec3::new(a[0], a[1], z), Vec3::new(b[0], b[1], z), 0.022, 2);
    vec![
        post(2.35, 3),
        bx(0.0, 0.05, 0.6, 0.01, 1.75, 2.35, 0),
        triangle(1.8 + 0.25 * 3f32.sqrt() / 2.0, 0.058, 0.5, 0.0, 1),
        ball([0.02, 2.13, z], [0.05, 0.05, 0.02], 2),
        limb([0.01, 2.1], [-0.01, 1.98]),
        limb([-0.01, 1.98], [-0.07, 1.85]),
        limb([-0.01, 1.98], [0.05, 1.85]),
        limb([0.005, 2.08], [-0.06, 2.0]),
        limb([0.005, 2.08], [0.06, 2.02]),
    ]
}

/// Bandes de 2,5 m parallèles à la route (axe gauche-droite), répétées tous les mètres vers
/// l'avant sur `len` mètres : la largeur de chaussée traversée.
fn crosswalk(len: f32) -> Vec<Piece> {
    let band = 0.5_f32.min(len);
    let n = ((len + 0.5) / 1.0).floor().max(1.0) as usize;
    (0..n).map(|i| bx(0.0, i as f32 - (n - 1) as f32 / 2.0, 2.5, band, 0.0, 0.01, 0)).collect()
}

/// Cône de chantier de 50 cm sur socle carré : cônes emboîtés, un peu plus larges à chaque bande,
/// pour que les bandes épousent la pente.
fn traffic_cone() -> Vec<Piece> {
    let (base, tip) = (0.03, 0.53);
    let layer = |y0: f32, grow: f32, slot: usize| {
        let h = tip - y0;
        Piece::new(Shape::Cone, Vec3::new(0.0, y0 + h / 2.0, 0.0), Vec3::new(0.6 * h + grow, h, 0.6 * h + grow), slot)
    };
    vec![
        bx(0.0, 0.0, 0.38, 0.38, 0.0, base, 2),
        layer(base, 0.0, 0),
        layer(0.22, 0.008, 1),
        layer(0.3, 0.016, 0),
        layer(0.36, 0.024, 1),
        layer(0.42, 0.032, 0),
    ]
}

/// Barrière de chantier : deux lisses rayées rouge et blanc sur montants, feu clignotant à droite.
fn work_barrier() -> Vec<Piece> {
    let mut v = vec![cyl(0.72, 0.0, 0.06, 1.1, 1.14, 2), ball([0.72, 1.2, 0.0], [0.12, 0.12, 0.12], 3)];
    for x in [-0.72, 0.72] {
        v.push(bx(x, 0.0, 0.05, 0.05, 0.0, 1.1, 2));
        v.push(bx(x, 0.0, 0.07, 0.6, 0.0, 0.05, 2));
    }
    for y in [0.55, 0.9] {
        v.extend((0..6).map(|i| bx(-0.625 + i as f32 * 0.25, 0.03, 0.25, 0.03, y, y + 0.15, i % 2)));
    }
    v
}

/// Ralentisseur modulaire de 4 m en travers de la route (axe gauche-droite), haut de 7 cm,
/// modules noirs et jaunes en gradins.
fn speed_bump() -> Vec<Piece> {
    let mut v = Vec::new();
    for i in 0..8 {
        let x = -1.75 + i as f32 * 0.5;
        for (d, y0, y1) in [(0.9, 0.0, 0.025), (0.6, 0.025, 0.05), (0.3, 0.05, 0.07)] {
            v.push(bx(x, 0.0, 0.5, d, y0, y1, i % 2));
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::{LEVEL_H, Prop};

    fn local_box(kind: PropKind, height: f32) -> (Vec3, Vec3) {
        Prop { height, ..Prop::new(kind, Vec2::ZERO, 0, 0.0) }.local_box()
    }

    #[test]
    fn markings_lie_flat_just_above_the_road() {
        for k in [PropKind::Crosswalk, PropKind::RoadDash, PropKind::ParkingSpot] {
            let (lo, hi) = local_box(k, LEVEL_H);
            assert!(k.flat() && lo.y > -1e-4 && hi.y <= 0.02, "{k:?} : {lo} {hi}");
        }
        assert!(!PropKind::SpeedBump.flat() && !PropKind::TrafficCone.flat());
    }
    #[test]
    fn crosswalk_spans_the_road_width() {
        assert_eq!(PropKind::Crosswalk.dim(), Some("longueur"));
        for len in [0.2, 1.0, 3.0, 6.0, 12.5] {
            let (lo, hi) = local_box(PropKind::Crosswalk, len);
            let across = hi.z - lo.z;
            assert!(across <= len + 1e-4 && across > len - 1.01, "{len} → {across}");
            assert!((hi.x - lo.x - 2.5).abs() < 1e-4);
        }
    }
    #[test]
    fn traffic_light_lamps_are_separate_parts_top_to_bottom() {
        let pieces = traffic_light();
        let y = |slot| pieces.iter().find(|p| p.slot == slot).map(|p| p.at.y);
        assert!(y(1) > y(2) && y(2) > y(3), "rouge, orange, vert de haut en bas");
    }
    #[test]
    fn sign_faces_look_forward() {
        let (lo, hi) = local_box(PropKind::StopSign, LEVEL_H);
        assert!((hi.x - lo.x - 0.64).abs() < 0.01 && hi.z < 0.1, "octogone de 64 cm : {lo} {hi}");
        // le triangle du cédez-le-passage pointe vers le bas
        let tri = yield_sign()[1];
        assert!((tri.rot * Vec3::Y).y < -0.99);
    }
}
