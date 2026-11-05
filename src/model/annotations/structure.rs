//! Annotations : escaliers et structure.

use super::Need::StairOpening;
use super::Placement::{AgainstWall, Floor, Row};
use super::Room::{Garage, Garden, Hall, Living, Store, Street};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Stairs,
        note(
            AgainstWall,
            &[Hall, Living],
            1.0,
            &[K::Railing],
            &[StairOpening],
            &["escalier", "escalier droit", "marches", "stairs", "staircase"],
        ),
    ),
    (
        K::SpiralStairs,
        note(
            Floor,
            &[Hall, Living],
            0.6,
            &[K::Railing],
            &[StairOpening],
            &["colimaçon", "escalier en colimaçon", "escalier hélicoïdal", "spiral staircase"],
        ),
    ),
    (
        K::LStairs,
        note(
            AgainstWall,
            &[Hall, Living],
            1.0,
            &[K::Railing],
            &[StairOpening],
            &["quart tournant", "escalier quart tournant", "escalier en l", "escalier tournant", "l-shaped stairs"],
        ),
    ),
    (
        K::UStairs,
        note(
            AgainstWall,
            &[Hall, Living],
            1.0,
            &[K::Railing],
            &[StairOpening],
            &["demi-tour", "escalier demi-tour", "escalier en u", "u-shaped stairs"],
        ),
    ),
    (
        K::Ladder,
        note(
            AgainstWall,
            &[Hall, Garage],
            0.8,
            &[],
            &[StairOpening],
            &["échelle de meunier", "échelle", "escalier raide", "grenier", "mezzanine", "ladder"],
        ),
    ),
    (
        K::FloatingStairs,
        note(
            AgainstWall,
            &[Hall, Living],
            1.0,
            &[],
            &[StairOpening],
            &["escalier suspendu", "escalier", "escalier moderne", "floating stairs"],
        ),
    ),
    (
        K::StoneSteps,
        note(
            AgainstWall,
            &[Garden, Street],
            1.0,
            &[K::Balustrade],
            &[],
            &["perron", "marches", "escalier extérieur", "front steps"],
        ),
    ),
    (K::Escalator, note(Floor, &[Store, Hall], 1.5, &[], &[StairOpening], &["escalator", "escalier mécanique"])),
    (
        K::Ramp,
        note(
            AgainstWall,
            &[Street, Garden],
            1.5,
            &[K::Railing],
            &[],
            &["rampe d'accès", "rampe", "accès handicapé", "pmr", "wheelchair ramp"],
        ),
    ),
    (K::Pillar, note(Floor, &[Garden, Hall], 0.0, &[K::Pergola], &[], &["pilier", "poteau", "pillar", "post"])),
    (K::Column, note(Row, &[Hall, Garden], 0.0, &[K::Arch], &[], &["colonne", "péristyle", "column"])),
    (
        K::Railing,
        note(
            Row,
            &[Hall, Living, Garden],
            0.0,
            &[K::Stairs],
            &[],
            &["garde-corps", "rambarde", "balcon", "trémie", "railing", "handrail"],
        ),
    ),
    (
        K::Balustrade,
        note(Row, &[Garden, Street], 0.0, &[K::StoneSteps], &[], &["balustrade", "rambarde en pierre", "terrasse"]),
    ),
    (K::Arch, note(Floor, &[Garden, Street], 1.0, &[K::Column], &[], &["arche", "arc", "passage", "arch"])),
];
