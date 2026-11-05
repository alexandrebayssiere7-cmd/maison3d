//! Chambre : lits (tête à l'arrière, -z), rangements, coiffeuse, coffre à jouets.

use bevy::math::{Quat, Vec3};

use super::super::{Piece, ball, bx, carcass, cyl, door, drawer, legs};
use crate::model::{METAL, PropKind, Shape, Slot, WHITE};

const FRAME: [f32; 3] = [0.45, 0.3, 0.2];
const INSIDE: [f32; 3] = [0.85, 0.8, 0.7];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Bed => {
            const { &[Slot::plain("Cadre", FRAME), Slot::plain("Draps", WHITE), Slot::plain("Oreillers", WHITE)] }
        }
        PropKind::Wardrobe => {
            const {
                &[
                    Slot::plain("Bois", [0.72, 0.56, 0.4]),
                    Slot::plain("Poignées et tringle", METAL),
                    Slot::plain("Linge plié", [0.7, 0.3, 0.3]),
                    Slot::plain("Vêtements foncés", [0.2, 0.25, 0.4]),
                    Slot::plain("Vêtements clairs", [0.85, 0.85, 0.8]),
                ]
            }
        }
        PropKind::SingleBed => {
            const {
                &[
                    Slot::plain("Cadre", FRAME),
                    Slot::plain("Draps", WHITE),
                    Slot::plain("Oreiller", WHITE),
                    Slot::plain("Couette", [0.4, 0.55, 0.75]),
                ]
            }
        }
        PropKind::BunkBed => {
            const {
                &[
                    Slot::textured("Bois", "Bois clair"),
                    Slot::plain("Draps", WHITE),
                    Slot::plain("Oreillers", WHITE),
                    Slot::plain("Couettes", [0.85, 0.45, 0.3]),
                    Slot::plain("Échelle", [0.7, 0.55, 0.38]),
                ]
            }
        }
        PropKind::Crib => {
            const {
                &[
                    Slot::plain("Bois", WHITE),
                    Slot::plain("Barreaux", WHITE),
                    Slot::plain("Matelas", [0.75, 0.85, 0.95]),
                    Slot::plain("Doudou", [0.75, 0.55, 0.35]),
                ]
            }
        }
        PropKind::Nightstand => {
            const {
                &[
                    Slot::textured("Bois", "Bois clair"),
                    Slot::plain("Poignée", METAL),
                    Slot::plain("Intérieur", INSIDE),
                    Slot::plain("Livre", [0.6, 0.2, 0.2]),
                ]
            }
        }
        PropKind::Dresser => {
            const {
                &[
                    Slot::textured("Bois", "Bois clair"),
                    Slot::plain("Poignées", METAL),
                    Slot::plain("Intérieur", INSIDE),
                    Slot::plain("Linge", [0.7, 0.3, 0.3]),
                ]
            }
        }
        PropKind::Vanity => {
            const {
                &[
                    Slot::plain("Bois", WHITE),
                    Slot::plain("Miroir", [0.78, 0.86, 0.9]),
                    Slot::plain("Flacons", [0.9, 0.55, 0.65]),
                ]
            }
        }
        PropKind::ToyChest => {
            const {
                &[
                    Slot::plain("Caisse", [0.35, 0.6, 0.85]),
                    Slot::plain("Couvercle", [0.95, 0.8, 0.25]),
                    Slot::plain("Intérieur", INSIDE),
                    Slot::plain("Ballon", [0.85, 0.15, 0.15]),
                    Slot::plain("Cubes", [0.3, 0.7, 0.3]),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Bed => bed(&mut v, 1.6, &[-0.38, 0.38]),
        PropKind::Wardrobe => wardrobe(&mut v, open),
        PropKind::SingleBed => {
            bed(&mut v, 0.9, &[0.0]);
            v.push(bx(0.0, 0.3, 0.88, 1.4, 0.55, 0.6, 3));
        }
        PropKind::BunkBed => bunk_bed(&mut v),
        PropKind::Crib => crib(&mut v),
        PropKind::Nightstand => {
            v.extend(carcass(0.45, 0.38, 0.55, 0, 2));
            v.push(bx(0.0, 0.0, 0.4, 0.33, 0.3, 0.32, 0));
            v.push(bx(-0.05, 0.0, 0.2, 0.15, 0.08, 0.11, 3));
            v.extend(drawer(0.0, 0.19, 0.4, 0.33, (0.33, 0.52), open * 0.25, (0, 1, 2)));
        }
        PropKind::Dresser => dresser(&mut v, open),
        PropKind::Vanity => vanity(&mut v),
        PropKind::ToyChest => toy_chest(&mut v, open),
        _ => {}
    }
    v
}

/// Lit de largeur `w`, long de 2 m, sur pieds : cadre et tête (partie 0), matelas (1),
/// oreillers (2) centrés en x sur `pillows`.
fn bed(v: &mut Vec<Piece>, w: f32, pillows: &[f32]) {
    v.push(bx(0.0, 0.0, w, 2.0, 0.1, 0.35, 0));
    v.push(bx(0.0, 0.05, w - 0.1, 1.85, 0.35, 0.55, 1));
    v.push(bx(0.0, -0.98, w, 0.06, 0.1, 1.0, 0));
    v.extend(legs(w / 2.0 - 0.05, 0.95, 0.1, 0));
    for &x in pillows {
        v.push(bx(x, -0.75, 0.6, 0.3, 0.55, 0.65, 2));
    }
}

fn wardrobe(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(1.2, 0.6, 2.0, 0, 0));
    v.push(bx(0.0, 0.0, 1.15, 0.55, 1.5, 1.53, 0));
    v.push(bx(0.0, -0.05, 1.15, 0.03, 1.38, 1.4, 1));
    // linge plié et vêtements suspendus
    v.push(bx(-0.3, 0.0, 0.35, 0.35, 1.53, 1.68, 2));
    v.push(bx(0.25, 0.0, 0.35, 0.35, 1.53, 1.65, 3));
    for i in 0..4 {
        v.push(bx(-0.35 + i as f32 * 0.22, -0.05, 0.05, 0.45, 0.6, 1.38, 3 + i % 2));
    }
    v.extend(door(open, (-0.6, 0.3), 0.6, (0.02, 1.98), 1.0, (0, 1)));
    v.extend(door(open, (0.6, 0.3), 0.6, (0.02, 1.98), -1.0, (0, 1)));
}

