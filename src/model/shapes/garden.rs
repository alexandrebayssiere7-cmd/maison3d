//! Jardin, terrasse et jeux.

use super::{Piece, bx, clumps};
use crate::model::{PropKind, Slot};

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Hedge => const { &[Slot::textured("Feuillage", "Feuillage")] },
        PropKind::Fence => const { &[Slot::plain("Bois", [0.6, 0.45, 0.3])] },
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Hedge => {
            v.push(bx(0.0, 0.0, 2.0, 0.7, 0.0, 1.2, 0));
            v.extend(clumps(
                &[
                    ([-0.6, 1.2, 0.0], [0.9, 0.45, 0.75]),
                    ([0.0, 1.25, 0.0], [0.95, 0.45, 0.75]),
                    ([0.6, 1.2, 0.0], [0.9, 0.45, 0.75]),
                ],
                0,
            ));
        }
        PropKind::Fence => {
            for x in [-1.0, 0.0, 1.0] {
                v.push(bx(x, 0.0, 0.08, 0.08, 0.0, 1.0, 0));
            }
            v.push(bx(0.0, 0.0, 2.0, 0.04, 0.35, 0.45, 0));
            v.push(bx(0.0, 0.0, 2.0, 0.04, 0.75, 0.85, 0));
        }
        _ => {}
    }
    v
}
