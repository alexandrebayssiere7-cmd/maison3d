//! Salon et chambre.

use bevy::math::Vec3;

use super::{Piece, bx, carcass, door, legs, light};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const WOOD: [f32; 3] = [0.55, 0.37, 0.2];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Table => const { &[Slot::plain("Plateau", WOOD), Slot::plain("Pieds", WOOD)] },
        PropKind::Chair => const { &[Slot::plain("Assise et dossier", WOOD), Slot::plain("Pieds", WOOD)] },
        PropKind::Sofa => const { &[Slot::plain("Tissu", [0.3, 0.42, 0.62]), Slot::plain("Pieds", DARK)] },
        PropKind::Lamp => const { &[Slot::plain("Abat-jour et lumière", [1.0, 0.9, 0.7]), Slot::plain("Pied", DARK)] },
        PropKind::Bed => {
            const { &[Slot::plain("Cadre", [0.45, 0.3, 0.2]), Slot::plain("Draps", WHITE), Slot::plain("Oreillers", WHITE)] }
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
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Table => {
            v.push(bx(0.0, 0.0, 1.6, 0.9, 0.72, 0.77, 0));
            v.extend(legs(0.72, 0.38, 0.72, 1));
        }
        PropKind::Chair => {
            v.push(bx(0.0, 0.0, 0.45, 0.45, 0.43, 0.48, 0));
            v.push(bx(0.0, -0.2, 0.45, 0.05, 0.48, 0.95, 0));
            v.extend(legs(0.19, 0.19, 0.43, 1));
        }
        PropKind::Sofa => {
            v.push(bx(0.0, 0.0, 2.0, 0.9, 0.1, 0.45, 0));
            v.push(bx(0.0, -0.35, 2.0, 0.2, 0.45, 0.85, 0));
            v.push(bx(-0.9, 0.0, 0.2, 0.9, 0.45, 0.65, 0));
            v.push(bx(0.9, 0.0, 0.2, 0.9, 0.45, 0.65, 0));
            v.extend(legs(0.9, 0.38, 0.1, 1));
        }
        PropKind::Lamp => {
            v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.3, 0.04, 0.3), 1));
            v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.04, 1.6, 0.04), 1));
            v.push(Piece::new(Shape::Sphere, Vec3::new(0.0, 1.7, 0.0), Vec3::new(0.4, 0.3, 0.4), 0));
            v.push(light([0.0, 1.5, 0.0], 0));
        }
        PropKind::Bed => {
            v.push(bx(0.0, 0.0, 1.6, 2.0, 0.1, 0.35, 0));
            v.push(bx(0.0, 0.05, 1.5, 1.85, 0.35, 0.55, 1));
            v.push(bx(0.0, -0.98, 1.6, 0.06, 0.1, 1.0, 0));
            v.push(bx(-0.38, -0.75, 0.6, 0.3, 0.55, 0.65, 2));
            v.push(bx(0.38, -0.75, 0.6, 0.3, 0.55, 0.65, 2));
        }
        PropKind::Wardrobe => wardrobe(&mut v, open),
        _ => {}
    }
    v
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