/// Deux couchages superposés entre quatre montants, échelle au pied côté +x.
fn bunk_bed(v: &mut Vec<Piece>) {
    for (x, z) in [(-0.45, -0.98), (0.45, -0.98), (-0.45, 0.98), (0.45, 0.98)] {
        v.push(bx(x, z, 0.06, 0.06, 0.0, 1.7, 0));
    }
    for y in [0.15, 1.05] {
        v.push(bx(0.0, 0.0, 0.9, 1.96, y, y + 0.15, 0));
        v.push(bx(0.0, 0.02, 0.84, 1.9, y + 0.15, y + 0.3, 1));
        v.push(bx(0.0, -0.97, 0.84, 0.03, y + 0.15, y + 0.6, 0));
        v.push(bx(0.0, -0.75, 0.5, 0.25, y + 0.3, y + 0.38, 2));
        v.push(bx(0.0, 0.3, 0.86, 1.3, y + 0.3, y + 0.35, 3));
    }
    // barrières du couchage du haut, ouvertes au-dessus de l'échelle
    for x in [-0.45, 0.45] {
        v.push(bx(x, 0.0, 0.04, 1.9, 1.5, 1.56, 0));
    }
    v.push(bx(-0.2, 0.98, 0.46, 0.04, 1.5, 1.56, 0));
    for x in [0.08, 0.42] {
        v.push(bx(x, 1.05, 0.04, 0.04, 0.0, 1.62, 4));
    }
    for k in 0..5 {
        let y = 0.32 + k as f32 * 0.28;
        v.push(bx(0.25, 1.05, 0.34, 0.03, y, y + 0.03, 4));
    }
}

/// Lit à barreaux sur les longs côtés, panneaux pleins aux deux bouts.
fn crib(v: &mut Vec<Piece>) {
    let (hx, hz) = (0.31, 0.6);
    for (x, z) in [(-hx, -hz), (hx, -hz), (-hx, hz), (hx, hz)] {
        v.push(bx(x, z, 0.05, 0.05, 0.0, 0.9, 0));
    }
    for z in [-hz, hz] {
        v.push(bx(0.0, z, 0.62, 0.03, 0.2, 0.88, 0));
    }
    for x in [-hx, hx] {
        v.push(bx(x, 0.0, 0.04, 1.2, 0.84, 0.88, 0));
        v.push(bx(x, 0.0, 0.04, 1.2, 0.2, 0.24, 0));
        for k in 0..11 {
            v.push(bx(x, -0.5 + k as f32 * 0.1, 0.02, 0.02, 0.24, 0.84, 1));
        }
    }
    v.push(bx(0.0, 0.0, 0.6, 1.16, 0.24, 0.28, 0));
    v.push(bx(0.0, 0.0, 0.58, 1.14, 0.28, 0.38, 2));
    v.push(ball([0.12, 0.45, -0.35], [0.14, 0.14, 0.12], 3));
    v.push(ball([0.12, 0.56, -0.35], [0.1, 0.1, 0.1], 3));
}

