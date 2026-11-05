//! Mobilier urbain (bancs, éclairage, abris, conteneurs…) et signalisation.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};

use super::{Piece, ball, bx, cyl, door, light, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const TYRES: Slot = Slot::plain("Roues", [0.08, 0.08, 0.09]);

// Parties colorables de chaque sorte, la partie 0 en tête.
const STREET_LAMP: &[Slot] =
    &[Slot::plain("Mât", [0.2, 0.22, 0.25]), Slot::plain("Lanterne et lumière", [1.0, 0.95, 0.8])];
const LANTERN: &[Slot] =
    &[Slot::plain("Fonte", [0.1, 0.14, 0.12]), Slot::plain("Vitres et lumière", [1.0, 0.85, 0.55])];
const BENCH: &[Slot] = &[Slot::textured("Lattes", "Bois clair"), Slot::plain("Piètement", [0.12, 0.22, 0.16])];
const TRASH_CAN: &[Slot] = &[
    Slot::plain("Corps", [0.15, 0.35, 0.2]),
    Slot::plain("Couvercle et cerclages", [0.1, 0.12, 0.11]),
    Slot::plain("Pied", [0.3, 0.3, 0.32]),
    Slot::plain("Ouverture", [0.04, 0.04, 0.04]),
];
const RECYCLING_BINS: &[Slot] = &[
    Slot::plain("Bac jaune", [0.95, 0.78, 0.1]),
    Slot::plain("Bac vert", [0.2, 0.55, 0.25]),
    Slot::plain("Bac bleu", [0.15, 0.35, 0.75]),
    Slot::plain("Couvercles", [0.25, 0.26, 0.28]),
    TYRES,
];
const DUMPSTER: &[Slot] = &[
    Slot::plain("Cuve", [0.25, 0.42, 0.28]),
    Slot::plain("Couvercle", [0.18, 0.32, 0.2]),
    TYRES,
    Slot::plain("Poignées et châssis", METAL),
];
const BUS_STOP: &[Slot] = &[
    Slot::plain("Structure", [0.35, 0.37, 0.4]),
    Slot::plain("Toit", [0.25, 0.26, 0.28]),
    Slot::textured("Banc", "Inox brossé"),
    Slot::plain("Affiche", [0.95, 0.6, 0.2]),
    Slot::plain("Plaque d'arrêt", [0.1, 0.35, 0.75]),
];
const FIRE_HYDRANT: &[Slot] = &[Slot::plain("Corps", [0.8, 0.1, 0.08]), Slot::plain("Raccords", METAL)];
const BOLLARD: &[Slot] = &[Slot::plain("Fût", [0.28, 0.2, 0.14]), Slot::plain("Bande réfléchissante", WHITE)];
const MAILBOX: &[Slot] = &[
    Slot::plain("Boîte", [0.98, 0.8, 0.1]),
    Slot::plain("Pied", [0.35, 0.36, 0.38]),
    Slot::plain("Fente", DARK),
    Slot::plain("Logo", [0.1, 0.2, 0.6]),
];
const PARKING_METER: &[Slot] = &[
    Slot::plain("Boîtier", [0.3, 0.32, 0.36]),
    Slot::plain("Écran", [0.55, 0.75, 0.55]),
    Slot::plain("Clavier", METAL),
    Slot::plain("Panneau solaire", [0.08, 0.1, 0.25]),
    Slot::plain("Socle", DARK),
];
const PHONE_BOOTH: &[Slot] = &[
    Slot::plain("Cadre", [0.75, 0.1, 0.08]),
    Slot::plain("Toit et socle", [0.2, 0.2, 0.22]),
    Slot::plain("Téléphone", DARK),
    Slot::plain("Enseigne", WHITE),
    Slot::plain("Poignée", METAL),
];
const BIKE_RACK: &[Slot] = &[Slot::plain("Arceaux", [0.3, 0.3, 0.32])];
const BILLBOARD: &[Slot] = &[
    Slot::plain("Affiche", [0.95, 0.55, 0.15]),
    Slot::plain("Motif", [0.15, 0.35, 0.75]),
    Slot::plain("Cadre", [0.2, 0.2, 0.22]),
    Slot::plain("Pied", [0.6, 0.62, 0.65]),
    Slot::plain("Projecteurs", DARK),
];
const UTILITY_POLE: &[Slot] = &[
    Slot::plain("Poteau", [0.4, 0.3, 0.2]),
    Slot::plain("Traverse", [0.5, 0.5, 0.52]),
    Slot::plain("Isolateurs", [0.85, 0.9, 0.85]),
];
const MANHOLE: &[Slot] = &[Slot::plain("Fonte", [0.25, 0.25, 0.27]), Slot::plain("Motif", [0.17, 0.17, 0.18])];
const KIOSK: &[Slot] = &[
    Slot::plain("Structure", [0.1, 0.3, 0.2]),
    Slot::textured("Toit", "Zinc"),
    Slot::textured("Comptoir", "Bois clair"),
    Slot::plain("Journaux", [0.92, 0.9, 0.85]),
    Slot::plain("Magazines", [0.85, 0.3, 0.3]),
    Slot::plain("Enseigne", [0.95, 0.8, 0.15]),
];
const DRINKING_FOUNTAIN: &[Slot] = &[
    Slot::plain("Fût", [0.2, 0.3, 0.25]),
    Slot::textured("Vasque", "Inox brossé"),
    Slot::plain("Grille", DARK),
    Slot::plain("Eau", [0.6, 0.8, 0.95]),
];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::StreetLamp => STREET_LAMP,
        PropKind::Lantern => LANTERN,
        PropKind::Bench => BENCH,
        PropKind::TrashCan => TRASH_CAN,
        PropKind::RecyclingBins => RECYCLING_BINS,
        PropKind::Dumpster => DUMPSTER,
        PropKind::BusStop => BUS_STOP,
        PropKind::FireHydrant => FIRE_HYDRANT,
        PropKind::Bollard => BOLLARD,
        PropKind::Mailbox => MAILBOX,
        PropKind::ParkingMeter => PARKING_METER,
        PropKind::PhoneBooth => PHONE_BOOTH,
        PropKind::BikeRack => BIKE_RACK,
        PropKind::Billboard => BILLBOARD,
        PropKind::UtilityPole => UTILITY_POLE,
        PropKind::Manhole => MANHOLE,
        PropKind::Kiosk => KIOSK,
        PropKind::DrinkingFountain => DRINKING_FOUNTAIN,
        _ => &[],
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    match kind {
        PropKind::StreetLamp => street_lamp(),
        PropKind::Lantern => lantern(),
        PropKind::Bench => bench(),
        PropKind::TrashCan => trash_can(),
        PropKind::RecyclingBins => recycling_bins(),
        PropKind::Dumpster => dumpster(open),
        PropKind::BusStop => bus_stop(),
        PropKind::FireHydrant => fire_hydrant(),
        PropKind::Bollard => bollard(),
        PropKind::Mailbox => mailbox(),
        PropKind::ParkingMeter => parking_meter(),
        PropKind::PhoneBooth => phone_booth(open),
        PropKind::BikeRack => bike_rack(),
        PropKind::Billboard => billboard(),
        PropKind::UtilityPole => utility_pole(),
        PropKind::Manhole => manhole(),
        PropKind::Kiosk => kiosk(),
        PropKind::DrinkingFountain => drinking_fountain(),
        _ => Vec::new(),
    }
}

