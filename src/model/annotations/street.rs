//! Annotations : mobilier urbain.

use super::Placement::{AgainstWall, Floor, Row};
use super::Room::{Garage, Garden, Store, Street};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::StreetLamp,
        note(
            Row,
            &[Street],
            0.0,
            &[K::Bench],
            &[],
            &["lampadaire de rue", "lampadaire", "réverbère", "éclairage public", "street light", "street lamp"],
        ),
    ),
    (
        K::Lantern,
        note(
            Row,
            &[Street, Garden],
            0.0,
            &[K::Bench],
            &[],
            &["réverbère", "lampadaire ancien", "éclairage public", "parc", "lantern", "lamppost"],
        ),
    ),
    (
        K::Bench,
        note(
            Floor,
            &[Street, Garden],
            0.6,
            &[K::TrashCan, K::StreetLamp],
            &[],
            &["banc public", "banc", "assise", "parc", "bench"],
        ),
    ),
    (
        K::TrashCan,
        note(
            Floor,
            &[Street],
            0.3,
            &[K::Bench],
            &[],
            &["poubelle de rue", "poubelle", "corbeille", "trash can", "litter bin"],
        ),
    ),
    (
        K::RecyclingBins,
        note(
            AgainstWall,
            &[Street, Garage],
            0.8,
            &[K::Dumpster],
            &[],
            &["conteneurs de tri", "tri sélectif", "poubelles", "recycling bins"],
        ),
    ),
    (
        K::Dumpster,
        note(
            Floor,
            &[Street, Store],
            1.0,
            &[K::RecyclingBins],
            &[],
            &["benne à ordures", "benne", "conteneur", "poubelle", "dumpster"],
        ),
    ),
    (
        K::BusStop,
        note(
            Floor,
            &[Street],
            1.5,
            &[K::Bench, K::StreetLamp],
            &[],
            &["abribus", "arrêt de bus", "abri", "bus stop", "bus shelter"],
        ),
    ),
    (
        K::FireHydrant,
        note(
            Floor,
            &[Street],
            0.5,
            &[K::StreetLamp],
            &[],
            &["borne incendie", "bouche d'incendie", "pompiers", "fire hydrant"],
        ),
    ),
    (K::Bollard, note(Row, &[Street], 0.0, &[K::StreetLamp], &[], &["potelet", "plot", "borne", "bollard"])),
    (
        K::Mailbox,
        note(
            Floor,
            &[Street],
            0.6,
            &[K::StreetLamp],
            &[],
            &["boîte aux lettres", "poste", "courrier", "mailbox", "postbox"],
        ),
    ),
    (
        K::ParkingMeter,
        note(
            Floor,
            &[Street],
            0.5,
            &[K::ParkingSpot],
            &[],
            &["horodateur", "parcmètre", "stationnement payant", "parking meter"],
        ),
    ),
    (
        K::PhoneBooth,
        note(Floor, &[Street], 0.9, &[K::Mailbox], &[], &["cabine téléphonique", "téléphone", "cabine", "phone booth"]),
    ),
    (
        K::BikeRack,
        note(
            Floor,
            &[Street],
            1.0,
            &[K::Bicycle],
            &[],
            &["arceaux à vélos", "parking vélo", "support vélo", "bike rack"],
        ),
    ),
    (
        K::Billboard,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::StreetLamp],
            &[],
            &["panneau publicitaire", "affiche", "publicité", "billboard"],
        ),
    ),
    (
        K::UtilityPole,
        note(
            Row,
            &[Street],
            0.0,
            &[K::StreetLamp],
            &[],
            &["poteau électrique", "poteau", "ligne électrique", "utility pole", "power line"],
        ),
    ),
    (
        K::Manhole,
        note(Floor, &[Street], 0.0, &[K::Crosswalk], &[], &["plaque d'égout", "égout", "bouche d'égout", "manhole"]),
    ),
    (
        K::Kiosk,
        note(
            Floor,
            &[Street],
            1.2,
            &[K::Bench],
            &[],
            &["kiosque à journaux", "journaux", "presse", "kiosk", "newsstand"],
        ),
    ),
    (
        K::DrinkingFountain,
        note(
            Floor,
            &[Street, Garden],
            0.6,
            &[K::Bench],
            &[],
            &["fontaine à boire", "fontaine", "eau potable", "drinking fountain"],
        ),
    ),
];
