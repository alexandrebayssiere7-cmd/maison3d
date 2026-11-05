//! Annotations : véhicules.

use super::Need::Road;
use super::Placement::Floor;
use super::Room::{Garage, Garden, Street};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Car,
        note(
            Floor,
            &[Street, Garage],
            0.0,
            &[K::ParkingSpot],
            &[Road],
            &["voiture", "auto", "automobile", "berline", "car"],
        ),
    ),
    (
        K::Motorcycle,
        note(
            Floor,
            &[Street, Garage],
            0.0,
            &[K::ParkingSpot],
            &[Road],
            &["moto", "deux-roues", "motorcycle", "motorbike"],
        ),
    ),
    (
        K::Bicycle,
        note(
            Floor,
            &[Street, Garage, Garden],
            0.0,
            &[K::BikeRack],
            &[],
            &["vélo", "bicyclette", "deux-roues", "bicycle", "bike"],
        ),
    ),
    (K::Van, note(Floor, &[Street], 0.0, &[K::ParkingSpot], &[Road], &["camionnette", "utilitaire", "fourgon", "van"])),
    (K::Bus, note(Floor, &[Street], 0.0, &[K::BusStop], &[Road], &["bus", "autobus", "transport en commun"])),
    (
        K::Truck,
        note(Floor, &[Street], 0.0, &[K::Dumpster], &[Road], &["camion", "poids lourd", "livraison", "truck", "lorry"]),
    ),
];