fn street_lamp() -> Vec<Piece> {
    vec![
        cyl(0.0, 0.0, 0.3, 0.0, 0.3, 0),
        cyl(0.0, 0.0, 0.12, 0.1, 4.5, 0),
        // bras qui monte légèrement du mât vers la lanterne
        span(Shape::Cube, Vec3::new(0.0, 4.32, 0.0), Vec3::new(0.0, 4.47, 0.6), 0.08, 0),
        bx(0.0, 0.75, 0.35, 0.45, 4.25, 4.45, 0),
        bx(0.0, 0.75, 0.3, 0.4, 4.2, 4.25, 1),
        light([0.0, 4.0, 0.75], 1),
    ]
}

/// Réverbère ancien : socle mouluré, fût bagué, barre porte-échelle et lanterne à quatre vitres.
fn lantern() -> Vec<Piece> {
    let mut v = vec![
        cyl(0.0, 0.0, 0.42, 0.0, 0.25, 0),
        cyl(0.0, 0.0, 0.3, 0.25, 0.55, 0),
        cyl(0.0, 0.0, 0.13, 0.55, 3.0, 0),
        cyl(0.0, 0.0, 0.18, 1.2, 1.28, 0),
        cyl(0.0, 0.0, 0.18, 2.2, 2.28, 0),
        bx(0.0, 0.0, 0.55, 0.05, 2.7, 2.75, 0),
        cyl(0.0, 0.0, 0.22, 3.0, 3.08, 0),
        bx(0.0, 0.0, 0.32, 0.32, 3.08, 3.12, 0),
        bx(0.0, 0.0, 0.27, 0.27, 3.12, 3.55, 1),
        Piece::new(Shape::Cone, Vec3::new(0.0, 3.68, 0.0), Vec3::new(0.48, 0.26, 0.48), 0),
        ball([0.0, 3.84, 0.0], [0.08, 0.1, 0.08], 0),
        light([0.0, 3.33, 0.0], 1),
    ];
    for (x, z) in [(-0.14, -0.14), (0.14, -0.14), (-0.14, 0.14), (0.14, 0.14)] {
        v.push(bx(x, z, 0.03, 0.03, 3.12, 3.55, 0));
    }
    v
}

