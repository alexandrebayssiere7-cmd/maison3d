//! Annotations : magasin.

use super::Placement::{AgainstWall, Ceiling, Floor};
use super::Room::{Bedroom, Garage, Living, Office, Store, Street};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Shelf,
        note(
            AgainstWall,
            &[Store, Office, Living, Garage],
            1.2,
            &[K::Gondola],
            &[],
            &["étagère", "rayonnage", "rayon", "bibliothèque", "shelf", "shelving"],
        ),
    ),
    (
        K::Checkout,
        note(
            Floor,
            &[Store],
            1.2,
            &[K::ShoppingCart, K::BasketStack],
            &[],
            &["caisse", "tapis de caisse", "checkout", "cash register"],
        ),
    ),
    (
        K::FridgeCase,
        note(
            AgainstWall,
            &[Store],
            1.5,
            &[K::FreezerCase],
            &[],
            &["meuble réfrigéré", "produits frais", "vitrine réfrigérée", "fridge case"],
        ),
    ),
    (
        K::FreezerCase,
        note(
            AgainstWall,
            &[Store],
            1.5,
            &[K::FridgeCase],
            &[],
            &["congélateur vitré", "surgelés", "congélateur", "freezer case"],
        ),
    ),
    (
        K::ProduceStand,
        note(
            Floor,
            &[Store],
            1.2,
            &[K::ShoppingCart],
            &[],
            &["étal de fruits et légumes", "fruits et légumes", "primeur", "étal", "produce stand"],
        ),
    ),
    (
        K::ShoppingCart,
        note(
            Floor,
            &[Store, Street],
            0.0,
            &[K::Checkout],
            &[],
            &["chariot", "caddie", "chariot de courses", "shopping cart", "trolley"],
        ),
    ),
    (
        K::BasketStack,
        note(
            Floor,
            &[Store],
            0.0,
            &[K::Checkout],
            &[],
            &["pile de paniers", "paniers", "panier de courses", "baskets"],
        ),
    ),
    (
        K::Gondola,
        note(
            Floor,
            &[Store],
            1.5,
            &[K::AisleSign, K::Shelf],
            &[],
            &["gondole", "rayon", "étagère double", "gondola", "aisle"],
        ),
    ),
    (
        K::Pallet,
        note(
            Floor,
            &[Store, Garage],
            1.0,
            &[K::Gondola],
            &[],
            &["palette de marchandises", "palette", "cartons", "réserve", "pallet"],
        ),
    ),
    (
        K::AisleSign,
        note(
            Ceiling,
            &[Store],
            0.0,
            &[K::Gondola],
            &[],
            &["panneau de rayon", "signalétique", "enseigne de rayon", "aisle sign"],
        ),
    ),
    (
        K::DisplayCase,
        note(
            AgainstWall,
            &[Store],
            1.2,
            &[K::Checkout],
            &[],
            &["vitrine de boulangerie", "boulangerie", "pâtisserie", "vitrine", "display case"],
        ),
    ),
    (
        K::SecurityGate,
        note(Floor, &[Store], 1.0, &[K::Checkout], &[], &["portique antivol", "antivol", "portique", "security gate"]),
    ),
    (
        K::ClothesRack,
        note(
            Floor,
            &[Store, Bedroom],
            0.8,
            &[K::Mannequin],
            &[],
            &["portant de vêtements", "vêtements", "portant", "boutique", "clothes rack"],
        ),
    ),
    (
        K::Mannequin,
        note(Floor, &[Store], 0.5, &[K::ClothesRack], &[], &["mannequin", "vitrine", "boutique de vêtements"]),
    ),
];
