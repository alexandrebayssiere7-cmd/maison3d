//! Arbres, plantes et fleurs, un sous-module par catégorie. Feuillages et troncs sont blancs :
//! leur texture donne la couleur.

mod flora;
mod trees;

use super::Piece;
use crate::model::{BARK, Category, PropKind, Slot};

const TRUNK: Slot = Slot::new("Tronc", BARK, "Écorce");
const LEAVES: Slot = Slot::textured("Feuillage", "Feuillage");
const LIGHT_LEAVES: Slot = Slot::textured("Feuillage", "Feuillage clair");

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