/// Banc public : assise et dossier en lattes, piètement en fonte ; l'avant est l'assise.
fn bench() -> Vec<Piece> {
    let mut v = Vec::new();
    for z in [0.2, 0.08, -0.04, -0.16] {
        v.push(bx(0.0, z, 1.8, 0.1, 0.43, 0.46, 0));
    }
    // dossier incliné vers l'arrière, collé aux montants
    for (y, z) in [(0.6, -0.242), (0.72, -0.268), (0.84, -0.294)] {
        let slat = Piece::new(Shape::Cube, Vec3::new(0.0, y, z), Vec3::new(1.8, 0.1, 0.025), 0);
        v.push(slat.turn(Quat::from_rotation_x(-0.18)));
    }
    for x in [-0.75, 0.75] {
        v.push(bx(x, 0.2, 0.06, 0.06, 0.0, 0.43, 1));
        v.push(span(Shape::Cube, Vec3::new(x, 0.01, -0.18), Vec3::new(x, 0.9, -0.34), 0.06, 1));
        v.push(bx(x, 0.02, 0.06, 0.46, 0.38, 0.43, 1));
        v.push(bx(x, -0.02, 0.06, 0.5, 0.62, 0.66, 1));
        v.push(bx(x, 0.2, 0.05, 0.05, 0.46, 0.62, 1));
    }
    v
}

/// Corbeille de rue cylindrique sur pied, couvercle bombé et ouverture à l'avant.
fn trash_can() -> Vec<Piece> {
    vec![
        cyl(0.0, 0.0, 0.32, 0.0, 0.08, 2),
        cyl(0.0, 0.0, 0.46, 0.08, 0.86, 0),
        cyl(0.0, 0.0, 0.48, 0.2, 0.24, 1),
        cyl(0.0, 0.0, 0.48, 0.82, 0.86, 1),
        ball([0.0, 0.88, 0.0], [0.47, 0.2, 0.47], 1),
        bx(0.0, 0.224, 0.18, 0.02, 0.62, 0.8, 3),
    ]
}

/// Trois bacs roulants de tri côte à côte, roues et poignées à l'arrière.
fn recycling_bins() -> Vec<Piece> {
    let mut v = Vec::new();
    for (i, x) in [-0.7, 0.0, 0.7].into_iter().enumerate() {
        v.push(bx(x, 0.0, 0.58, 0.7, 0.04, 1.0, i));
        v.push(bx(x, 0.03, 0.62, 0.76, 1.0, 1.05, 3));
        v.push(bx(x, -0.37, 0.5, 0.04, 0.92, 0.97, 3));
        for dx in [-0.31, 0.31] {
            v.push(Piece::new(Shape::Wheel, Vec3::new(x + dx, 0.1, -0.28), Vec3::new(0.05, 0.2, 0.2), 4));
        }
    }
    v
}

