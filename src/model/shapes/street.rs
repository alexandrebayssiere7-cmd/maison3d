//! Mobilier urbain et signalisation.

use bevy::math::Vec3;

use super::{Piece, bx, cyl, light, span};
use crate::model::{PropKind, Shape, Slot};

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::StreetLamp => {
            const { &[Slot::plain("Mât", [0.2, 0.22, 0.25]), Slot::plain("Lanterne et lumière", [1.0, 0.95, 0.8])] }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    if kind == PropKind::StreetLamp {
        street_lamp(&mut v);
    }
    v
}

fn street_lamp(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, 0.0, 0.3, 0.0, 0.3, 0));
    v.push(cyl(0.0, 0.0, 0.12, 0.1, 4.5, 0));
    // bras qui monte légèrement du mât vers la lanterne
    v.push(span(Shape::Cube, Vec3::new(0.0, 4.32, 0.0), Vec3::new(0.0, 4.47, 0.6), 0.08, 0));
    v.push(bx(0.0, 0.75, 0.35, 0.45, 4.25, 4.45, 0));
    v.push(bx(0.0, 0.75, 0.3, 0.4, 4.2, 4.25, 1));
    v.push(light([0.0, 4.0, 0.75], 1));
}
