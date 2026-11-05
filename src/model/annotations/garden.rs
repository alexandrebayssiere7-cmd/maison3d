//! Annotations : jardin.

use super::Placement::{AgainstWall, Floor, Row};
use super::Room::{Garage, Garden, Street};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (K::Hedge, note(Row, &[Garden, Street], 0.0, &[K::Fence], &[], &["haie", "bordure", "hedge"])),
    (
        K::Fence,
        note(
            Row,
            &[Garden, Street],
            0.0,
            &[K::Hedge, K::Gate],
            &[],
            &["clôture", "barrière", "palissade", "grillage", "fence"],
        ),
    ),
    (
        K::PicketFence,
        note(Row, &[Garden], 0.0, &[K::Gate], &[], &["clôture blanche", "clôture", "barrière blanche", "picket fence"]),
    ),
    (
        K::LowWall,
        note(
            Row,
            &[Garden, Street],
            0.0,
            &[K::Gate],
            &[],
            &["muret de pierre", "muret", "mur bas", "pierre", "low wall"],
        ),
    ),
    (
        K::Gate,
        note(Floor, &[Garden, Street], 2.0, &[K::Fence, K::LowWall], &[], &["portail", "entrée", "portillon", "gate"]),
    ),
    (
        K::GardenShed,
        note(
            Floor,
            &[Garden],
            1.5,
            &[K::Wheelbarrow, K::Woodpile],
            &[],
            &["abri de jardin", "cabane", "remise", "shed"],
        ),
    ),
    (
        K::Greenhouse,
        note(Floor, &[Garden], 1.0, &[K::VegetablePatch], &[], &["serre", "potager", "semis", "greenhouse"]),
    ),
    (
        K::Pergola,
        note(Floor, &[Garden], 0.0, &[K::GardenTable, K::LoungeSet], &[], &["pergola", "tonnelle", "terrasse"]),
    ),
    (K::Well, note(Floor, &[Garden], 1.0, &[K::Bush], &[], &["puits", "eau", "ancien", "well"])),
    (
        K::Fountain,
        note(Floor, &[Garden, Street], 1.0, &[K::Bench], &[], &["fontaine", "bassin", "jet d'eau", "fountain"]),
    ),
    (
        K::Pond,
        note(Floor, &[Garden], 0.0, &[K::Reeds, K::WeepingWillow], &[], &["mare", "bassin", "étang", "eau", "pond"]),
    ),
    (
        K::Planter,
        note(
            AgainstWall,
            &[Garden, Street],
            0.0,
            &[K::Flowers],
            &[],
            &["jardinière", "bac à fleurs", "pot", "planter"],
        ),
    ),
    (
        K::VegetablePatch,
        note(
            Floor,
            &[Garden],
            0.6,
            &[K::Greenhouse],
            &[],
            &["carré potager", "potager", "légumes", "tomates", "vegetable patch"],
        ),
    ),
    (
        K::SteppingStones,
        note(Floor, &[Garden], 0.0, &[K::Pond], &[], &["pas japonais", "chemin", "dalles", "stepping stones"]),
    ),
    (
        K::StoneLantern,
        note(
            Floor,
            &[Garden],
            0.0,
            &[K::CherryBlossom, K::Pond],
            &[],
            &["lanterne japonaise", "tōrō", "toro", "jardin japonais", "stone lantern"],
        ),
    ),
    (
        K::Torii,
        note(Floor, &[Garden], 2.0, &[K::StoneLantern], &[], &["torii", "portique japonais", "jardin japonais"]),
    ),
    (
        K::Woodpile,
        note(
            AgainstWall,
            &[Garden, Garage],
            0.0,
            &[K::GardenShed],
            &[],
            &["tas de bois", "bûches", "bois de chauffage", "woodpile"],
        ),
    ),
    (K::Wheelbarrow, note(Floor, &[Garden], 0.0, &[K::GardenShed], &[], &["brouette", "jardinage", "wheelbarrow"])),
    (K::GardenGnome, note(Floor, &[Garden], 0.0, &[K::Flowers], &[], &["nain de jardin", "nain", "gnome"])),
    (K::Doghouse, note(Floor, &[Garden], 1.0, &[K::Hedge], &[], &["niche", "chien", "chenil", "doghouse", "kennel"])),
    (K::Birdhouse, note(Floor, &[Garden], 0.0, &[K::Tree], &[], &["nichoir", "oiseaux", "birdhouse"])),
];