/// Conteneur à ordures de 1 100 l sur quatre roulettes ; le couvercle se soulève vers l'arrière.
fn dumpster(open: f32) -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 0.0, 1.3, 1.0, 0.22, 1.12, 0),
        bx(0.0, 0.0, 1.36, 1.06, 1.08, 1.17, 0),
        bx(0.0, 0.0, 1.2, 0.9, 0.15, 0.22, 3),
    ];
    for x in [-0.25, 0.25] {
        v.push(bx(x, 0.505, 0.08, 0.02, 0.3, 1.05, 0));
    }
    for x in [-0.69, 0.69] {
        v.push(bx(x, 0.0, 0.04, 0.5, 0.9, 0.95, 3));
    }
    for (x, z) in [(-0.5, -0.36), (0.5, -0.36), (-0.5, 0.36), (0.5, 0.36)] {
        v.push(bx(x, z, 0.06, 0.06, 0.08, 0.15, 3));
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.075, z), Vec3::new(0.05, 0.15, 0.15), 2));
    }
    let q = Quat::from_rotation_x(-open * 1.9);
    let hinge = Vec3::new(0.0, 1.17, -0.53);
    v.push(Piece::new(Shape::Cube, hinge + q * Vec3::new(0.0, 0.03, 0.54), Vec3::new(1.38, 0.06, 1.08), 1).turn(q));
    v.push(Piece::new(Shape::Cube, hinge + q * Vec3::new(0.0, 0.0, 1.09), Vec3::new(0.5, 0.05, 0.04), 3).turn(q));
    v
}

/// Abribus ouvert vers l'avant (la chaussée) : paroi du fond et côté ouest vitrés, caisson
/// publicitaire côté est, banc et plaque d'arrêt.
fn bus_stop() -> Vec<Piece> {
    let glass = |at: [f32; 3], size: [f32; 3]| Piece::new(Shape::Glass, Vec3::from(at), Vec3::from(size), 0);
    let mut v = vec![
        bx(0.0, -0.05, 4.1, 1.6, 2.4, 2.5, 1),
        glass([0.0, 1.25, -0.7], [3.84, 2.2, 0.02]),
        glass([-1.96, 1.25, -0.25], [0.02, 2.2, 0.86]),
        bx(0.0, -0.7, 3.9, 0.06, 0.1, 0.15, 0),
        bx(0.0, -0.7, 3.9, 0.06, 2.34, 2.4, 0),
        bx(-0.3, -0.5, 1.8, 0.35, 0.45, 0.5, 2),
        bx(1.96, -0.25, 0.14, 0.9, 0.15, 2.2, 0),
        bx(1.96, -0.25, 0.15, 0.8, 0.3, 2.05, 3),
        bx(-1.6, 0.74, 0.7, 0.03, 2.5, 2.75, 4),
    ];
    for x in [-1.96, 1.96] {
        for z in [-0.7, 0.2] {
            v.push(bx(x, z, 0.08, 0.08, 0.0, 2.4, 0));
        }
    }
    for x in [-1.0, 0.4] {
        v.push(bx(x, -0.6, 0.05, 0.2, 0.0, 0.45, 2));
    }
    v
}

/// Poteau d'incendie : fût, chapeau, deux sorties latérales et une sortie avant bouchonnées.
fn fire_hydrant() -> Vec<Piece> {
    let front = |z0: f32, z1: f32, d: f32, slot| {
        span(Shape::Cylinder, Vec3::new(0.0, 0.4, z0), Vec3::new(0.0, 0.4, z1), d, slot)
    };
    vec![
        cyl(0.0, 0.0, 0.34, 0.0, 0.06, 0),
        cyl(0.0, 0.0, 0.24, 0.06, 0.74, 0),
        cyl(0.0, 0.0, 0.29, 0.7, 0.76, 0),
        ball([0.0, 0.78, 0.0], [0.25, 0.2, 0.25], 0),
        cyl(0.0, 0.0, 0.06, 0.86, 0.92, 1),
        Piece::new(Shape::Wheel, Vec3::new(0.0, 0.55, 0.0), Vec3::new(0.36, 0.1, 0.1), 0),
        Piece::new(Shape::Wheel, Vec3::new(-0.19, 0.55, 0.0), Vec3::new(0.03, 0.13, 0.13), 1),
        Piece::new(Shape::Wheel, Vec3::new(0.19, 0.55, 0.0), Vec3::new(0.03, 0.13, 0.13), 1),
        front(0.0, 0.19, 0.13, 0),
        front(0.19, 0.22, 0.15, 1),
    ]
}

