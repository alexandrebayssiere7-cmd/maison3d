//! Magasin et supermarché.

use super::{Piece, bx};
use crate::model::{PropKind, Slot};

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Shelf => {
            const {
                &[
                    Slot::plain("Montants et tablettes", [0.72, 0.56, 0.4]),
                    Slot::plain("Produits rouges", [0.8, 0.2, 0.2]),
                    Slot::plain("Produits bleus", [0.2, 0.5, 0.8]),
                    Slot::plain("Produits jaunes", [0.9, 0.7, 0.1]),
                    Slot::plain("Produits verts", [0.3, 0.7, 0.3]),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    if kind == PropKind::Shelf {
        shelf(&mut v);
    }
    v
}

fn shelf(v: &mut Vec<Piece>) {
    for x in [-0.49, 0.49] {
        v.push(bx(x, 0.0, 0.02, 0.4, 0.0, 1.8, 0));
    }
    v.push(bx(0.0, -0.19, 1.0, 0.02, 0.0, 1.8, 0));
    for k in 0..5 {
        let y = 0.08 + k as f32 * 0.42;
        v.push(bx(0.0, 0.0, 0.96, 0.38, y, y + 0.025, 0));
    }
    // marchandises colorées sur les tablettes
    for k in 0..4 {
        let y = 0.105 + k as f32 * 0.42;
        v.push(bx(-0.2 + (k % 2) as f32 * 0.4, 0.02, 0.4, 0.28, y, y + 0.25, 1 + k));
    }
}
