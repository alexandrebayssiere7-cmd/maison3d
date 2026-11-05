//! Annotations : garage et atelier.

use super::Placement::{AgainstWall, Floor};
use super::Room::{Bathroom, Garage, Garden, Store};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Workbench,
        note(
            AgainstWall,
            &[Garage],
            1.0,
            &[K::ToolChest, K::MetalShelf],
            &[],
            &["établi", "atelier", "bricolage", "workbench"],
        ),
    ),
    (
        K::ToolChest,
        note(
            AgainstWall,
            &[Garage],
            0.8,
            &[K::Workbench],
            &[],
            &["servante d'atelier", "outils", "caisse à outils", "tool chest"],
        ),
    ),
    (
        K::MetalShelf,
        note(
            AgainstWall,
            &[Garage, Store],
            0.9,
            &[K::Boxes],
            &[],
            &["rayonnage métallique", "étagère métallique", "rangement", "metal shelf", "shelving"],
        ),
    ),
    (
        K::TireStack,
        note(AgainstWall, &[Garage], 0.0, &[K::CarLift], &[], &["pile de pneus", "pneus", "roues", "tires"]),
    ),
    (
        K::JerryCans,
        note(AgainstWall, &[Garage], 0.0, &[K::Lawnmower], &[], &["bidons", "jerricans", "essence", "jerry cans"]),
    ),
    (
        K::WaterHeater,
        note(
            AgainstWall,
            &[Garage, Bathroom],
            0.6,
            &[K::WashingMachine],
            &[],
            &["chauffe-eau", "ballon d'eau chaude", "cumulus", "water heater", "boiler"],
        ),
    ),
    (
        K::Lawnmower,
        note(
            AgainstWall,
            &[Garage, Garden],
            0.0,
            &[K::JerryCans],
            &[],
            &["tondeuse", "tondeuse à gazon", "pelouse", "lawnmower"],
        ),
    ),
    (K::Stepladder, note(AgainstWall, &[Garage], 0.0, &[K::Workbench], &[], &["escabeau", "échelle", "stepladder"])),
    (K::Boxes, note(AgainstWall, &[Garage, Store], 0.0, &[K::MetalShelf], &[], &["cartons", "déménagement", "boxes"])),
    (
        K::ChestFreezer,
        note(AgainstWall, &[Garage], 0.8, &[K::MetalShelf], &[], &["congélateur coffre", "congélateur", "freezer"]),
    ),
    (
        K::CarLift,
        note(
            Floor,
            &[Garage],
            1.5,
            &[K::Car, K::TireStack],
            &[],
            &["pont élévateur", "garage auto", "élévateur", "car lift"],
        ),
    ),
];
