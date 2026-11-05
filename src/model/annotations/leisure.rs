//! Annotations : terrasse et jeux.

use super::Placement::Floor;
use super::Room::Garden;
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::GardenTable,
        note(
            Floor,
            &[Garden],
            0.8,
            &[K::GardenChair, K::Parasol],
            &[],
            &["table de jardin", "table d'extérieur", "terrasse", "garden table"],
        ),
    ),
    (
        K::GardenChair,
        note(
            Floor,
            &[Garden],
            0.5,
            &[K::GardenTable],
            &[],
            &["chaise de jardin", "chaise d'extérieur", "garden chair"],
        ),
    ),
    (
        K::Parasol,
        note(Floor, &[Garden], 0.0, &[K::GardenTable, K::Lounger], &[], &["parasol", "ombrelle", "ombre", "umbrella"]),
    ),
    (
        K::Lounger,
        note(
            Floor,
            &[Garden],
            0.5,
            &[K::Parasol],
            &[],
            &["transat", "bain de soleil", "chaise longue", "lounger", "sun lounger"],
        ),
    ),
    (
        K::LoungeSet,
        note(
            Floor,
            &[Garden],
            0.8,
            &[K::Parasol, K::Pergola],
            &[],
            &["salon de jardin", "canapé de jardin", "salon d'extérieur", "lounge set"],
        ),
    ),
    (K::Barbecue, note(Floor, &[Garden], 1.0, &[K::GardenTable], &[], &["barbecue", "bbq", "grill"])),
    (
        K::Firepit,
        note(Floor, &[Garden], 1.5, &[K::GardenChair, K::Woodpile], &[], &["brasero", "feu", "foyer", "fire pit"]),
    ),
    (K::Hammock, note(Floor, &[Garden], 1.0, &[K::Tree], &[], &["hamac", "repos", "hammock"])),
    (K::HotTub, note(Floor, &[Garden], 1.0, &[K::Lounger], &[], &["spa", "jacuzzi", "bain à remous", "hot tub"])),
    (
        K::Swing,
        note(
            Floor,
            &[Garden],
            2.0,
            &[K::Slide, K::Sandbox],
            &[],
            &["balançoire", "jeux d'enfants", "portique", "swing"],
        ),
    ),
    (K::Slide, note(Floor, &[Garden], 2.0, &[K::Swing], &[], &["toboggan", "jeux d'enfants", "slide"])),
    (K::Trampoline, note(Floor, &[Garden], 2.0, &[K::Swing], &[], &["trampoline", "jeux d'enfants"])),
    (K::Sandbox, note(Floor, &[Garden], 0.5, &[K::Swing], &[], &["bac à sable", "sable", "jeux d'enfants", "sandbox"])),
    (
        K::Seesaw,
        note(Floor, &[Garden], 1.0, &[K::Swing], &[], &["tape-cul", "bascule", "balançoire à bascule", "seesaw"]),
    ),
    (
        K::PlayHouse,
        note(
            Floor,
            &[Garden],
            1.0,
            &[K::Sandbox],
            &[],
            &["cabane d'enfant", "cabane", "maison de jardin", "playhouse"],
        ),
    ),
];