/// Trois tiroirs, le plus haut sort le plus loin et laisse voir le linge.
fn dresser(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(1.0, 0.5, 0.85, 0, 2));
    let mut out = 0.0;
    for (k, ys) in [(0.1, 0.34), (0.35, 0.59), (0.6, 0.82)].into_iter().enumerate() {
        out = open * (0.15 + 0.08 * k as f32);
        v.extend(drawer(0.0, 0.25, 0.95, 0.44, ys, out, (0, 1, 2)));
    }
    v.push(bx(0.0, 0.25 + out - 0.22, 0.7, 0.3, 0.62, 0.7, 3));
}

/// Plateau sur pieds et miroir ovale sur deux montants, à l'arrière.
fn vanity(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.0, 0.45, 0.72, 0.76, 0));
    v.extend(legs(0.47, 0.2, 0.72, 0));
    v.push(bx(0.0, 0.2, 0.9, 0.03, 0.6, 0.72, 0));
    for x in [-0.36, 0.36] {
        v.push(bx(x, -0.17, 0.04, 0.04, 0.76, 1.3, 0));
    }
    // disques couchés debout : épaisseur selon z, ovale de 0,7 × 0,9 m
    let up = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 1.25, -0.17), Vec3::new(0.7, 0.03, 0.9), 0).turn(up));
    v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 1.25, -0.15), Vec3::new(0.62, 0.01, 0.82), 1).turn(up));
    v.push(cyl(0.3, 0.05, 0.06, 0.76, 0.9, 2));
    v.push(cyl(0.4, 0.0, 0.05, 0.76, 0.86, 2));
    v.push(ball([0.3, 0.92, 0.05], [0.04, 0.04, 0.04], 2));
}

/// Caisse ouverte dont le couvercle pivote sur l'arête arrière du dessus.
fn toy_chest(v: &mut Vec<Piece>, open: f32) {
    let (w, d, h) = (0.8, 0.45, 0.45);
    for z in [-0.21, 0.21] {
        v.push(bx(0.0, z, w, 0.03, 0.0, h, 0));
    }
    for x in [-0.385, 0.385] {
        v.push(bx(x, 0.0, 0.03, d, 0.0, h, 0));
    }
    v.push(bx(0.0, 0.0, w - 0.06, d - 0.06, 0.0, 0.05, 2));
    v.push(ball([0.18, 0.15, 0.0], [0.2, 0.2, 0.2], 3));
    v.push(bx(-0.15, 0.03, 0.14, 0.14, 0.05, 0.19, 4).yaw(0.4));
    v.push(bx(-0.2, -0.09, 0.12, 0.12, 0.05, 0.17, 4));
    let q = Quat::from_rotation_x(-open * 1.4);
    let at = Vec3::new(0.0, h, -d / 2.0) + q * Vec3::new(0.0, 0.02, d / 2.0 + 0.01);
    v.push(Piece::new(Shape::Cube, at, Vec3::new(w + 0.02, 0.04, d + 0.02), 1).turn(q));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn top(pieces: &[Piece]) -> f32 {
        pieces.iter().map(|p| p.at.y + p.size.y / 2.0).fold(0.0, f32::max)
    }

    #[test]
    fn toy_chest_lid_lifts() {
        let (shut, open) = (shapes(PropKind::ToyChest, 0.0), shapes(PropKind::ToyChest, 1.0));
        assert!((top(&shut) - 0.49).abs() < 1e-4, "couvercle posé : {}", top(&shut));
        let lid = open.last().expect("couvercle");
        assert!(lid.at.y > 0.6 && lid.at.z < -0.1, "relevé vers l'arrière : {lid:?}");
    }
    #[test]
    fn drawers_come_out_toward_the_front() {
        for kind in [PropKind::Nightstand, PropKind::Dresser] {
            let front = |open: f32| shapes(kind, open).iter().map(|p| p.at.z).fold(f32::MIN, f32::max);
            assert!(front(1.0) > front(0.0) + 0.2, "{kind:?}");
        }
    }
    #[test]
    fn beds_keep_their_size_and_stand_on_legs() {
        for (kind, w) in [(PropKind::Bed, 1.6), (PropKind::SingleBed, 0.9)] {
            let s = kind.size();
            assert!((s.x - w).abs() < 0.01 && (s.y - 2.0).abs() < 0.01, "{kind:?} : {s}");
            let low = shapes(kind, 0.0).iter().map(|p| p.at.y - p.size.y / 2.0).fold(f32::MAX, f32::min);
            assert!(low.abs() < 1e-6, "{kind:?} posé au sol");
        }
        assert!(top(&shapes(PropKind::BunkBed, 0.0)) > 1.6, "couchage du haut");
    }
}
