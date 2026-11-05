//! Annotations : signalisation et marquages routiers.

use super::Need::Road;
use super::Placement::{Floor, Row};
use super::Room::Street;
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::TrafficLight,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::Crosswalk, K::PedestrianSign],
            &[Road],
            &["feu tricolore", "feu rouge", "feux", "signalisation", "traffic light"],
        ),
    ),
    (
        K::StopSign,
        note(Floor, &[Street], 0.0, &[K::RoadDash], &[Road], &["panneau stop", "stop", "panneau", "stop sign"]),
    ),
    (
        K::YieldSign,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::RoadDash],
            &[Road],
            &["cédez le passage", "priorité", "panneau", "yield sign", "give way"],
        ),
    ),
    (
        K::NoEntrySign,
        note(Floor, &[Street], 0.0, &[K::StopSign], &[Road], &["sens interdit", "interdit", "panneau", "no entry"]),
    ),
    (
        K::SpeedLimitSign,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::SpeedBump],
            &[Road],
            &["limitation de vitesse", "vitesse", "50", "panneau", "speed limit"],
        ),
    ),
    (
        K::PedestrianSign,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::Crosswalk],
            &[Road],
            &["panneau passage piéton", "piétons", "panneau", "pedestrian sign"],
        ),
    ),
    (
        K::Crosswalk,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::PedestrianSign, K::TrafficLight],
            &[Road],
            &["passage piéton", "zébra", "bandes blanches", "crosswalk", "zebra crossing"],
        ),
    ),
    (
        K::RoadDash,
        note(
            Row,
            &[Street],
            0.0,
            &[K::Crosswalk],
            &[Road],
            &["ligne discontinue", "marquage", "ligne blanche", "pointillés", "road marking"],
        ),
    ),
    (
        K::ParkingSpot,
        note(
            Row,
            &[Street],
            0.0,
            &[K::Car, K::ParkingMeter],
            &[Road],
            &["place de parking", "parking", "stationnement", "emplacement", "parking space"],
        ),
    ),
    (
        K::TrafficCone,
        note(
            Row,
            &[Street],
            0.0,
            &[K::WorkBarrier],
            &[],
            &["cône de chantier", "plot de chantier", "travaux", "traffic cone"],
        ),
    ),
    (
        K::WorkBarrier,
        note(
            Row,
            &[Street],
            0.0,
            &[K::TrafficCone],
            &[],
            &["barrière de chantier", "travaux", "chantier", "barrière", "road barrier"],
        ),
    ),
    (
        K::SpeedBump,
        note(
            Floor,
            &[Street],
            0.0,
            &[K::SpeedLimitSign],
            &[Road],
            &["ralentisseur", "dos d'âne", "gendarme couché", "speed bump"],
        ),
    ),
];
