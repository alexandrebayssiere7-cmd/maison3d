//! Annotations : bureau.

use super::Placement::{AgainstWall, Floor, Row};
use super::Room::{Bedroom, Hall, Office, Sport, Store};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (
        K::Desk,
        note(
            AgainstWall,
            &[Office, Bedroom],
            0.9,
            &[K::OfficeChair, K::FilingCabinet],
            &[],
            &["bureau", "bureau droit", "table de travail", "desk"],
        ),
    ),
    (
        K::ComputerDesk,
        note(
            AgainstWall,
            &[Office, Bedroom],
            0.9,
            &[K::OfficeChair],
            &[],
            &["bureau informatique", "ordinateur", "poste de travail", "pc", "computer desk"],
        ),
    ),
    (
        K::OfficeChair,
        note(
            Floor,
            &[Office, Bedroom],
            0.5,
            &[K::Desk, K::ComputerDesk],
            &[],
            &["fauteuil de bureau", "chaise de bureau", "siège de bureau", "office chair"],
        ),
    ),
    (
        K::FilingCabinet,
        note(
            Floor,
            &[Office],
            0.5,
            &[K::Desk],
            &[],
            &["caisson à tiroirs", "caisson", "tiroirs de bureau", "filing cabinet"],
        ),
    ),
    (
        K::OfficeCabinet,
        note(
            AgainstWall,
            &[Office],
            0.9,
            &[K::Desk],
            &[],
            &["armoire de bureau", "classeurs", "armoire à dossiers", "office cabinet"],
        ),
    ),
    (
        K::MeetingTable,
        note(
            Floor,
            &[Office],
            1.0,
            &[K::OfficeChair, K::Whiteboard],
            &[],
            &["table de réunion", "salle de réunion", "table ovale", "meeting table", "conference table"],
        ),
    ),
    (
        K::Cubicle,
        note(
            Row,
            &[Office],
            0.0,
            &[K::ComputerDesk],
            &[],
            &["cloison d'open space", "cloison", "open space", "box", "cubicle", "partition"],
        ),
    ),
    (
        K::Whiteboard,
        note(Floor, &[Office], 1.0, &[K::MeetingTable], &[], &["tableau blanc", "tableau", "paperboard", "whiteboard"]),
    ),
    (
        K::Copier,
        note(
            AgainstWall,
            &[Office],
            0.9,
            &[K::OfficeCabinet],
            &[],
            &["photocopieur", "imprimante", "photocopieuse", "copier", "printer"],
        ),
    ),
    (
        K::WaterCooler,
        note(
            AgainstWall,
            &[Office, Hall],
            0.6,
            &[K::CoatRack],
            &[],
            &["fontaine à eau", "bonbonne", "eau", "water cooler"],
        ),
    ),
    (
        K::Lockers,
        note(AgainstWall, &[Office, Sport], 0.9, &[K::Bench], &[], &["vestiaire", "casiers", "consigne", "lockers"]),
    ),
    (
        K::ReceptionDesk,
        note(
            Floor,
            &[Office, Hall, Store],
            1.2,
            &[K::OfficeChair, K::Plant],
            &[],
            &["banque d'accueil", "accueil", "réception", "comptoir d'accueil", "reception desk"],
        ),
    ),
    (
        K::CoatRack,
        note(
            Floor,
            &[Hall, Office],
            0.4,
            &[K::WaterCooler],
            &[],
            &["portemanteau", "vestiaire", "patère", "coat rack", "coat stand"],
        ),
    ),
    (
        K::ServerRack,
        note(
            AgainstWall,
            &[Office],
            1.0,
            &[K::ComputerDesk],
            &[],
            &["baie serveur", "serveurs", "informatique", "server rack"],
        ),
    ),
];
