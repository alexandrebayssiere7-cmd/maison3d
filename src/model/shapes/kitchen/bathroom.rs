//! Salle de bain : douches, lavabos, miroirs et linge.

use bevy::math::{Quat, Vec3};

use super::super::{Piece, ball, bx, carcass, cyl, door, span, swing};
use super::disc;
use crate::model::{METAL, PropKind, Shape, Slot, WHITE};

const MIRROR: [f32; 3] = [0.78, 0.85, 0.9];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Bathtub => const { &[Slot::plain("Émail", WHITE), Slot::plain("Eau", [0.55, 0.75, 0.9])] },
        PropKind::Toilet => const { &[Slot::plain("Céramique", WHITE)] },
        PropKind::WalkInShower => {
            const {
                &[
                    Slot::plain("Receveur", [0.85, 0.85, 0.83]),
                    Slot::plain("Caniveau", METAL),
                    Slot::textured("Colonne et barres", "Chrome"),
                ]
            }
        }
        PropKind::ShowerCabin => {
            const {
                &[
                    Slot::plain("Receveur", WHITE),
                    Slot::plain("Paroi du fond", [0.92, 0.92, 0.9]),
                    Slot::textured("Profilés et robinetterie", "Chrome"),
                ]
            }
        }
        PropKind::Washbasin => {
            const {
                &[
                    Slot::plain("Céramique", WHITE),
                    Slot::plain("Fond de vasque", [0.84, 0.86, 0.88]),
                    Slot::textured("Robinet", "Chrome"),
                ]
            }
        }
        PropKind::VanityUnit => {
            const {
                &[
                    Slot::textured("Façades", "Bois clair"),
                    Slot::textured("Plan", "Marbre blanc"),
                    Slot::plain("Poignées", METAL),
                    Slot::plain("Vasque", WHITE),
                    Slot::textured("Robinet", "Chrome"),
                    Slot::plain("Miroir", MIRROR),
                    Slot::plain("Serviettes", [0.55, 0.7, 0.85]),
                    Slot::plain("Fond de vasque", [0.84, 0.86, 0.88]),
                ]
            }
        }
        PropKind::Mirror => const { &[Slot::plain("Miroir", MIRROR), Slot::textured("Cadre", "Bois foncé")] },
        PropKind::TowelRadiator => const { &[Slot::plain("Tubes", WHITE), Slot::plain("Serviette", [0.85, 0.4, 0.3])] },
        PropKind::WashingMachine => {
            const {
                &[
                    Slot::plain("Carrosserie", WHITE),
                    Slot::textured("Hublot et bouton", "Chrome"),
                    Slot::plain("Vitre du hublot", [0.2, 0.25, 0.3]),
                    Slot::textured("Tambour", "Inox brossé"),
                    Slot::plain("Bandeau", [0.82, 0.83, 0.85]),
                ]
            }
        }
        PropKind::Hamper => {
            const {
                &[
                    Slot::plain("Osier", [0.7, 0.55, 0.35]),
                    Slot::plain("Bordure et anses", [0.5, 0.36, 0.22]),
                    Slot::plain("Linge", [0.6, 0.75, 0.9]),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Bathtub => {
            v.push(bx(0.0, 0.0, 0.75, 1.7, 0.0, 0.55, 0));
            v.push(bx(0.0, 0.0, 0.6, 1.5, 0.5, 0.56, 1));
        }
        PropKind::Toilet => {
            v.push(bx(0.0, 0.05, 0.4, 0.5, 0.0, 0.42, 0));
            v.push(bx(0.0, -0.22, 0.4, 0.18, 0.42, 0.8, 0));
        }
        PropKind::WalkInShower => walk_in_shower(&mut v),
        PropKind::ShowerCabin => shower_cabin(&mut v, open),
        PropKind::Washbasin => washbasin(&mut v),
        PropKind::VanityUnit => vanity_unit(&mut v, open),
        PropKind::Mirror => {
            v.push(bx(0.0, 0.0, 0.66, 0.025, 1.07, 1.93, 1));
            v.push(bx(0.0, 0.005, 0.6, 0.025, 1.1, 1.9, 0));
        }
        PropKind::TowelRadiator => towel_radiator(&mut v),
        PropKind::WashingMachine => washing_machine(&mut v, open),
        PropKind::Hamper => hamper(&mut v),
        _ => {}
    }
    v
}

/// Mitigeur, colonne et pommeau de tête posés contre le mur du fond (z = `wall`).
fn shower_column(v: &mut Vec<Piece>, x: f32, wall: f32, head: f32, slot: usize) {
    v.push(bx(x, wall, 0.18, 0.06, 1.0, 1.08, slot));
    v.push(cyl(x, wall, 0.025, 1.08, 2.05, slot));
    v.push(span(Shape::Cylinder, Vec3::new(x, 2.03, wall), Vec3::new(x, 2.03, head), 0.02, slot));
    v.push(cyl(x, head, 0.25, 2.0, 2.02, slot));
}

/// Receveur extra-plat, paroi fixe à l'avant gauche tenue au mur par une barre, entrée à droite.
fn walk_in_shower(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.4, 0.9, 0.0, 0.03, 0));
    v.push(bx(0.0, -0.38, 0.8, 0.06, 0.03, 0.034, 1));
    v.push(Piece::new(Shape::Glass, Vec3::new(-0.3, 1.03, 0.43), Vec3::new(0.8, 2.0, 0.01), 0));
    v.push(bx(-0.3, 0.43, 0.8, 0.03, 0.03, 0.06, 2));
    v.push(span(Shape::Cylinder, Vec3::new(-0.68, 2.0, 0.43), Vec3::new(-0.68, 2.0, -0.45), 0.02, 2));
    shower_column(v, 0.3, -0.42, -0.15, 2);
}

