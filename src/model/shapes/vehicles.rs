//! Véhicules. Seule la voiture (`Car`) se conduit pendant la visite.

use bevy::math::Vec3;

use super::{Piece, bx};
use crate::model::{DARK, PropKind, Shape, Slot, WHITE};

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Car => {
            const {
                &[
                    Slot::plain("Carrosserie", [0.72, 0.08, 0.1]),
                    Slot::plain("Tableau de bord et pare-chocs", DARK),
                    Slot::plain("Sièges", [0.25, 0.25, 0.28]),
                    Slot::plain("Phares", [1.0, 0.97, 0.85]),
                    Slot::plain("Feux arrière", [0.8, 0.05, 0.05]),
                    Slot::plain("Plaques", WHITE),
                    Slot::plain("Pneus", [0.08, 0.08, 0.09]),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    match kind {
        PropKind::Car => car(),
        _ => Vec::new(),
    }
}

/// Berline de 4,2 m, l'avant vers +z ; habitacle vitré pour conduire en vue intérieure.
fn car() -> Vec<Piece> {
    let glass =
        |x: f32, z: f32, sx: f32, sz: f32| Piece::new(Shape::Glass, Vec3::new(x, 1.2, z), Vec3::new(sx, 0.5, sz), 0);
    let mut v = vec![bx(0.0, 0.0, 1.8, 4.2, 0.3, 0.95, 0), bx(0.0, -0.15, 1.72, 2.1, 1.45, 1.52, 0)];
    for (x, z) in [(-0.84, 0.88), (0.84, 0.88), (-0.84, -1.18), (0.84, -1.18), (-0.84, -0.15), (0.84, -0.15)] {
        v.push(bx(x, z, 0.06, 0.08, 0.95, 1.45, 0));
    }
    v.push(glass(0.0, 0.88, 1.62, 0.03));
    v.push(glass(0.0, -1.18, 1.62, 0.03));
    v.push(glass(-0.85, -0.15, 0.03, 2.0));
    v.push(glass(0.85, -0.15, 0.03, 2.0));
    // intérieur : planche de bord, volant (place conducteur à gauche, x > 0), sièges
    v.push(bx(0.0, 0.72, 1.62, 0.3, 0.95, 1.05, 1));
    v.push(bx(0.4, 0.5, 0.36, 0.05, 1.0, 1.28, 1));
    for x in [-0.4, 0.4] {
        v.push(bx(x, -0.45, 0.5, 0.12, 0.95, 1.35, 2));
    }
    v.push(bx(0.0, -0.95, 1.5, 0.12, 0.95, 1.3, 2));
    // phares, feux, pare-chocs, plaques
    for x in [-0.6, 0.6] {
        v.push(bx(x, 2.1, 0.35, 0.03, 0.62, 0.76, 3));
        v.push(bx(x, -2.1, 0.35, 0.03, 0.66, 0.78, 4));
    }
    v.push(bx(0.0, 2.13, 1.82, 0.08, 0.3, 0.46, 1));
    v.push(bx(0.0, -2.13, 1.82, 0.08, 0.3, 0.46, 1));
    v.push(bx(0.0, 2.15, 0.5, 0.02, 0.47, 0.58, 5));
    v.push(bx(0.0, -2.15, 0.5, 0.02, 0.5, 0.61, 5));
    for (x, z) in [(-0.82, 1.35), (0.82, 1.35), (-0.82, -1.35), (0.82, -1.35)] {
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.34, z), Vec3::new(0.24, 0.68, 0.68), 6));
    }
    v
}
