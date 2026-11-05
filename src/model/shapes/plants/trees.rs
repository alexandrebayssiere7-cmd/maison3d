//! Arbres. Feuillages et troncs sont blancs : leur texture donne la couleur.

use bevy::math::{Quat, Vec3};

use super::{LEAVES, LIGHT_LEAVES, TRUNK};
use crate::model::shapes::{Piece, ball, clumps, cyl, span};
use crate::model::{BIRCH_BARK, PropKind, Shape, Slot};

const NEEDLES: Slot = Slot::textured("Aiguilles", "Aiguilles de pin");

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Tree | PropKind::Oak => const { &[LEAVES, TRUNK] },
        PropKind::Pine | PropKind::Cypress => const { &[NEEDLES, TRUNK] },
        PropKind::Birch => const { &[LIGHT_LEAVES, Slot::new("Tronc", BIRCH_BARK, "Écorce de bouleau")] },
        PropKind::Palm => {
            const { &[Slot::textured("Palmes", "Feuillage"), TRUNK, Slot::plain("Cœur", [0.45, 0.3, 0.15])] }
        }
        PropKind::AppleTree => const { &[LEAVES, TRUNK, Slot::plain("Pommes", [0.8, 0.1, 0.08])] },
        PropKind::WeepingWillow => const { &[LIGHT_LEAVES, TRUNK] },
        // teinte qui grise le feuillage clair
        PropKind::OliveTree => const { &[Slot::new("Feuillage", [0.95, 0.85, 0.9], "Feuillage clair"), TRUNK] },
        PropKind::Maple => const { &[Slot::new("Feuillage", [1.0, 0.62, 0.58], "Feuillage d'automne"), TRUNK] },
        PropKind::CherryBlossom => {
            const { &[Slot::plain("Fleurs", [0.98, 0.74, 0.84]), TRUNK, Slot::textured("Feuilles", "Feuillage")] }
        }
        PropKind::PlaneTree => const { &[LEAVES, Slot::new("Tronc", [0.9, 0.86, 0.7], "Écorce de bouleau")] },
        PropKind::DeadTree => const { &[Slot::new("Bois", [0.85, 0.82, 0.78], "Écorce")] },
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
        PropKind::Oak => oak(),
        PropKind::WeepingWillow => weeping_willow(),
        PropKind::OliveTree => olive_tree(),
        PropKind::Maple => maple(),
        PropKind::CherryBlossom => cherry_blossom(),
        PropKind::PlaneTree => plane_tree(),
        PropKind::DeadTree => dead_tree(),
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

/// Charpentières : branches de section `d` qui partent du tronc en `from` vers chaque `ends`.
fn limbs(from: Vec3, ends: &[[f32; 3]], d: f32, slot: usize) -> Vec<Piece> {
    ends.iter().map(|&e| span(Shape::Cylinder, from, Vec3::from(e), d, slot)).collect()
}

fn oak() -> Vec<Piece> {
    // pied évasé, tronc épais et charpentières en éventail sous une couronne large
    let mut v = vec![
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.35, 0.0), Vec3::new(1.3, 0.7, 1.3), 1),
        cyl(0.0, 0.0, 0.75, 0.0, 2.4, 1),
    ];
    v.extend(limbs(
        Vec3::new(0.0, 2.1, 0.0),
        &[[1.6, 3.6, 0.5], [-1.5, 3.6, -0.6], [0.4, 3.6, -1.6], [-0.5, 3.6, 1.5]],
        0.32,
        1,
    ));
    v.extend(clumps(
        &[
            ([0.0, 4.6, 0.0], [4.6, 3.0, 4.6]),
            ([2.1, 4.0, 0.6], [3.0, 2.4, 3.0]),
            ([-2.0, 4.1, -0.6], [3.1, 2.5, 3.1]),
            ([0.5, 4.2, -2.1], [2.9, 2.3, 2.9]),
            ([-0.6, 4.0, 2.0], [2.9, 2.3, 2.9]),
            ([0.4, 5.9, 0.3], [3.2, 2.2, 3.2]),
            ([1.6, 5.4, -1.1], [2.2, 1.8, 2.2]),
            ([-1.5, 5.5, 1.0], [2.2, 1.8, 2.2]),
        ],
        0,
    ));
    v
}

