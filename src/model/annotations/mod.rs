//! Annotations du catalogue pour les agents : où va chaque objet, comment le poser, avec quoi,
//! et par quels mots le chercher. Table séparée de `catalog!` pour que les branches qui ajoutent
//! des objets n'aient pas à toucher les mêmes lignes ; un test exige une annotation par sorte.

use serde::Serialize;

use super::PropKind;

mod garage;
mod garden;
mod kitchen;
mod leisure;
mod living;
mod office;
mod plants;
mod search;
mod signage;
mod sport;
mod store;
mod street;
mod structure;
mod vehicles;

pub use search::{fold, search};

/// Pièce ou lieu où un objet a sa place.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Room {
    Living,
    Bedroom,
    Kitchen,
    Bathroom,
    Office,
    /// Entrée, couloir, palier.
    Hall,
    Garage,
    Store,
    Garden,
    Street,
    /// Terrains et salle de sport.
    Sport,
}

impl Room {
    /// Pièce nommée en anglais (`Kitchen`) ou en français (« cuisine », « salle de bain »).
    pub fn parse(word: &str) -> Option<Self> {
        let w = fold(word.trim());
        let french = match w.as_str() {
            "salon" | "sejour" => Some(Self::Living),
            "chambre" => Some(Self::Bedroom),
            "cuisine" => Some(Self::Kitchen),
            "salle de bain" | "salle de bains" | "sdb" => Some(Self::Bathroom),
            "bureau" => Some(Self::Office),
            "entree" | "couloir" | "palier" => Some(Self::Hall),
            "magasin" | "boutique" => Some(Self::Store),
            "jardin" => Some(Self::Garden),
            "rue" => Some(Self::Street),
            _ => None,
        };
        french.or_else(|| Self::ALL.into_iter().find(|r| fold(&format!("{r:?}")) == w))
    }

    pub const ALL: [Self; 11] = [
        Self::Living,
        Self::Bedroom,
        Self::Kitchen,
        Self::Bathroom,
        Self::Office,
        Self::Hall,
        Self::Garage,
        Self::Store,
        Self::Garden,
        Self::Street,
        Self::Sport,
    ];
}

/// Façon de poser l'objet : ce que devra respecter un placement automatique.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Placement {
    /// Au sol, dégagé : on peut tourner autour (table, arbre).
    Floor,
    /// Au sol, dos contre un mur, avant vers la pièce (canapé, lit, armoire).
    AgainstWall,
    /// Fixé au mur, en hauteur (miroir, applique, étagère murale).
    OnWall,
    /// Posé sur un autre meuble (vase, lampe de chevet).
    OnTop,
    /// Suspendu au plafond (lustre).
    Ceiling,
    /// Module à aligner en rangée avec `place_row` (haie, clôture, lampadaires).
    Row,
}

impl Placement {
    pub const ALL: [Self; 6] = [Self::Floor, Self::AgainstWall, Self::OnWall, Self::OnTop, Self::Ceiling, Self::Row];
}

/// Ce que l'objet exige autour de lui pour avoir du sens.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Need {
    /// Une route ou une allée sous lui.
    Road,
    /// Une trémie dans le sol de l'étage d'arrivée.
    StairOpening,
}

/// Ce qu'un agent doit savoir pour choisir et poser un objet.
#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct Annotation {
    pub rooms: &'static [Room],
    pub placement: Placement,
    /// Espace à laisser libre devant l'objet, en mètres (ouverture des portes, passage, assise).
    pub front: f32,
    /// Objets qu'on pose habituellement avec lui.
    pub goes_with: &'static [PropKind],
    pub needs: &'static [Need],
    /// Mots par lesquels le chercher, en français puis en anglais, en minuscules.
    pub synonyms: &'static [&'static str],
}

pub(super) const fn note(
    placement: Placement,
    rooms: &'static [Room],
    front: f32,
    goes_with: &'static [PropKind],
    needs: &'static [Need],
    synonyms: &'static [&'static str],
) -> Annotation {
    Annotation { rooms, placement, front, goes_with, needs, synonyms }
}

/// Une table par catégorie, comme `shapes/` : chaque branche qui ajoute des objets complète la sienne.
const TABLES: [&[(PropKind, Annotation)]; 13] = [
    living::NOTES,
    kitchen::NOTES,
    office::NOTES,
    garage::NOTES,
    store::NOTES,
    street::NOTES,
    signage::NOTES,
    plants::NOTES,
    garden::NOTES,
    leisure::NOTES,
    sport::NOTES,
    structure::NOTES,
    vehicles::NOTES,
];

impl PropKind {
    /// Annotation de cette sorte ; `None` seulement pour une sorte ajoutée sans la sienne (refusé par les tests).
    pub fn annotation(self) -> Option<Annotation> {
        TABLES.iter().flat_map(|t| t.iter()).find(|(k, _)| *k == self).map(|(_, a)| *a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Toute sorte du catalogue doit être annotée : la liste des manquantes dit quoi compléter
    /// après la fusion d'une branche qui ajoute des objets.
    #[test]
    fn every_kind_is_annotated() {
        let missing: Vec<PropKind> = PropKind::ALL.into_iter().filter(|k| k.annotation().is_none()).collect();
        assert!(missing.is_empty(), "sortes sans annotation (src/model/annotations/) : {missing:?}");
    }

    #[test]
    fn annotations_are_consistent() {
        for k in PropKind::ALL {
            let Some(a) = k.annotation() else { continue };
            assert!(!a.rooms.is_empty(), "{k:?} : aucune pièce");
            assert!((0.0..=3.0).contains(&a.front), "{k:?} : dégagement {}", a.front);
            assert!(!a.synonyms.is_empty(), "{k:?} : aucun synonyme");
            for s in a.synonyms {
                assert_eq!(*s, s.to_lowercase(), "{k:?} : synonyme « {s} » pas en minuscules");
                assert_eq!(*s, s.trim(), "{k:?} : espaces autour de « {s} »");
            }
            // l'objet se trouve au moins par son nom affiché
            assert!(
                a.synonyms.contains(&k.name().to_lowercase().as_str()),
                "{k:?} : « {} » absent des synonymes",
                k.name()
            );
            // un objet d'extérieur va dehors, un objet d'intérieur dedans
            let outside = |r: &Room| matches!(r, Room::Garden | Room::Street | Room::Sport);
            assert!(
                !k.outdoor() || a.rooms.iter().any(|r| outside(r) || *r == Room::Garage),
                "{k:?} : dehors sans lieu extérieur"
            );
            assert!(k.outdoor() || !a.rooms.iter().all(outside), "{k:?} : d'intérieur mais seulement dehors");
            assert!(!a.goes_with.contains(&k), "{k:?} va avec lui-même");
        }
    }

    #[test]
    fn no_kind_is_annotated_twice() {
        let all: Vec<PropKind> = TABLES.iter().flat_map(|t| t.iter().map(|(k, _)| *k)).collect();
        for k in &all {
            assert_eq!(all.iter().filter(|x| *x == k).count(), 1, "{k:?} annoté plusieurs fois");
        }
    }
}
