//! Arbres. Feuillages et troncs sont blancs : leur texture donne la couleur.

use bevy::math::Vec3;

use super::{LEAVES, LIGHT_LEAVES, TRUNK};
use crate::model::shapes::{Piece, ball, clumps, cyl};
use crate::model::{BIRCH_BARK, PropKind, Shape, Slot};

const NEEDLES: Slot = Slot::textured("Aiguilles", "Aiguilles de pin");

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Tree => const { &[LEAVES, TRUNK] },
        PropKind::Pine | PropKind::Cypress => const { &[NEEDLES, TRUNK] },
        PropKind::Birch => const { &[LIGHT_LEAVES, Slot::new("Tronc", BIRCH_BARK, "Écorce de bouleau")] },
        PropKind::Palm => {
            const { &[Slot::textured("Palmes", "Feuillage"), TRUNK, Slot::plain("Cœur", [0.45, 0.3, 0.15])] }
        }
        PropKind::AppleTree => const { &[LEAVES, TRUNK, Slot::plain("Pommes", [0.8, 0.1, 0.08])] },
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind) -> Vec<Piece> {
    match kind {
        PropKind::Tree => tree(),
        PropKind::Pine => pine(),
        PropKind::Birch => birch(),
        PropKind::Palm => palm(),
        PropKind::Cypress => cypress(),
        PropKind::AppleTree => apple_tree(),
        _ => Vec::new(),
    }
}

fn tree() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.32, 0.0, 2.6, 1)];
    // couronne en grappes de boules de tailles variées, pour une silhouette irrégulière
    v.extend(clumps(
        &[
            ([0.0, 3.3, 0.0], [2.6, 2.3, 2.6]),
            ([0.9, 2.9, 0.3], [1.8, 1.6, 1.8]),
            ([-0.8, 3.0, -0.4], [1.9, 1.7, 1.9]),
            ([0.2, 4.1, -0.5], [1.7, 1.5, 1.7]),
            ([-0.3, 4.2, 0.6], [1.6, 1.4, 1.6]),
            ([0.5, 2.6, -0.9], [1.4, 1.2, 1.4]),
            ([-0.6, 2.7, 0.9], [1.3, 1.2, 1.3]),
            ([0.0, 4.8, 0.0], [1.1, 1.0, 1.1]),
        ],
        0,
    ));
    v
}

fn pine() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.3, 0.0, 1.4, 1)];
    for (y, r, h) in [(1.7, 2.8, 1.9), (2.7, 2.2, 1.8), (3.6, 1.6, 1.6), (4.4, 1.0, 1.3), (5.0, 0.5, 0.9)] {
        v.push(Piece::new(Shape::Cone, Vec3::new(0.0, y, 0.0), Vec3::new(r, h, r), 0));
    }
    v
}

fn birch() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.22, 0.0, 4.0, 1)];
    v.extend(clumps(
        &[
            ([0.0, 4.2, 0.0], [1.6, 2.6, 1.6]),
            ([0.45, 3.6, 0.2], [1.2, 1.8, 1.2]),
            ([-0.4, 3.8, -0.25], [1.2, 2.0, 1.2]),
            ([0.1, 5.3, 0.1], [1.0, 1.6, 1.0]),
        ],
        0,
    ));
    v
}

fn palm() -> Vec<Piece> {
    // tronc en tronçons, légèrement penché, et palmes en étoile
    let mut v: Vec<Piece> = (0..6)
        .map(|i| {
            let k = i as f32;
            let d = 0.34 - k * 0.02;
            Piece::new(Shape::Cylinder, Vec3::new(k * 0.06, 0.4 + k * 0.8, 0.0), Vec3::new(d, 0.85, d), 1)
        })
        .collect();
    let top = Vec3::new(0.36, 4.9, 0.0);
    for k in 0..7 {
        let a = k as f32 * std::f32::consts::TAU / 7.0;
        let at = top + Vec3::new(a.cos(), -0.15, -a.sin()) * 1.1;
        v.push(Piece::new(Shape::Sphere, at, Vec3::new(2.4, 0.12, 0.55), 0).yaw(a));
    }
    v.push(Piece::new(Shape::Sphere, top, Vec3::splat(0.5), 2));
    v
}

fn cypress() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.25, 0.0, 0.6, 1)];
    v.extend(clumps(
        &[([0.0, 3.2, 0.0], [1.3, 5.6, 1.3]), ([0.0, 5.7, 0.0], [0.7, 1.6, 0.7]), ([0.1, 1.8, 0.1], [1.1, 2.6, 1.1])],
        0,
    ));
    v
}

fn apple_tree() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.26, 0.0, 1.8, 1)];
    v.extend(clumps(
        &[
            ([0.0, 2.6, 0.0], [2.6, 1.9, 2.6]),
            ([0.8, 2.3, 0.3], [1.5, 1.3, 1.5]),
            ([-0.7, 2.4, -0.4], [1.6, 1.4, 1.6]),
            ([0.1, 3.2, -0.2], [1.6, 1.2, 1.6]),
        ],
        0,
    ));
    // pommes à moitié dans la couronne
    for k in 0..10 {
        let (a, e) = (k as f32 * 2.4, [-0.4_f32, 0.1, 0.5][k % 3]);
        let p = [1.3 * e.cos() * a.cos(), 2.6 + 0.95 * e.sin(), -1.3 * e.cos() * a.sin()];
        v.push(ball(p, [0.16; 3], 2));
    }
    v
}
