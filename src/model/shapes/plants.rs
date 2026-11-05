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
