//! Annotations : salon et chambre.

use super::Placement::{AgainstWall, Ceiling, Floor};
use super::Room::{Bathroom, Bedroom, Hall, Kitchen, Living, Office};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Table,
        note(
            Floor,
            &[Living, Kitchen],
            0.8,
            &[K::Chair],
            &[],
            &["table", "table à manger", "table de repas", "salle à manger", "dining table"],
        ),
    ),
    (K::Chair, note(Floor, &[Living, Kitchen, Office], 0.5, &[K::Table], &[], &["chaise", "siège", "chair", "seat"])),
    (
        K::Sofa,
        note(
            AgainstWall,
            &[Living],
            0.9,
            &[K::CoffeeTable, K::Lamp],
            &[],
            &["canapé", "divan", "banquette", "sofa", "couch"],
        ),
    ),
    (
        K::Lamp,
        note(
            Floor,
            &[Living, Bedroom, Office],
            0.3,
            &[K::Sofa, K::Armchair],
            &[],
            &["lampe sur pied", "lampe", "liseuse", "éclairage", "floor lamp"],
        ),
    ),
    (
        K::Armchair,
        note(
            Floor,
            &[Living, Bedroom],
            0.6,
            &[K::CoffeeTable, K::Ottoman, K::Lamp],
            &[],
            &["fauteuil", "fauteuil club", "siège", "armchair"],
        ),
    ),
    (
        K::CoffeeTable,
        note(
            Floor,
            &[Living],
            0.4,
            &[K::Sofa, K::Armchair, K::Rug],
            &[],
            &["table basse", "table de salon", "coffee table"],
        ),
    ),
    (
        K::RoundTable,
        note(Floor, &[Living, Kitchen], 0.8, &[K::Chair], &[], &["table ronde", "guéridon", "round table"]),
    ),
    (
        K::CornerSofa,
        note(
            AgainstWall,
            &[Living],
            0.9,
            &[K::CoffeeTable, K::TvStand],
            &[],
            &["canapé d'angle", "canapé en l", "méridienne", "corner sofa", "sectional"],
        ),
    ),
    (
        K::TvStand,
        note(
            AgainstWall,
            &[Living, Bedroom],
            1.5,
            &[K::Sofa, K::CornerSofa],
            &[],
            &["meuble télé", "télévision", "télé", "tv", "tv stand"],
        ),
    ),
    (
        K::Bookcase,
        note(
            AgainstWall,
            &[Living, Office, Bedroom],
            0.8,
            &[K::Armchair],
            &[],
            &["bibliothèque", "livres", "étagère à livres", "bookcase", "bookshelf"],
        ),
    ),
    (
        K::Sideboard,
        note(AgainstWall, &[Living, Kitchen], 0.7, &[K::Table], &[], &["buffet", "bahut", "enfilade", "sideboard"]),
    ),
    (
        K::Rug,
        note(
            Floor,
            &[Living, Bedroom, Office],
            0.0,
            &[K::CoffeeTable, K::Sofa, K::Bed],
            &[],
            &["tapis", "carpette", "rug", "carpet"],
        ),
    ),
    (
        K::Fireplace,
        note(AgainstWall, &[Living], 1.0, &[K::Armchair, K::Sofa], &[], &["cheminée", "foyer", "feu", "fireplace"]),
    ),
    (
        K::Piano,
        note(AgainstWall, &[Living], 0.8, &[K::Bookcase], &[], &["piano droit", "piano", "instrument", "musique"]),
    ),
    (
        K::CeilingLamp,
        note(
            Ceiling,
            &[Living, Bedroom, Kitchen, Office, Hall],
            0.0,
            &[K::Table],
            &[],
            &["suspension", "lustre", "plafonnier", "luminaire", "ceiling lamp", "chandelier"],
        ),
    ),
    (K::Aquarium, note(AgainstWall, &[Living, Office], 0.8, &[K::Sofa], &[], &["aquarium", "poissons", "fish tank"])),
    (
        K::Ottoman,
        note(Floor, &[Living, Bedroom], 0.3, &[K::Armchair, K::Vanity], &[], &["pouf", "repose-pieds", "ottoman"]),
    ),
    (
        K::Bed,
        note(
            AgainstWall,
            &[Bedroom],
            0.7,
            &[K::Nightstand, K::Wardrobe],
            &[],
            &["lit", "lit double", "couchage", "bed", "double bed"],
        ),
    ),
    (
        K::Wardrobe,
        note(
            AgainstWall,
            &[Bedroom, Hall],
            0.9,
            &[K::Bed],
            &[],
            &["armoire", "penderie", "placard", "dressing", "rangement", "wardrobe"],
        ),
    ),
    (
        K::SingleBed,
        note(
            AgainstWall,
            &[Bedroom],
            0.7,
            &[K::Nightstand, K::Dresser],
            &[],
            &["lit simple", "lit", "lit une place", "chambre d'enfant", "single bed"],
        ),
    ),
    (
        K::BunkBed,
        note(
            AgainstWall,
            &[Bedroom],
            0.8,
            &[K::ToyChest],
            &[],
            &["lits superposés", "lit superposé", "lits jumeaux", "chambre d'enfant", "bunk bed"],
        ),
    ),
    (
        K::Crib,
        note(
            AgainstWall,
            &[Bedroom],
            0.6,
            &[K::ToyChest, K::Dresser],
            &[],
            &["lit bébé", "berceau", "chambre de bébé", "crib", "cot"],
        ),
    ),
    (
        K::Nightstand,
        note(
            AgainstWall,
            &[Bedroom],
            0.4,
            &[K::Bed, K::SingleBed],
            &[],
            &["table de chevet", "chevet", "nightstand", "bedside table"],
        ),
    ),
    (
        K::Dresser,
        note(
            AgainstWall,
            &[Bedroom],
            0.8,
            &[K::Bed, K::Mirror],
            &[],
            &["commode", "tiroirs", "rangement", "dresser", "chest of drawers"],
        ),
    ),
    (
        K::Vanity,
        note(
            AgainstWall,
            &[Bedroom, Bathroom],
            0.7,
            &[K::Ottoman],
            &[],
            &["coiffeuse", "table de maquillage", "dressing table", "vanity"],
        ),
    ),
    (
        K::ToyChest,
        note(
            AgainstWall,
            &[Bedroom],
            0.6,
            &[K::SingleBed, K::BunkBed, K::Crib],
            &[],
            &["coffre à jouets", "jouets", "coffre", "toy chest", "toy box"],
        ),
    ),
];