fn weeping_willow() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.5, 0.0, 3.0, 1)];
    v.extend(limbs(
        Vec3::new(0.0, 2.7, 0.0),
        &[[1.0, 4.0, 0.3], [-0.9, 4.0, -0.4], [0.2, 4.0, -1.0], [-0.3, 4.0, 1.0]],
        0.2,
        1,
    ));
    v.extend(clumps(
        &[([0.0, 4.6, 0.0], [3.6, 2.2, 3.6]), ([0.8, 4.2, 0.5], [2.4, 1.6, 2.4]), ([-0.8, 4.3, -0.4], [2.4, 1.6, 2.4])],
        0,
    ));
    // rideaux de rameaux : ellipsoïdes allongés, à plat face à l'extérieur, évasés vers le bas,
    // qui retombent presque jusqu'au sol en laissant voir le tronc
    for (n, r, len) in [(14, 1.9, 3.2), (7, 0.9, 2.8)] {
        for k in 0..n {
            let a = (k as f32 + 0.5 * (r - 0.9)) * std::f32::consts::TAU / n as f32;
            let len = len + 0.4 * ((k * 5) % 3) as f32;
            let at = [(r + 0.3) * a.sin(), 4.5 - len / 2.0, (r + 0.3) * a.cos()];
            v.push(ball(at, [0.7, len, 0.35], 0).turn(Quat::from_rotation_y(a) * Quat::from_rotation_x(-0.18)));
        }
    }
    v
}

fn olive_tree() -> Vec<Piece> {
    // tronc tortueux : tronçons qui changent de direction, un nœud à chaque coude
    let knots = [Vec3::ZERO, Vec3::new(0.25, 0.7, 0.1), Vec3::new(-0.05, 1.3, 0.2), Vec3::new(0.15, 1.9, -0.05)];
    let mut v = vec![Piece::new(Shape::Cone, Vec3::new(0.0, 0.25, 0.0), Vec3::new(0.8, 0.5, 0.8), 1)];
    for (i, w) in knots.windows(2).enumerate() {
        let d = 0.45 - 0.07 * i as f32;
        v.push(span(Shape::Cylinder, w[0], w[1], d, 1));
        v.push(ball(w[1].to_array(), [d; 3], 1));
    }
    v.extend(limbs(knots[3], &[[1.0, 2.5, 0.5], [-0.9, 2.4, -0.4], [0.0, 2.7, -1.0], [-0.3, 2.6, 0.9]], 0.16, 1));
    // feuillage léger, en touffes aplaties et espacées
    v.extend(clumps(
        &[
            ([1.1, 2.8, 0.5], [1.8, 1.1, 1.7]),
            ([-1.0, 2.7, -0.5], [1.9, 1.1, 1.8]),
            ([0.0, 3.0, -1.1], [1.7, 1.0, 1.6]),
            ([-0.4, 2.9, 1.0], [1.7, 1.0, 1.6]),
            ([0.2, 3.4, 0.1], [2.2, 1.2, 2.2]),
        ],
        0,
    ));
    v
}

fn maple() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.36, 0.0, 2.6, 1)];
    v.extend(clumps(
        &[
            ([0.0, 4.2, 0.0], [3.6, 3.4, 3.6]),
            ([1.2, 3.5, 0.4], [2.2, 2.0, 2.2]),
            ([-1.1, 3.6, -0.5], [2.3, 2.1, 2.3]),
            ([0.3, 3.4, -1.2], [2.0, 1.9, 2.0]),
            ([-0.4, 3.5, 1.2], [2.0, 1.9, 2.0]),
            ([0.2, 5.6, 0.1], [2.4, 2.0, 2.4]),
            ([0.0, 6.4, 0.0], [1.3, 1.1, 1.3]),
        ],
        0,
    ));
    v
}

