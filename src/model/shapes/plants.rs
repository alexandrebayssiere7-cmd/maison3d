//! Arbres, plantes et fleurs, un sous-module par catégorie. Feuillages et troncs sont blancs :
//! leur texture donne la couleur.

mod flora;
mod trees;

use bevy::math::Vec3;

use super::{Piece, ball, span};
use crate::model::{BARK, Category, PropKind, Shape, Slot};

const TRUNK: Slot = Slot::new("Tronc", BARK, "Écorce");
const LEAVES: Slot = Slot::textured("Feuillage", "Feuillage");
const LIGHT_LEAVES: Slot = Slot::textured("Feuillage", "Feuillage clair");

/// Charpentières : branches de section `d` qui partent du tronc en `from` vers chaque `ends`.
fn limbs(from: Vec3, ends: &[[f32; 3]], d: f32, slot: usize) -> Vec<Piece> {
    ends.iter().map(|&e| span(Shape::Cylinder, from, Vec3::from(e), d, slot)).collect()
}

/// Tronc tortueux : tronçons qui changent de direction d'un nœud au suivant, de diamètre `d`
/// aminci de `taper` à chaque tronçon, avec une boule à chaque coude.
fn gnarled(knots: &[Vec3], d: f32, taper: f32, slot: usize) -> Vec<Piece> {
    let segment = |(i, w): (usize, &[Vec3])| {
        let d = d - taper * i as f32;
        [span(Shape::Cylinder, w[0], w[1], d, slot), ball(w[1].to_array(), [d; 3], slot)]
    };
    knots.windows(2).enumerate().flat_map(segment).collect()
}

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    if kind.category() == Category::Trees { trees::slots(kind) } else { flora::slots(kind) }
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    if kind.category() == Category::Trees { trees::shapes(kind) } else { flora::shapes(kind) }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::Prop;

    #[test]
    fn plants_stand_on_the_ground_centred_on_their_footprint() {
        for k in PropKind::ALL.into_iter().filter(|k| matches!(k.category(), Category::Trees | Category::Plants)) {
            let (lo, hi) = Prop::new(k, Vec2::ZERO, 0, 0.0).local_box();
            assert!((-0.15..0.02).contains(&lo.y), "{k:?} flotte ou s'enfonce : {lo}");
            // centre de l'emprise à 10 % de sa taille près de l'origine
            let (c, s) = ((lo + hi) / 2.0, hi - lo);
            assert!(c.x.abs() < 0.1 * s.x && c.z.abs() < 0.1 * s.z, "{k:?} décentré : {c}");
        }
    }
}