fn bollard() -> Vec<Piece> {
    vec![
        cyl(0.0, 0.0, 0.14, 0.0, 0.05, 0),
        cyl(0.0, 0.0, 0.09, 0.05, 0.9, 0),
        cyl(0.0, 0.0, 0.095, 0.75, 0.81, 1),
        cyl(0.0, 0.0, 0.12, 0.88, 0.92, 0),
        ball([0.0, 0.97, 0.0], [0.11, 0.11, 0.11], 0),
    ]
}

/// Boîte aux lettres de rue sur pied, toit arrondi, fente et logo à l'avant.
fn mailbox() -> Vec<Piece> {
    vec![
        bx(0.0, 0.0, 0.25, 0.25, 0.0, 0.03, 1),
        bx(0.0, 0.0, 0.08, 0.08, 0.03, 0.85, 1),
        bx(0.0, 0.0, 0.42, 0.34, 0.85, 1.38, 0),
        Piece::new(Shape::Wheel, Vec3::new(0.0, 1.38, 0.0), Vec3::new(0.42, 0.16, 0.34), 0),
        bx(0.0, 0.172, 0.26, 0.01, 1.24, 1.27, 2),
        bx(0.0, 0.172, 0.14, 0.01, 1.0, 1.14, 3),
    ]
}

/// Horodateur : écran et clavier à l'avant, panneau solaire incliné sur le dessus.
fn parking_meter() -> Vec<Piece> {
    vec![
        bx(0.0, 0.0, 0.36, 0.3, 0.0, 0.05, 4),
        bx(0.0, 0.0, 0.24, 0.18, 0.05, 0.95, 0),
        bx(0.0, 0.0, 0.4, 0.28, 0.95, 1.55, 0),
        bx(0.0, 0.142, 0.22, 0.01, 1.32, 1.46, 1),
        bx(0.0, 0.142, 0.18, 0.01, 1.12, 1.27, 2),
        bx(0.0, 0.15, 0.14, 0.03, 1.0, 1.05, 4),
        bx(0.0, 0.0, 0.44, 0.32, 1.55, 1.58, 4),
        Piece::new(Shape::Cube, Vec3::new(0.0, 1.63, 0.0), Vec3::new(0.4, 0.02, 0.3), 3)
            .turn(Quat::from_rotation_x(0.35)),
    ]
}

/// Cabine téléphonique vitrée, porte vitrée à l'avant qui s'ouvre, combiné au fond.
fn phone_booth(open: f32) -> Vec<Piece> {
    let glass =
        |x: f32, z: f32, sx: f32, sz: f32| Piece::new(Shape::Glass, Vec3::new(x, 1.15, z), Vec3::new(sx, 2.0, sz), 0);
    let mut v = vec![
        bx(0.0, 0.0, 1.0, 1.0, 0.0, 0.1, 1),
        bx(0.0, 0.0, 1.0, 1.0, 2.15, 2.2, 0),
        bx(0.0, 0.0, 1.02, 1.02, 2.2, 2.42, 3),
        bx(0.0, 0.0, 1.1, 1.1, 2.42, 2.5, 1),
        glass(0.0, -0.47, 0.88, 0.02),
        glass(-0.47, 0.0, 0.02, 0.88),
        glass(0.47, 0.0, 0.02, 0.88),
        bx(0.0, -0.47, 0.88, 0.04, 1.0, 1.04, 0),
        bx(-0.47, 0.0, 0.04, 0.88, 1.0, 1.04, 0),
        bx(0.47, 0.0, 0.04, 0.88, 1.0, 1.04, 0),
        bx(0.0, -0.4, 0.26, 0.1, 1.15, 1.5, 2),
        bx(-0.08, -0.34, 0.05, 0.05, 1.2, 1.42, 2),
    ];
    for (x, z) in [(-0.47, -0.47), (0.47, -0.47), (-0.47, 0.47), (0.47, 0.47)] {
        v.push(bx(x, z, 0.06, 0.06, 0.1, 2.15, 0));
    }
    let [mut pane, handle] = door(open, (-0.44, 0.47), 0.88, (0.12, 2.12), 1.0, (0, 4));
    pane.shape = Shape::Glass;
    v.extend([pane, handle]);
    v
}

