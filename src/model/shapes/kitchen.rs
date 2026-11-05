//! Cuisine et salle de bain.

use super::{Piece, bx, carcass, door};
use crate::model::{DARK, METAL, PropKind, Slot, WHITE};

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Kitchen => {
            const {
                &[
                    Slot::plain("Façades", [0.88, 0.88, 0.85]),
                    Slot::plain("Plan de travail", DARK),
                    Slot::plain("Évier et poignées", METAL),
                    Slot::plain("Plaques", [0.05, 0.05, 0.05]),
                    Slot::plain("Vaisselle", WHITE),
                    Slot::plain("Casseroles", [0.6, 0.3, 0.2]),
                ]
            }
        }
        PropKind::Fridge => {
            const {
                &[
                    Slot::plain("Carrosserie", WHITE),
                    Slot::plain("Poignée", METAL),
                    Slot::plain("Intérieur", WHITE),
                    Slot::plain("Clayettes", [0.8, 0.9, 0.95]),
                    Slot::plain("Jus", [0.9, 0.5, 0.1]),
                    Slot::plain("Fromage", [0.95, 0.9, 0.6]),
                    Slot::plain("Légumes", [0.3, 0.6, 0.3]),
                ]
            }
        }
        PropKind::Bathtub => const { &[Slot::plain("Émail", WHITE), Slot::plain("Eau", [0.55, 0.75, 0.9])] },
        PropKind::Toilet => const { &[Slot::plain("Céramique", WHITE)] },
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Kitchen => kitchen(&mut v, open),
        PropKind::Fridge => fridge(&mut v, open),
        PropKind::Bathtub => {
            v.push(bx(0.0, 0.0, 0.75, 1.7, 0.0, 0.55, 0));
            v.push(bx(0.0, 0.0, 0.6, 1.5, 0.5, 0.56, 1));
        }
        PropKind::Toilet => {
            v.push(bx(0.0, 0.05, 0.4, 0.5, 0.0, 0.42, 0));
            v.push(bx(0.0, -0.22, 0.4, 0.18, 0.42, 0.8, 0));
        }
        _ => {}
    }
    v
}

fn kitchen(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(2.4, 0.6, 0.86, 0, 0));
    v.push(bx(0.0, 0.0, 2.35, 0.55, 0.42, 0.44, 0));
    v.push(bx(0.0, 0.0, 2.42, 0.62, 0.86, 0.9, 1));
    v.push(bx(0.5, 0.0, 0.6, 0.4, 0.9, 0.91, 2));
    v.push(bx(-0.6, 0.0, 0.6, 0.5, 0.9, 0.91, 3));
    // vaisselle
    v.push(bx(-0.8, 0.0, 0.3, 0.3, 0.44, 0.56, 4));
    v.push(bx(0.3, -0.05, 0.4, 0.25, 0.08, 0.3, 5));
    for (hx, side) in [(-1.2, 1.0), (0.0, -1.0), (0.0, 1.0), (1.2, -1.0)] {
        v.extend(door(open, (hx, 0.3), 0.6, (0.06, 0.84), side, (0, 2)));
    }
}

fn fridge(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.7, 0.7, 1.85, 0, 2));
    for y in [0.5, 0.9, 1.3] {
        v.push(bx(0.0, 0.0, 0.64, 0.6, y, y + 0.02, 3));
    }
    v.push(bx(-0.15, 0.0, 0.15, 0.15, 0.92, 1.1, 4));
    v.push(bx(0.12, 0.05, 0.25, 0.12, 1.32, 1.4, 5));
    v.push(bx(0.0, 0.0, 0.2, 0.15, 0.52, 0.75, 6));
    v.extend(door(open, (-0.35, 0.35), 0.7, (0.02, 1.84), 1.0, (0, 1)));
}