fn cherry_blossom() -> Vec<Piece> {
    // tronc court, branches étalées en coupe sous une couronne large et plate
    let mut v = vec![cyl(0.0, 0.0, 0.34, 0.0, 1.5, 1)];
    let ends = [[1.7, 2.6, 0.5], [-1.6, 2.7, -0.4], [0.4, 2.8, -1.6], [-0.5, 2.6, 1.6], [0.0, 3.2, 0.0]];
    v.extend(limbs(Vec3::new(0.0, 1.4, 0.0), &ends, 0.14, 1));
    v.extend(clumps(
        &[
            ([1.6, 2.9, 0.4], [2.0, 1.5, 2.0]),
            ([-1.5, 3.0, -0.4], [2.1, 1.6, 2.1]),
            ([0.4, 3.1, -1.5], [1.9, 1.5, 1.9]),
            ([-0.5, 2.9, 1.5], [1.9, 1.4, 1.9]),
            ([0.0, 3.6, 0.0], [2.4, 1.8, 2.4]),
            ([1.0, 3.6, 1.0], [1.4, 1.1, 1.4]),
            ([-1.0, 3.7, -1.0], [1.4, 1.2, 1.4]),
            ([1.1, 3.5, -1.0], [1.3, 1.0, 1.3]),
            ([-1.1, 3.4, 1.0], [1.3, 1.0, 1.3]),
            ([0.2, 4.3, 0.2], [1.3, 1.0, 1.3]),
        ],
        0,
    ));
    // quelques jeunes feuilles qui percent au bord des fleurs
    v.extend(clumps(
        &[([2.3, 2.7, -0.3], [0.7, 0.5, 0.7]), ([-2.1, 2.8, 0.6], [0.7, 0.5, 0.7]), ([0.1, 2.8, 2.1], [0.7, 0.5, 0.7])],
        2,
    ));
    v
}

fn plane_tree() -> Vec<Piece> {
    // fût droit et couronne haute : 4 m dégagés sous les branches, au-dessus de la rue
    let mut v = vec![cyl(0.0, 0.0, 0.48, 0.0, 3.6, 1)];
    v.extend(limbs(
        Vec3::new(0.0, 3.4, 0.0),
        &[[1.0, 5.0, 0.3], [-0.9, 5.0, -0.3], [0.2, 5.0, -1.0], [-0.2, 5.0, 0.9]],
        0.22,
        1,
    ));
    v.extend(clumps(
        &[
            ([0.0, 6.0, 0.0], [4.6, 3.6, 4.6]),
            ([1.5, 5.3, 0.5], [2.6, 2.2, 2.6]),
            ([-1.5, 5.4, -0.5], [2.7, 2.3, 2.7]),
            ([0.5, 5.2, -1.5], [2.5, 2.1, 2.5]),
            ([-0.5, 5.3, 1.5], [2.5, 2.1, 2.5]),
            ([0.3, 7.6, 0.2], [3.0, 2.2, 3.0]),
        ],
        0,
    ));
    v
}

fn dead_tree() -> Vec<Piece> {
    let mut v = vec![
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.9, 0.6, 0.9), 0),
        cyl(0.0, 0.0, 0.42, 0.0, 2.4, 0),
        span(Shape::Cylinder, Vec3::new(0.0, 2.3, 0.0), Vec3::new(0.25, 4.4, -0.1), 0.26, 0),
    ];
    // une branche nue par étage, tournée de l'angle d'or, chacune fourchue en deux rameaux
    for k in 0..5 {
        let (a, y) = (k as f32 * 2.4 + 0.3, 1.8 + 0.5 * k as f32);
        let base = Vec3::new(0.05 * k as f32, y, 0.0);
        let tip = base + Vec3::new(a.sin() * 1.5, 1.1, a.cos() * 1.5);
        v.push(span(Shape::Cylinder, base, tip, 0.13 - 0.01 * k as f32, 0));
        for (da, up) in [(0.6, 0.7), (-0.7, 0.9)] {
            let b = a + da;
            v.push(span(Shape::Cylinder, tip, tip + Vec3::new(b.sin() * 0.7, up, b.cos() * 0.7), 0.06, 0));
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hauteur du bas de la plus basse pièce de feuillage (partie 0), boules non tournées.
    fn lowest_leaves(kind: PropKind) -> f32 {
        shapes(kind).iter().filter(|p| p.slot == 0).map(|p| p.at.y - p.size.y / 2.0).fold(f32::MAX, f32::min)
    }

    #[test]
    fn plane_tree_clears_the_street_and_willow_weeps_to_the_ground() {
        assert!(lowest_leaves(PropKind::PlaneTree) >= 4.0, "4 m libres sous le platane");
        assert!(lowest_leaves(PropKind::WeepingWillow) < 1.2, "les rideaux du saule descendent bas");
    }
    #[test]
    fn dead_tree_is_bare_wood() {
        assert_eq!(slots(PropKind::DeadTree).len(), 1);
        assert!(shapes(PropKind::DeadTree).iter().all(|p| p.shape == Shape::Cylinder || p.shape == Shape::Cone));
    }
}