/// Trois arceaux en U renversé espacés de 0,9 m ; un vélo se gare de chaque côté, dans l'axe avant-arrière.
fn bike_rack() -> Vec<Piece> {
    let mut v = Vec::new();
    for x in [-0.9, 0.0, 0.9] {
        let p = |y: f32, z: f32| Vec3::new(x, y, z);
        let hoop = [p(0.0, -0.35), p(0.62, -0.35), p(0.75, -0.22), p(0.75, 0.22), p(0.62, 0.35), p(0.0, 0.35)];
        v.extend(hoop.windows(2).map(|w| span(Shape::Cylinder, w[0], w[1], 0.05, 0)));
    }
    v
}

/// Panneau publicitaire de 4 × 3 m sur un pied, affiche à l'avant éclairée par deux projecteurs.
fn billboard() -> Vec<Piece> {
    let mut v = vec![
        cyl(0.0, -0.1, 0.7, 0.0, 0.25, 3),
        cyl(0.0, -0.1, 0.35, 0.25, 3.0, 3),
        bx(0.0, -0.1, 4.2, 0.2, 2.9, 6.1, 2),
        bx(0.0, 0.005, 4.0, 0.01, 3.0, 6.0, 0),
        bx(-0.6, 0.012, 1.8, 0.005, 3.5, 4.2, 1),
        Piece::new(Shape::Cylinder, Vec3::new(1.0, 4.9, 0.012), Vec3::new(1.4, 0.005, 1.4), 1)
            .turn(Quat::from_rotation_x(FRAC_PI_2)),
    ];
    for x in [-1.3, 1.3] {
        v.push(span(Shape::Cube, Vec3::new(x, 6.1, -0.05), Vec3::new(x, 6.32, 0.4), 0.05, 4));
        v.push(bx(x, 0.45, 0.3, 0.2, 6.25, 6.4, 4));
    }
    v
}

/// Poteau électrique en bois, traverse et isolateurs ; les câbles partent vers l'avant et l'arrière.
fn utility_pole() -> Vec<Piece> {
    let mut v =
        vec![cyl(0.0, 0.0, 0.28, 0.0, 4.0, 0), cyl(0.0, 0.0, 0.22, 4.0, 8.0, 0), bx(0.0, 0.0, 2.0, 0.1, 7.3, 7.42, 1)];
    for x in [-0.5, 0.5] {
        v.push(span(Shape::Cube, Vec3::new(x, 7.3, 0.0), Vec3::new(0.0, 6.8, 0.0), 0.05, 1));
    }
    for (x, y) in [(-0.85, 7.42), (0.85, 7.42), (0.0, 8.0)] {
        v.push(cyl(x, 0.0, 0.03, y, y + 0.08, 1));
        v.push(cyl(x, 0.0, 0.12, y + 0.08, y + 0.12, 2));
        v.push(cyl(x, 0.0, 0.1, y + 0.14, y + 0.18, 2));
        v.push(ball([x, y + 0.2, 0.0], [0.07, 0.06, 0.07], 2));
    }
    v
}

/// Plaque d'égout ronde de 75 cm, à fleur de sol : cadre, tampon et quadrillage en relief.
fn manhole() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.75, 0.0, 0.012, 0), cyl(0.0, 0.0, 0.64, 0.012, 0.015, 1)];
    for o in [-0.16, 0.0, 0.16] {
        v.push(bx(o, 0.0, 0.03, 0.5, 0.015, 0.018, 0));
        v.push(bx(0.0, o, 0.5, 0.03, 0.015, 0.018, 0));
    }
    v
}

