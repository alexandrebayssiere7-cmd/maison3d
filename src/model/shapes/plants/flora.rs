//! Plantes et fleurs. Les feuillages sont blancs : leur texture donne la couleur.

use super::LIGHT_LEAVES;
use crate::model::shapes::{Piece, ball, bx, clumps, cyl};
use crate::model::{LEAF, PropKind, Slot};

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Plant => const { &[LIGHT_LEAVES, Slot::plain("Pot", [0.7, 0.35, 0.2])] },
        PropKind::Bush => const { &[LIGHT_LEAVES] },
        PropKind::Flowers => {
            const {
                &[
                    Slot::plain("Fleurs", [0.85, 0.15, 0.25]),
                    Slot::plain("Fleurs blanches", [0.97, 0.97, 0.95]),
                    Slot::plain("Fleurs jaunes", [0.98, 0.85, 0.2]),
                    Slot::plain("Tiges et feuilles", LEAF),
                    Slot::plain("Terre", [0.35, 0.24, 0.15]),
                ]
            }
        }
        PropKind::Rosebush => const { &[Slot::plain("Roses", [0.95, 0.45, 0.6]), Slot::plain("Feuillage", LEAF)] },
        PropKind::Sunflower => {
            const {
                &[
                    Slot::plain("Pétales", [0.98, 0.8, 0.1]),
                    Slot::plain("Tiges et feuilles", LEAF),
                    Slot::plain("Cœurs", [0.35, 0.2, 0.08]),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind) -> Vec<Piece> {
    match kind {
        PropKind::Plant => {
            let mut v = vec![cyl(0.0, 0.0, 0.35, 0.0, 0.4, 1)];
            v.extend(clumps(
                &[
                    ([0.0, 0.62, 0.0], [0.5, 0.45, 0.5]),
                    ([0.08, 0.8, 0.05], [0.35, 0.35, 0.35]),
                    ([-0.1, 0.72, -0.06], [0.32, 0.3, 0.32]),
                ],
                0,
            ));
            v
        }
        PropKind::Bush => clumps(
            &[
                ([0.0, 0.45, 0.0], [1.1, 0.9, 1.1]),
                ([0.4, 0.4, 0.3], [0.8, 0.75, 0.8]),
                ([-0.35, 0.38, -0.25], [0.85, 0.7, 0.8]),
                ([-0.2, 0.42, 0.4], [0.7, 0.65, 0.7]),
                ([0.1, 0.72, -0.1], [0.7, 0.6, 0.7]),
            ],
            0,
        ),
        PropKind::Flowers => flowers(),
        PropKind::Rosebush => rosebush(),
        PropKind::Sunflower => sunflower(),
        _ => Vec::new(),
    }
}

fn flowers() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 1.3, 0.9, 0.0, 0.1, 4)];
    for k in 0..15 {
        let jitter = ((k * 37) % 7) as f32 * 0.02;
        let (x, z) = ((k % 5) as f32 * 0.24 - 0.48 + jitter, (k / 5) as f32 * 0.26 - 0.26 - jitter);
        let h = 0.35 + jitter * 2.0;
        v.push(cyl(x, z, 0.03, 0.0, h, 3));
        v.push(ball([x, h, z], [0.14, 0.08, 0.14], flower_head(k)));
    }
    v.push(ball([0.0, 0.12, 0.0], [1.2, 0.25, 0.8], 3));
    v
}

/// Partie du cœur de la k-ième fleur d'un massif : rouge, blanche ou jaune.
fn flower_head(k: usize) -> usize {
    match k % 4 {
        3 => 2,
        1 => 1,
        _ => 0,
    }
}

fn rosebush() -> Vec<Piece> {
    let mut v = clumps(
        &[
            ([0.0, 0.45, 0.0], [1.0, 0.9, 1.0]),
            ([0.3, 0.6, 0.2], [0.7, 0.7, 0.7]),
            ([-0.3, 0.55, -0.2], [0.7, 0.7, 0.7]),
        ],
        1,
    );
    for k in 0..12 {
        let (a, e) = (k as f32 * 2.4, [-0.2_f32, 0.3, 0.7][k % 3]);
        v.push(ball([0.5 * e.cos() * a.cos(), 0.45 + 0.45 * e.sin(), -0.5 * e.cos() * a.sin()], [0.15; 3], 0));
    }
    v
}

fn sunflower() -> Vec<Piece> {
    let mut v = Vec::new();
    for (x, z, h) in [(0.0, 0.0, 1.7), (0.35, 0.2, 1.4), (-0.3, 0.25, 1.55), (0.1, -0.3, 1.25)] {
        v.push(cyl(x, z, 0.04, 0.0, h, 1));
        v.push(ball([x, h * 0.55, z + 0.1], [0.28, 0.04, 0.16], 1));
        v.push(cyl(x, z, 0.45, h - 0.02, h + 0.02, 0));
        v.push(cyl(x, z, 0.22, h, h + 0.04, 2));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flower_heads_mix_three_colors() {
        assert_eq!([flower_head(0), flower_head(1), flower_head(3)], [0, 1, 2]);
    }
}