fn shower_cabin(v: &mut Vec<Piece>, open: f32) {
    v.push(bx(0.0, 0.0, 0.9, 0.9, 0.0, 0.15, 0));
    v.push(bx(0.0, -0.44, 0.9, 0.02, 0.15, 2.1, 1));
    for x in [-0.44, 0.44] {
        v.push(Piece::new(Shape::Glass, Vec3::new(x, 1.12, 0.0), Vec3::new(0.01, 1.94, 0.86), 0));
        v.push(bx(x, 0.0, 0.03, 0.9, 2.07, 2.1, 2));
        v.push(bx(x, 0.435, 0.03, 0.03, 0.15, 2.07, 2));
    }
    v.push(bx(0.0, 0.435, 0.9, 0.03, 2.07, 2.1, 2));
    shower_column(v, 0.0, -0.42, -0.3, 2);
    v.extend(swing(
        [
            Piece::new(Shape::Glass, Vec3::new(0.0, 1.11, 0.44), Vec3::new(0.82, 1.9, 0.01), 0),
            bx(0.34, 0.47, 0.02, 0.04, 0.95, 1.3, 2),
        ],
        Vec3::new(-0.42, 0.0, 0.44),
        Quat::from_rotation_y(-open * 1.6),
    ));
}

/// Mitigeur posé en `z` sur un plan à la hauteur `y`, bec haut de `rise`.
fn faucet(v: &mut Vec<Piece>, z: f32, y: f32, rise: f32, slot: usize) {
    v.push(cyl(0.0, z, 0.04, y, y + rise, slot));
    v.push(span(
        Shape::Cylinder,
        Vec3::new(0.0, y + rise - 0.02, z),
        Vec3::new(0.0, y + rise - 0.04, z + 0.12),
        0.025,
        slot,
    ));
    v.push(bx(0.0, z - 0.02, 0.02, 0.08, y + rise, y + rise + 0.02, slot));
}

fn washbasin(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, -0.1, 0.2, 0.0, 0.72, 0));
    v.push(ball([0.0, 0.78, 0.02], [0.52, 0.2, 0.4], 0));
    v.push(bx(0.0, 0.0, 0.56, 0.45, 0.8, 0.86, 0));
    v.push(cyl(0.0, 0.04, 0.34, 0.86, 0.862, 1));
    faucet(v, -0.18, 0.86, 0.14, 2);
}