/// Kiosque à journaux : comptoir ouvert à l'avant, magazines sur les côtés, toit bombé en zinc.
fn kiosk() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, -0.85, 2.4, 0.1, 0.0, 2.4, 0),
        bx(0.0, 0.85, 2.4, 0.1, 0.0, 1.0, 0),
        bx(0.0, 0.85, 2.4, 0.1, 2.0, 2.4, 0),
        bx(-1.15, 0.0, 0.1, 1.8, 0.0, 2.4, 0),
        bx(1.15, 0.0, 0.1, 1.8, 0.0, 2.4, 0),
        bx(0.0, 0.95, 2.5, 0.3, 1.0, 1.05, 2),
        bx(0.0, 0.0, 2.9, 2.3, 2.4, 2.5, 1),
        ball([0.0, 2.5, 0.0], [2.5, 0.9, 2.0], 1),
        cyl(0.0, 0.0, 0.08, 2.9, 3.15, 0),
        ball([0.0, 3.2, 0.0], [0.14, 0.14, 0.14], 0),
        bx(0.0, 0.905, 1.8, 0.01, 2.08, 2.32, 5),
    ];
    for x in [-0.9, -0.3, 0.3, 0.9] {
        v.push(bx(x, 0.95, 0.4, 0.26, 1.05, 1.075, 3));
        v.push(bx(x * 0.9, 0.7, 0.4, 0.02, 1.5, 1.95, 4));
    }
    for x in [-1.205, 1.205] {
        for z in [-0.45, 0.15] {
            v.push(bx(x, z, 0.01, 0.5, 1.1, 1.8, 4));
        }
    }
    v
}

/// Fontaine à boire sur colonne : vasque, bec et bouton poussoir à l'avant.
fn drinking_fountain() -> Vec<Piece> {
    vec![
        cyl(0.0, 0.0, 0.4, 0.0, 0.06, 0),
        cyl(0.0, 0.0, 0.22, 0.06, 0.85, 0),
        cyl(0.0, 0.0, 0.46, 0.85, 0.95, 1),
        cyl(0.0, 0.0, 0.38, 0.95, 0.952, 2),
        cyl(0.06, 0.0, 0.03, 0.95, 1.02, 1),
        span(Shape::Cylinder, Vec3::new(0.06, 1.02, 0.0), Vec3::new(-0.04, 1.1, 0.0), 0.015, 3),
        span(Shape::Cylinder, Vec3::new(-0.04, 1.1, 0.0), Vec3::new(-0.1, 0.97, 0.0), 0.015, 3),
        bx(0.0, 0.115, 0.06, 0.02, 0.75, 0.8, 1),
    ]
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::{Category, LEVEL_H, Prop};

    #[test]
    fn street_and_signage_stand_on_the_ground_around_their_origin() {
        for k in Category::Street.kinds().chain(Category::Signage.kinds()) {
            assert!(k.outdoor(), "{k:?} se pose dehors");
            let (lo, hi) = Prop::new(k, Vec2::ZERO, 0, 0.0).local_box();
            let (c, s) = ((lo + hi) / 2.0, hi - lo);
            assert!(lo.y > -1e-3, "{k:?} sous le sol : {lo}");
            // le lampadaire garde son mât à l'origine, sa lanterne déborde au-dessus de la chaussée
            let centred = c.x.abs() < 0.1 + 0.1 * s.x && c.z.abs() < 0.1 + 0.1 * s.z;
            assert!(centred || k == PropKind::StreetLamp, "{k:?} décentré : {c}");
        }
    }
    #[test]
    fn lamps_shine_and_lids_and_doors_open() {
        for k in [PropKind::StreetLamp, PropKind::Lantern] {
            assert_eq!(shapes(k, 0.0, LEVEL_H).iter().filter(|p| p.shape == Shape::Light).count(), 1, "{k:?}");
        }
        assert!(PropKind::Dumpster.openable() && PropKind::PhoneBooth.openable());
        assert!(!PropKind::BusStop.openable() && !PropKind::Kiosk.openable() && !PropKind::TrashCan.openable());
        let lid =
            |open| shapes(PropKind::Dumpster, open, LEVEL_H).iter().map(|p| p.at.y + p.size.y).fold(0.0, f32::max);
        assert!(lid(1.0) > lid(0.0) + 0.5, "le couvercle se relève");
    }
    #[test]
    fn manhole_lies_flat_on_the_ground() {
        let (lo, hi) = Prop::new(PropKind::Manhole, Vec2::ZERO, 0, 0.0).local_box();
        assert!(PropKind::Manhole.flat() && lo.y > -1e-4 && hi.y <= 0.02, "{lo} {hi}");
    }
}
