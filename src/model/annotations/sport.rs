//! Annotations : sport et terrains.

use super::Placement::{Floor, Row};
use super::Room::{Garage, Garden, Sport};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::SoccerGoal,
        note(
            Floor,
            &[Sport],
            0.0,
            &[K::SoccerPitch, K::CornerFlag],
            &[],
            &["cage de foot", "but", "football", "soccer goal"],
        ),
    ),
    (
        K::HandballGoal,
        note(
            Floor,
            &[Sport],
            0.0,
            &[K::HandballCourt, K::SoccerPitch5],
            &[],
            &["but de hand et futsal", "but", "futsal", "handball goal"],
        ),
    ),
    (
        K::SoccerBall,
        note(
            Floor,
            &[Sport, Garden],
            0.0,
            &[K::SoccerGoal],
            &[],
            &["ballon de foot", "ballon", "football", "soccer ball"],
        ),
    ),
    (
        K::Basketball,
        note(
            Floor,
            &[Sport, Garden],
            0.0,
            &[K::BasketHoop],
            &[],
            &["ballon de basket", "ballon", "basket", "basketball"],
        ),
    ),
    (
        K::BasketHoop,
        note(
            Floor,
            &[Sport, Garden],
            0.0,
            &[K::BasketCourt, K::Basketball],
            &[],
            &["panier de basket", "panier", "basket", "basketball hoop"],
        ),
    ),
    (
        K::TennisNet,
        note(Floor, &[Sport], 0.0, &[K::TennisCourt], &[], &["filet de tennis", "filet", "tennis", "tennis net"]),
    ),
    (
        K::VolleyNet,
        note(
            Floor,
            &[Sport, Garden],
            0.0,
            &[K::VolleyCourt],
            &[],
            &["filet de volley", "filet", "volley", "volleyball net"],
        ),
    ),
    (
        K::RugbyPosts,
        note(Floor, &[Sport], 0.0, &[K::SoccerPitch], &[], &["poteaux de rugby", "rugby", "poteaux", "rugby posts"]),
    ),
    (
        K::PingPongTable,
        note(
            Floor,
            &[Sport, Garage, Garden],
            2.0,
            &[],
            &[],
            &["table de ping-pong", "tennis de table", "ping-pong", "ping pong table"],
        ),
    ),
    (
        K::Bleachers,
        note(
            Floor,
            &[Sport],
            1.0,
            &[K::Floodlight, K::Scoreboard],
            &[],
            &["gradins", "tribune", "spectateurs", "bleachers", "stands"],
        ),
    ),
    (
        K::DugoutBench,
        note(Floor, &[Sport], 1.0, &[K::SoccerPitch], &[], &["banc de touche", "banc", "abri de touche", "dugout"]),
    ),
    (
        K::CornerFlag,
        note(Floor, &[Sport], 0.0, &[K::SoccerPitch], &[], &["drapeau de corner", "drapeau", "corner", "corner flag"]),
    ),
    (
        K::Floodlight,
        note(
            Floor,
            &[Sport],
            0.0,
            &[K::Bleachers],
            &[],
            &["projecteur de stade", "projecteur", "éclairage de stade", "floodlight"],
        ),
    ),
    (
        K::Scoreboard,
        note(Floor, &[Sport], 0.0, &[K::Bleachers], &[], &["tableau d'affichage", "score", "affichage", "scoreboard"]),
    ),
    (
        K::WeightBench,
        note(
            Floor,
            &[Sport, Garage],
            1.0,
            &[K::Treadmill, K::PunchingBag],
            &[],
            &["banc de musculation", "musculation", "fitness", "weight bench", "gym"],
        ),
    ),
    (
        K::Treadmill,
        note(
            Floor,
            &[Sport, Garage],
            1.0,
            &[K::WeightBench],
            &[],
            &["tapis de course", "course", "fitness", "treadmill", "gym"],
        ),
    ),
    (
        K::PunchingBag,
        note(Floor, &[Sport, Garage], 1.0, &[K::WeightBench], &[], &["sac de frappe", "boxe", "punching bag", "gym"]),
    ),
    (K::Hurdle, note(Row, &[Sport], 0.0, &[K::FieldLine], &[], &["haie d'athlétisme", "athlétisme", "hurdle"])),
    (
        K::FieldLine,
        note(Floor, &[Sport], 0.0, &[K::FieldArc], &[], &["ligne", "ligne de terrain", "marquage", "field line"]),
    ),
    (K::FieldArc, note(Floor, &[Sport], 0.0, &[K::FieldLine], &[], &["demi-cercle", "arc", "marquage", "field arc"])),
    (
        K::FieldQuarterArc,
        note(Floor, &[Sport], 0.0, &[K::CornerFlag], &[], &["quart de cercle", "arc de corner", "marquage"]),
    ),
    (
        K::FieldCircle,
        note(Floor, &[Sport], 0.0, &[K::FieldDot], &[], &["cercle", "rond central", "marquage", "center circle"]),
    ),
    (
        K::FieldDot,
        note(Floor, &[Sport], 0.0, &[K::FieldCircle], &[], &["point", "point de penalty", "point central", "marquage"]),
    ),
    (
        K::SoccerPitch5,
        note(
            Floor,
            &[Sport],
            1.0,
            &[K::HandballGoal],
            &[],
            &["terrain de foot à 5", "five", "futsal", "foot à 5", "small soccer pitch"],
        ),
    ),
    (
        K::SoccerPitch,
        note(
            Floor,
            &[Sport],
            2.0,
            &[K::SoccerGoal, K::CornerFlag, K::Bleachers],
            &[],
            &["terrain de foot", "stade", "football", "soccer field", "pitch"],
        ),
    ),
    (
        K::BasketCourt,
        note(Floor, &[Sport], 2.0, &[K::BasketHoop], &[], &["terrain de basket", "basket", "basketball court"]),
    ),
    (K::TennisCourt, note(Floor, &[Sport], 3.0, &[K::TennisNet], &[], &["court de tennis", "tennis", "tennis court"])),
    (
        K::VolleyCourt,
        note(Floor, &[Sport], 2.0, &[K::VolleyNet], &[], &["terrain de volley", "volley", "volleyball court"]),
    ),
    (
        K::HandballCourt,
        note(Floor, &[Sport], 2.0, &[K::HandballGoal], &[], &["terrain de hand", "handball", "handball court"]),
    ),
];
