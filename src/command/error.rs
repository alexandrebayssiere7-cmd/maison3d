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
    /// Partie demandée et nombre de parties de l'élément.
    NoSuchPart {
        part: usize,
        count: usize,
    },
    /// Plus de finitions que l'objet ou l'ouverture n'a de parties après la principale.
    TooManyFinishes {
        count: usize,
        max: usize,
    },
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
            Self::NoSuchPart { part, count: 1 } => {
                write!(f, "Partie {part} inexistante : cet élément n'a que la partie 0")
            }
            Self::NoSuchPart { part, count } => write!(
                f,
                "Partie {part} inexistante : cet élément a {count} parties (0 à {}, voir `maison3d catalog`)",
                count.saturating_sub(1)
            ),
            Self::TooManyFinishes { count, max } => write!(
                f,
                "{count} finitions données, {max} au plus : une par partie après la principale (`maison3d catalog`)"
            ),
            Self::BadFields(e) => write!(f, "Champs refusés par set : {e}"),
            Self::TooMany { count, max } => write!(f, "{count} objets demandés, {max} au plus par rangée"),
        }
    }
}

impl std::error::Error for EditError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_has_a_message() {
        let door = OpeningKind::Door;
        let cases = [
            (EditError::LevelOutOfRange(12), "Étage 12"),
            (EditError::EmptyRect { min: Vec2::ONE, max: Vec2::ZERO }, "Rectangle vide"),
            (EditError::NotFound(ElemRef::Wall(3), 0), "aucun élément"),
            (EditError::NotFound(ElemRef::Wall(3), 2), "0 à 1"),
            (EditError::WallTooShort, "Mur trop court"),
            (EditError::PoolOffGrid, "au mètre"),
            (EditError::RoadTooShort, "Route trop courte"),
            (EditError::WallNotFound(4, 2), "il y a 2 murs"),
            (EditError::OpeningOutsideWall { kind: door, t: 0.1, t_min: 2.0, t_max: 1.0, wall_len: 0.5 }, "trop court"),
            (EditError::OpeningOutsideWall { kind: door, t: 0.1, t_min: 0.5, t_max: 3.5, wall_len: 4.0 }, "entre 0.50"),
            (EditError::OpeningOverlaps { kind: door, t: 1.0, other: 0 }, "chevauche"),
            (EditError::OpeningTooTall { top: 3.5, wall_height: 3.0 }, "trop haute"),
            (EditError::OpeningNotDuplicable, "dupliquent"),
            (EditError::NotRotatable, "pivotent"),
            (EditError::OutOfWorld(Vec2::splat(99.0)), "hors du terrain"),
            (EditError::OutOfRange { field: "amp", value: 99.0, min: 0.0, max: 60.0 }, "amp = 99"),
            (EditError::BadColor([2.0, 0.0, 0.0]), "Couleur"),
            (EditError::NoSuchPart { part: 1, count: 1 }, "que la partie 0"),
            (EditError::NoSuchPart { part: 9, count: 3 }, "0 à 2"),
            (EditError::TooManyFinishes { count: 4, max: 1 }, "1 au plus"),
            (EditError::UnknownTexture { texture: 99, max: 40 }, "Texture 99"),
            (EditError::BadFields("x".into()), "set : x"),
            (EditError::TooMany { count: 500, max: 200 }, "500 objets"),
        ];
        for (e, needle) in cases {
            let msg = e.to_string();
            assert!(msg.contains(needle), "{e:?} → {msg}");
        }
    }
}
