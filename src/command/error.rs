//! Raisons du refus d'une commande, formulées pour qu'un agent sache quoi corriger.

use bevy::math::Vec2;

use crate::model::{ElemRef, OpeningKind, Rgb};

#[derive(Clone, PartialEq, Debug)]
pub enum EditError {
    LevelOutOfRange(i32),
    EmptyRect {
        min: Vec2,
        max: Vec2,
    },
    /// L'élément visé et le nombre d'éléments de sa liste.
    NotFound(ElemRef, usize),
    WallTooShort,
    PoolOffGrid,
    RoadTooShort,
    /// Le mur visé et le nombre de murs.
    WallNotFound(usize, usize),
    OpeningOutsideWall {
        kind: OpeningKind,
        t: f32,
        t_min: f32,
        t_max: f32,
        wall_len: f32,
    },
    OpeningOverlaps {
        kind: OpeningKind,
        t: f32,
        other: usize,
    },
    OpeningTooTall {
        top: f32,
        wall_height: f32,
    },
    OpeningNotDuplicable,
    NotRotatable,
    OutOfWorld(Vec2),
    OutOfRange {
        field: &'static str,
        value: f32,
        min: f32,
        max: f32,
    },
    BadColor(Rgb),
    UnknownTexture {
        texture: usize,
        max: usize,
    },
    /// Champs refusés par `set` (nom inconnu, mauvais type).
    BadFields(String),
    TooMany {
        count: usize,
        max: usize,
    },
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LevelOutOfRange(l) => write!(f, "Étage {l} hors limites (-1 à 9)"),
            Self::EmptyRect { min, max } => {
                write!(
                    f,
                    "Rectangle vide : min [{}, {}] doit être < max [{}, {}] sur x et z",
                    min.x, min.y, max.x, max.y
                )
            }
            Self::NotFound(e, 0) => write!(f, "Élément introuvable : {e:?} (aucun élément de ce type)"),
            Self::NotFound(e, n) => write!(f, "Élément introuvable : {e:?} (indices valides : 0 à {})", n - 1),
            Self::WallTooShort => write!(f, "Mur trop court (min 5 cm)"),
            Self::PoolOffGrid => write!(f, "Piscine : coins au mètre près exigés"),
            Self::RoadTooShort => write!(f, "Route trop courte (min 5 cm)"),
            Self::WallNotFound(w, n) => write!(f, "Mur introuvable : {w} (il y a {n} murs)"),
            Self::OpeningOutsideWall { kind, t, t_min, t_max, wall_len } if t_min > t_max => {
                write!(f, "{} à t={t} : le mur de {wall_len:.2} m est trop court pour cette largeur", kind.name())
            }
            Self::OpeningOutsideWall { kind, t, t_min, t_max, wall_len } => write!(
                f,
                "{} à t={t} dépasse du mur de {wall_len:.2} m : t doit être entre {t_min:.2} et {t_max:.2}",
                kind.name()
            ),
            Self::OpeningOverlaps { kind, t, other } => {
                write!(f, "{} à t={t} chevauche l'ouverture {other} de ce mur", kind.name())
            }
            Self::OpeningTooTall { top, wall_height } => {
                write!(
                    f,
                    "Ouverture trop haute : son haut (allège + hauteur = {top:.2} m) dépasse le mur de {wall_height} m"
                )
            }
            Self::OpeningNotDuplicable => write!(f, "Les ouvertures ne se dupliquent pas"),
            Self::NotRotatable => write!(f, "Seuls les objets (Prop) pivotent"),
            Self::OutOfWorld(p) => {
                write!(f, "Point [{}, {}] hors du terrain : x et z doivent rester entre -80 et 80", p.x, p.y)
            }
            Self::OutOfRange { field, value, min, max } => {
                write!(f, "{field} = {value} hors limites : doit être entre {min} et {max}")
            }
            Self::BadColor(c) => write!(f, "Couleur {c:?} invalide : chaque composante doit être entre 0 et 1"),
            Self::UnknownTexture { texture, max } => {
                write!(f, "Texture {texture} inconnue : 0 (aucune) à {max}, voir `maison3d textures`")
            }
            Self::BadFields(e) => write!(f, "Champs refusés par set : {e}"),
            Self::TooMany { count, max } => write!(f, "{count} objets demandés, {max} au plus par rangée"),
        }
    }
}

impl std::error::Error for EditError {}
