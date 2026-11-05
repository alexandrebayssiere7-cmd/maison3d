//! Annotations : cuisine et salle de bain.

use super::Placement::{AgainstWall, Floor, OnWall};
use super::Room::{Bathroom, Bedroom, Garage, Hall, Kitchen, Living};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Kitchen,
        note(
            AgainstWall,
            &[Kitchen],
            1.0,
            &[K::Fridge, K::Table],
            &[],
            &["cuisine", "bloc cuisine", "plan de travail", "évier", "plaques", "kitchen", "counter"],
        ),
    ),
    (
        K::Fridge,
        note(
            AgainstWall,
            &[Kitchen],
            0.9,
            &[K::Kitchen, K::BaseCabinet],
            &[],
            &["frigo", "réfrigérateur", "congélateur", "fridge", "refrigerator"],
        ),
    ),
    (
        K::KitchenIsland,
        note(
            Floor,
            &[Kitchen],
            1.0,
            &[K::BarStool, K::BaseCabinet],
            &[],
            &["îlot central", "îlot", "île de cuisine", "kitchen island"],
        ),
    ),
    (
        K::BaseCabinet,
        note(
            AgainstWall,
            &[Kitchen],
            1.0,
            &[K::WallCabinet, K::Range, K::Dishwasher],
            &[],
            &["meuble bas", "meuble de cuisine", "placard bas", "plan de travail", "base cabinet"],
        ),
    ),
    (
        K::WallCabinet,
        note(
            OnWall,
            &[Kitchen],
            0.0,
            &[K::BaseCabinet],
            &[],
            &["meuble haut", "placard haut", "meuble suspendu", "wall cabinet"],
        ),
    ),
    (
        K::Range,
        note(
            AgainstWall,
            &[Kitchen],
            1.0,
            &[K::RangeHood, K::BaseCabinet],
            &[],
            &["cuisinière", "four", "plaques de cuisson", "gazinière", "cooker", "stove", "range"],
        ),
    ),
    (
        K::OvenTower,
        note(
            AgainstWall,
            &[Kitchen],
            1.0,
            &[K::BaseCabinet, K::Fridge],
            &[],
            &["colonne four", "four encastré", "micro-ondes", "colonne", "oven"],
        ),
    ),
    (
        K::Dishwasher,
        note(
            AgainstWall,
            &[Kitchen],
            1.0,
            &[K::BaseCabinet, K::Kitchen],
            &[],
            &["lave-vaisselle", "vaisselle", "dishwasher"],
        ),
    ),
    (
        K::RangeHood,
        note(
            OnWall,
            &[Kitchen],
            0.0,
            &[K::Range],
            &[],
            &["hotte", "hotte aspirante", "extracteur", "range hood", "cooker hood"],
        ),
    ),
    (
        K::BarStool,
        note(
            Floor,
            &[Kitchen, Living],
            0.4,
            &[K::KitchenIsland, K::HighTable],
            &[],
            &["tabouret de bar", "tabouret", "bar stool", "stool"],
        ),
    ),
    (
        K::KitchenBin,
        note(
            AgainstWall,
            &[Kitchen],
            0.4,
            &[K::Kitchen],
            &[],
            &["poubelle de cuisine", "poubelle", "corbeille", "bin", "trash"],
        ),
    ),
    (
        K::HighTable,
        note(
            Floor,
            &[Kitchen, Living],
            0.6,
            &[K::BarStool],
            &[],
            &["mange-debout", "table haute", "table de bar", "high table"],
        ),
    ),
    (
        K::Bathtub,
        note(AgainstWall, &[Bathroom], 0.7, &[K::Toilet, K::Washbasin], &[], &["baignoire", "bain", "bathtub"]),
    ),
    (K::Toilet, note(AgainstWall, &[Bathroom], 0.6, &[K::Washbasin], &[], &["wc", "toilettes", "toilet"])),
    (
        K::WalkInShower,
        note(
            AgainstWall,
            &[Bathroom],
            0.8,
            &[K::VanityUnit, K::TowelRadiator],
            &[],
            &["douche à l'italienne", "douche", "douche italienne", "walk-in shower", "shower"],
        ),
    ),
    (
        K::ShowerCabin,
        note(
            AgainstWall,
            &[Bathroom],
            0.8,
            &[K::Washbasin, K::TowelRadiator],
            &[],
            &["cabine de douche", "douche", "cabine", "shower cabin", "shower"],
        ),
    ),
    (
        K::Washbasin,
        note(
            AgainstWall,
            &[Bathroom],
            0.7,
            &[K::Mirror, K::Toilet],
            &[],
            &["lavabo sur colonne", "lavabo", "évier de salle de bain", "washbasin", "sink"],
        ),
    ),
    (
        K::VanityUnit,
        note(
            AgainstWall,
            &[Bathroom],
            0.8,
            &[K::Mirror, K::Bathtub],
            &[],
            &["meuble vasque", "vasque", "meuble de salle de bain", "vanity unit", "sink"],
        ),
    ),
    (
        K::Mirror,
        note(
            OnWall,
            &[Bathroom, Bedroom, Hall],
            0.0,
            &[K::Washbasin, K::Dresser],
            &[],
            &["miroir mural", "glace", "miroir", "mirror"],
        ),
    ),
    (
        K::TowelRadiator,
        note(
            OnWall,
            &[Bathroom],
            0.3,
            &[K::WalkInShower, K::Bathtub],
            &[],
            &["sèche-serviettes", "radiateur", "serviettes", "towel radiator", "towel rail"],
        ),
    ),
    (
        K::WashingMachine,
        note(
            AgainstWall,
            &[Bathroom, Garage],
            0.9,
            &[K::Hamper],
            &[],
            &["lave-linge", "machine à laver", "buanderie", "washing machine", "washer"],
        ),
    ),
    (
        K::Hamper,
        note(
            AgainstWall,
            &[Bathroom, Bedroom],
            0.3,
            &[K::WashingMachine],
            &[],
            &["panier à linge", "linge sale", "corbeille à linge", "hamper", "laundry basket"],
        ),
    ),
];
