//! Escaliers (marches d'environ 20 cm), piliers et autres éléments de structure, à la hauteur
//! demandée.

use bevy::math::Vec3;

use super::{Piece, bx, cyl};
use crate::model::{PropKind, Shape, Slot};

const STEPS: [f32; 3] = [0.6, 0.45, 0.3];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Stairs | PropKind::LStairs => const { &[Slot::plain("Marches", STEPS)] },
        PropKind::SpiralStairs => {
            const { &[Slot::plain("Marches", [0.35, 0.36, 0.38]), Slot::plain("Mât", [0.35, 0.36, 0.38])] }
        }
        PropKind::Pillar => const { &[Slot::plain("Pilier", [0.85, 0.85, 0.8])] },
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, height: f32) -> Vec<Piece> {
    let n = ((height / 0.2).round() as usize).max(1);
    let rise = height / n as f32;
    match kind {
        PropKind::Stairs => straight(n, rise),
        PropKind::SpiralStairs => spiral(n, rise, height),
        PropKind::LStairs => quarter_turn(n, rise),
        PropKind::Pillar => vec![bx(0.0, 0.0, 0.3, 0.3, 0.0, height, 0)],
        _ => Vec::new(),
    }
}

fn straight(n: usize, rise: f32) -> Vec<Piece> {
    let run = 0.28;
    (0..n)
        .map(|i| {
            let z = -(n as f32) * run / 2.0 + (i as f32 + 0.5) * run;
            bx(0.0, z, 1.0, run, 0.0, (i + 1) as f32 * rise, 0)
        })
        .collect()
}

fn spiral(n: usize, rise: f32, height: f32) -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.16, 0.0, height + 1.0, 1)];
    for i in 0..n {
        let a = i as f32 * 0.5;
        let top = (i + 1) as f32 * rise;
        let dir = Vec3::new(a.cos(), 0.0, -a.sin());
        v.push(Piece::new(Shape::Cube, dir * 0.55 + Vec3::Y * (top - 0.03), Vec3::new(0.9, 0.06, 0.42), 0).yaw(a));
    }
    v
}

/// Première volée vers le palier (centré sur l'origine), seconde volée vers +x.
fn quarter_turn(n: usize, rise: f32) -> Vec<Piece> {
    let run = 0.28;
    let n1 = n / 2;
    let mut v = Vec::new();
    for i in 0..n1 {
        v.push(bx(0.0, -0.5 - (n1 - i) as f32 * run + run / 2.0, 1.0, run, 0.0, (i + 1) as f32 * rise, 0));
    }
    v.push(bx(0.0, 0.0, 1.0, 1.0, 0.0, (n1 + 1) as f32 * rise, 0));
    for j in 0..n.saturating_sub(n1 + 1) {
        v.push(bx(0.5 + (j as f32 + 0.5) * run, 0.0, run, 1.0, 0.0, (n1 + 2 + j) as f32 * rise, 0));
    }
    v
}