/// Meuble à deux portes, vasque à poser, mitigeur haut et miroir au mur au-dessus.
fn vanity_unit(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.8, 0.46, 0.82, 0, 0));
    v.push(bx(0.0, 0.0, 0.75, 0.4, 0.4, 0.42, 0));
    v.push(bx(-0.2, 0.0, 0.3, 0.3, 0.42, 0.52, 6));
    v.push(bx(0.0, 0.0, 0.82, 0.48, 0.82, 0.86, 1));
    v.push(cyl(0.0, 0.05, 0.4, 0.86, 0.99, 3));
    v.push(cyl(0.0, 0.05, 0.34, 0.99, 0.992, 7));
    faucet(v, -0.18, 0.86, 0.29, 4);
    v.push(bx(0.0, -0.23, 0.7, 0.02, 1.15, 1.85, 5));
    v.extend(door(open, (-0.4, 0.23), 0.4, (0.06, 0.8), 1.0, (0, 2)));
    v.extend(door(open, (0.4, 0.23), 0.4, (0.06, 0.8), -1.0, (0, 2)));
}

/// Échelle de tubes fixée au mur, une serviette pliée sur les barreaux.
fn towel_radiator(v: &mut Vec<Piece>) {
    for x in [-0.25, 0.25] {
        v.push(cyl(x, 0.02, 0.035, 0.25, 1.5, 0));
        for y in [0.4, 1.35] {
            v.push(bx(x, -0.025, 0.02, 0.07, y - 0.01, y + 0.01, 0));
        }
    }
    for i in 0..12 {
        v.push(Piece::new(Shape::Wheel, Vec3::new(0.0, 0.3 + i as f32 * 0.1, 0.02), Vec3::new(0.5, 0.022, 0.022), 0));
    }
    v.push(bx(0.0, 0.05, 0.36, 0.02, 0.85, 1.25, 1));
    v.push(bx(0.0, 0.02, 0.36, 0.08, 1.24, 1.27, 1));
}

/// Lave-linge à hublot, charnière à gauche.
fn washing_machine(v: &mut Vec<Piece>, open: f32) {
    v.push(bx(0.0, -0.01, 0.6, 0.58, 0.0, 0.85, 0));
    v.push(bx(0.0, 0.285, 0.58, 0.01, 0.74, 0.83, 4));
    v.push(bx(-0.18, 0.29, 0.16, 0.012, 0.76, 0.81, 0));
    v.push(disc([0.18, 0.785, 0.29], 0.06, 0.025, 1));
    v.push(disc([0.0, 0.42, 0.285], 0.34, 0.01, 3));
    v.extend(swing(
        [disc([0.0, 0.42, 0.305], 0.42, 0.04, 1), disc([0.0, 0.42, 0.33], 0.3, 0.012, 2)],
        Vec3::new(-0.21, 0.42, 0.3),
        Quat::from_rotation_y(-open * 1.6),
    ));
}

fn hamper(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, 0.0, 0.4, 0.0, 0.55, 0));
    v.push(cyl(0.0, 0.0, 0.42, 0.55, 0.6, 1));
    for x in [-0.21, 0.21] {
        v.push(bx(x, 0.0, 0.03, 0.1, 0.45, 0.52, 1));
    }
    v.push(ball([0.02, 0.6, 0.0], [0.34, 0.14, 0.32], 2));
    v.push(bx(0.08, 0.2, 0.14, 0.02, 0.4, 0.62, 2));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doors_open_only_where_there_are_some() {
        for k in [PropKind::ShowerCabin, PropKind::VanityUnit, PropKind::WashingMachine] {
            assert!(k.openable(), "{k:?} s'ouvre");
        }
        for k in [PropKind::WalkInShower, PropKind::Washbasin, PropKind::Mirror, PropKind::Hamper] {
            assert!(!k.openable(), "{k:?} ne s'ouvre pas");
        }
    }
    #[test]
    fn porthole_and_cabin_door_swing_outwards() {
        for k in [PropKind::WashingMachine, PropKind::ShowerCabin] {
            let front = |open| shapes(k, open, 3.0).iter().map(|p| p.at.z).fold(f32::MIN, f32::max);
            assert!(front(1.0) > front(0.0) + 0.1, "{k:?} : la porte s'ouvre vers l'avant");
        }
    }
    #[test]
    fn showers_have_glass_and_the_mirror_hangs_thin() {
        for k in [PropKind::WalkInShower, PropKind::ShowerCabin] {
            assert!(shapes(k, 0.0, 3.0).iter().any(|p| p.shape == Shape::Glass), "{k:?} vitrée");
        }
        let s = PropKind::Mirror.size();
        assert!(s.y < 0.05 && !PropKind::Mirror.flat(), "miroir mince mais debout : {s}");
    }
}
