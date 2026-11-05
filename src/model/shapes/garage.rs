//! Garage et atelier.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::math::{Quat, Vec3};

use super::kitchen::{shift, swing};
use super::{Piece, ball, bx, carcass, cyl, legs, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const CARDBOARD: [f32; 3] = [0.72, 0.55, 0.35];
const RUBBER: [f32; 3] = [0.08, 0.08, 0.09];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Workbench => {
            const {
                &[
                    Slot::textured("Plateau", "Bois clair"),
                    Slot::plain("Piétement et étau", [0.2, 0.32, 0.5]),
                    Slot::plain("Panneau à outils", [0.65, 0.5, 0.33]),
                    Slot::plain("Manches", [0.75, 0.15, 0.1]),
                    Slot::plain("Outils", METAL),
                ]
            }
        }
        PropKind::ToolChest => {
            const {
                &[
                    Slot::plain("Caisson et tiroirs", [0.75, 0.1, 0.1]),
                    Slot::plain("Tapis du dessus", DARK),
                    Slot::plain("Poignées", METAL),
                    Slot::plain("Intérieur", [0.3, 0.3, 0.32]),
                    Slot::plain("Roulettes", RUBBER),
                ]
            }
        }
        PropKind::MetalShelf => {
            const {
                &[
                    Slot::plain("Montants", [0.2, 0.35, 0.6]),
                    Slot::plain("Tablettes", METAL),
                    Slot::plain("Cartons", CARDBOARD),
                    Slot::plain("Bacs", [0.25, 0.45, 0.75]),
                    Slot::plain("Pots de peinture", WHITE),
                ]
            }
        }
        PropKind::TireStack => const { &[Slot::plain("Pneus", RUBBER), Slot::plain("Jantes", METAL)] },
        PropKind::JerryCans => {
            const {
                &[
                    Slot::plain("Bidons rouges", [0.75, 0.1, 0.08]),
                    Slot::plain("Bidon vert", [0.15, 0.45, 0.2]),
                    Slot::plain("Bouchons", RUBBER),
                ]
            }
        }
        PropKind::WaterHeater => {
            const {
                &[
                    Slot::plain("Cuve", WHITE),
                    Slot::plain("Pieds", METAL),
                    Slot::plain("Eau froide", [0.2, 0.4, 0.8]),
                    Slot::plain("Eau chaude", [0.8, 0.2, 0.15]),
                    Slot::plain("Capot", [0.75, 0.75, 0.75]),
                ]
            }
        }
        PropKind::Lawnmower => {
            const {
                &[
                    Slot::plain("Carter", [0.2, 0.55, 0.2]),
                    Slot::plain("Moteur", DARK),
                    Slot::plain("Roues", RUBBER),
                    Slot::plain("Bac", [0.25, 0.27, 0.25]),
                    Slot::plain("Guidon", METAL),
                ]
            }
        }
        PropKind::Stepladder => {
            const {
                &[
                    Slot::plain("Montants", [0.78, 0.8, 0.82]),
                    Slot::plain("Marches", [0.6, 0.62, 0.65]),
                    Slot::plain("Plateau et patins", [0.9, 0.45, 0.1]),
                ]
            }
        }
        PropKind::Boxes => const { &[Slot::plain("Cartons", CARDBOARD), Slot::plain("Adhésif", [0.85, 0.75, 0.55])] },
        PropKind::ChestFreezer => {
            const {
                &[
                    Slot::plain("Carrosserie", WHITE),
                    Slot::plain("Poignée", [0.6, 0.6, 0.62]),
                    Slot::plain("Intérieur", [0.85, 0.88, 0.9]),
                    Slot::plain("Socle", DARK),
                    Slot::plain("Provisions", [0.35, 0.55, 0.85]),
                ]
            }
        }
        PropKind::CarLift => {
            const {
                &[
                    Slot::plain("Colonnes", [0.15, 0.3, 0.65]),
                    Slot::plain("Bras et chariots", [0.25, 0.25, 0.28]),
                    Slot::plain("Tampons", [0.9, 0.75, 0.1]),
                    Slot::plain("Centrale", [0.6, 0.6, 0.62]),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Workbench => workbench(&mut v),
        PropKind::ToolChest => tool_chest(&mut v, open),
        PropKind::MetalShelf => metal_shelf(&mut v),
        PropKind::TireStack => tire_stack(&mut v),
        PropKind::JerryCans => jerry_cans(&mut v),
        PropKind::WaterHeater => water_heater(&mut v),
        PropKind::Lawnmower => lawnmower(&mut v),
        PropKind::Stepladder => stepladder(&mut v),
        PropKind::Boxes => boxes(&mut v),
        PropKind::ChestFreezer => chest_freezer(&mut v, open),
        PropKind::CarLift => car_lift(&mut v),
        _ => {}
    }
    v
}

/// Établi, panneau perforé au mur (à l'arrière) et outils accrochés.
fn workbench(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.8, 0.7, 0.85, 0.9, 0));
    v.push(bx(0.0, 0.0, 1.75, 0.65, 0.15, 0.18, 0));
    v.extend(legs(0.85, 0.3, 0.85, 1));
    v.push(bx(0.72, 0.3, 0.16, 0.14, 0.9, 1.0, 1));
    v.push(bx(0.0, -0.335, 1.8, 0.02, 0.9, 1.8, 2));
    let z = -0.31;
    // marteau, scie, clés, tournevis
    v.push(bx(-0.65, z, 0.03, 0.025, 1.2, 1.5, 3));
    v.push(bx(-0.65, z, 0.13, 0.03, 1.5, 1.55, 4));
    v.push(bx(-0.25, z, 0.36, 0.005, 1.3, 1.44, 4));
    v.push(bx(-0.02, z, 0.11, 0.025, 1.3, 1.44, 3));
    for i in 0..3 {
        let f = i as f32;
        v.push(bx(0.25 + f * 0.07, z, 0.02, 0.006, 1.37 - f * 0.04, 1.55, 4));
        v.push(cyl(0.55 + f * 0.08, z, 0.03, 1.45, 1.55, 3));
        v.push(cyl(0.55 + f * 0.08, z, 0.008, 1.28, 1.45, 4));
    }
}

/// Servante à roulettes ; ses tiroirs glissent vers l'avant.
fn tool_chest(v: &mut Vec<Piece>, open: f32) {
    v.extend(shift(carcass(0.68, 0.45, 0.9, 0, 3), Vec3::Y * 0.1));
    v.push(bx(0.0, 0.0, 0.7, 0.47, 1.0, 1.03, 1));
    let dz = open * 0.3;
    for (y0, y1) in [(0.88, 0.97), (0.78, 0.87), (0.66, 0.77), (0.5, 0.65), (0.2, 0.49)] {
        v.push(bx(0.0, dz, 0.62, 0.4, y0 + 0.01, y1 - 0.01, 3));
        v.push(bx(0.0, 0.215 + dz, 0.64, 0.02, y0, y1, 0));
        v.push(bx(0.0, 0.235 + dz, 0.5, 0.02, y1 - 0.035, y1 - 0.015, 2));
    }
    for (x, z) in [(-0.28, -0.17), (0.28, -0.17), (-0.28, 0.17), (0.28, 0.17)] {
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.04, z), Vec3::new(0.03, 0.08, 0.08), 4));
        v.push(bx(x, z, 0.05, 0.05, 0.08, 0.1, 2));
    }
    v.push(span(Shape::Cylinder, Vec3::new(-0.39, 0.92, -0.15), Vec3::new(-0.39, 0.92, 0.15), 0.025, 2));
    for z in [-0.15, 0.15] {
        v.push(bx(-0.365, z, 0.05, 0.02, 0.91, 0.93, 2));
    }
}

fn metal_shelf(v: &mut Vec<Piece>) {
    v.extend(legs(0.58, 0.23, 1.9, 0));
    for y in [0.1, 0.55, 1.0, 1.45, 1.875] {
        v.push(bx(0.0, 0.0, 1.2, 0.5, y, y + 0.025, 1));
    }
    v.push(bx(-0.3, 0.0, 0.45, 0.4, 0.125, 0.45, 2));
    v.push(bx(0.25, 0.0, 0.5, 0.4, 0.125, 0.4, 2));
    v.push(bx(-0.32, 0.0, 0.4, 0.35, 1.475, 1.75, 2));
    v.push(bx(0.3, 0.0, 0.35, 0.38, 1.475, 1.7, 2));
    for x in [-0.28, 0.28] {
        v.push(bx(x, 0.0, 0.45, 0.38, 0.575, 0.82, 3));
    }
    v.push(bx(-0.15, 0.0, 0.5, 0.35, 1.025, 1.3, 3));
    for x in [0.28, 0.46] {
        v.push(cyl(x, 0.05, 0.17, 1.025, 1.22, 4));
    }
}

/// Quatre pneus couchés (roues d'axe vertical), un peu décalés, jantes comprises.
fn tire_stack(v: &mut Vec<Piece>) {
    let lie = Quat::from_rotation_z(FRAC_PI_2);
    for (i, (x, z)) in [(0.0, 0.0), (0.02, -0.015), (-0.015, 0.02), (0.01, 0.0)].into_iter().enumerate() {
        let y = 0.1 + i as f32 * 0.2;
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, y, z), Vec3::new(0.2, 0.62, 0.62), 0).turn(lie));
        v.push(cyl(x, z, 0.4, y - 0.1, y + 0.102, 1));
    }
}

fn jerry_cans(v: &mut Vec<Piece>) {
    for (x, slot) in [(-0.2, 0), (0.0, 0), (0.2, 1)] {
        v.push(bx(x, 0.0, 0.17, 0.35, 0.0, 0.44, slot));
        v.push(bx(x, -0.07, 0.04, 0.16, 0.44, 0.49, slot));
        v.push(cyl(x, 0.12, 0.05, 0.42, 0.5, 2));
    }
}

/// Ballon d'eau chaude de 200 l sur trépied, arrivées d'eau froide et chaude au-dessus.
fn water_heater(v: &mut Vec<Piece>) {
    for k in 0..3 {
        let a = k as f32 * TAU / 3.0;
        v.push(bx(0.2 * a.sin(), 0.2 * a.cos(), 0.04, 0.04, 0.0, 0.3, 1));
    }
    v.push(ball([0.0, 0.3, 0.0], [0.56, 0.16, 0.56], 0));
    v.push(cyl(0.0, 0.0, 0.56, 0.3, 1.7, 0));
    v.push(ball([0.0, 1.7, 0.0], [0.56, 0.16, 0.56], 0));
    v.push(cyl(-0.1, 0.0, 0.03, 1.72, 2.1, 2));
    v.push(cyl(0.1, 0.0, 0.03, 1.72, 2.1, 3));
    v.push(bx(0.0, 0.26, 0.2, 0.06, 0.35, 0.55, 4));
}

/// Tondeuse poussée : avant vers +z, guidon vers l'arrière.
fn lawnmower(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.25, 0.48, 0.5, 0.06, 0.26, 0));
    v.push(cyl(0.0, 0.25, 0.24, 0.26, 0.42, 1));
    v.push(cyl(0.0, 0.25, 0.14, 0.42, 0.47, 1));
    for (x, z) in [(-0.27, 0.05), (0.27, 0.05), (-0.27, 0.42), (0.27, 0.42)] {
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.1, z), Vec3::new(0.05, 0.2, 0.2), 2));
    }
    v.push(bx(0.0, -0.17, 0.4, 0.34, 0.14, 0.44, 3));
    for x in [-0.21, 0.21] {
        v.push(span(Shape::Cylinder, Vec3::new(x, 0.26, 0.0), Vec3::new(x, 0.95, -0.52), 0.025, 4));
    }
    v.push(Piece::new(Shape::Wheel, Vec3::new(0.0, 0.95, -0.52), Vec3::new(0.45, 0.03, 0.03), 4));
}

/// Escabeau ouvert : montants à marches devant, pieds d'appui derrière, plateau en haut.
fn stepladder(v: &mut Vec<Piece>) {
    let front = |y: f32| 0.32 - 0.24 * y / 1.4;
    let back = |y: f32| -0.38 + 0.32 * y / 1.4;
    for x in [-0.21, 0.21] {
        v.push(span(Shape::Cube, Vec3::new(x, 0.0, front(0.0)), Vec3::new(x * 0.9, 1.4, front(1.4)), 0.04, 0));
        v.push(span(Shape::Cube, Vec3::new(x, 0.0, back(0.0)), Vec3::new(x * 0.9, 1.4, back(1.4)), 0.035, 0));
        v.push(span(
            Shape::Cube,
            Vec3::new(x * 0.96, 0.55, front(0.55)),
            Vec3::new(x * 0.96, 0.55, back(0.55)),
            0.015,
            0,
        ));
        for z in [front(0.0), back(0.0)] {
            v.push(bx(x, z, 0.05, 0.06, 0.0, 0.03, 2));
        }
    }
    for k in 1..5 {
        let y = k as f32 * 0.28;
        v.push(bx(0.0, front(y) - 0.02, 0.38, 0.09, y - 0.025, y, 1));
    }
    v.push(bx(0.0, -0.02, 0.42, 0.22, 1.4, 1.46, 2));
}

/// Pile de cartons scotchés, ceux du haut un peu de travers.
fn boxes(v: &mut Vec<Piece>) {
    let stack = [
        (-0.25, 0.0, 0.5, 0.4, 0.0, 0.4, 0.0),
        (0.27, 0.05, 0.42, 0.36, 0.0, 0.34, 0.0),
        (-0.22, 0.02, 0.4, 0.32, 0.4, 0.68, 0.25),
        (0.25, 0.04, 0.3, 0.26, 0.34, 0.56, -0.15),
    ];
    for (x, z, sx, sz, y0, y1, a) in stack {
        v.push(bx(x, z, sx, sz, y0, y1, 0).yaw(a));
        v.push(bx(x, z, 0.06, sz + 0.004, y1, y1 + 0.003, 1).yaw(a));
    }
}

/// Coffre congélateur, couvercle articulé à l'arrière qui se lève par l'avant.
fn chest_freezer(v: &mut Vec<Piece>, open: f32) {
    let (w, d) = (1.1, 0.65);
    v.push(bx(0.0, 0.0, w - 0.02, d - 0.02, 0.0, 0.05, 3));
    v.push(bx(0.0, 0.0, w, d, 0.05, 0.12, 0));
    for z in [-1.0, 1.0] {
        v.push(bx(0.0, z * (d / 2.0 - 0.025), w, 0.05, 0.12, 0.82, 0));
    }
    for x in [-1.0, 1.0] {
        v.push(bx(x * (w / 2.0 - 0.025), 0.0, 0.05, d - 0.1, 0.12, 0.82, 0));
    }
    v.push(bx(0.0, 0.0, w - 0.1, d - 0.1, 0.12, 0.14, 2));
    v.push(bx(-0.28, 0.0, 0.36, 0.42, 0.14, 0.4, 4));
    v.push(bx(0.12, -0.06, 0.3, 0.3, 0.14, 0.3, 4));
    v.push(bx(0.36, 0.08, 0.2, 0.25, 0.14, 0.34, 4));
    v.extend(swing(
        [bx(0.0, 0.0, w, d, 0.82, 0.88, 0), bx(0.0, d / 2.0 + 0.015, 0.25, 0.03, 0.8, 0.86, 1)],
        Vec3::new(0.0, 0.88, -d / 2.0),
        Quat::from_rotation_x(-open * 1.3),
    ));
}

/// Pont deux colonnes de 2,8 m ; bras repliés le long des colonnes pour laisser entrer la voiture
/// par l'avant (+z).
fn car_lift(v: &mut Vec<Piece>) {
    for s in [-1.0, 1.0] {
        let x = s * 1.55;
        v.push(bx(x, 0.0, 0.55, 0.65, 0.0, 0.02, 0));
        v.push(bx(x, 0.0, 0.3, 0.36, 0.02, 2.8, 0));
        v.push(bx(x - s * 0.2, 0.0, 0.1, 0.3, 0.1, 0.7, 1));
        for zs in [-1.0, 1.0] {
            v.push(span(
                Shape::Cube,
                Vec3::new(x - s * 0.22, 0.13, 0.0),
                Vec3::new(s * 1.05, 0.13, zs * 0.95),
                0.09,
                1,
            ));
            v.push(cyl(s * 1.05, zs * 0.95, 0.14, 0.08, 0.19, 2));
        }
    }
    v.push(bx(0.0, 0.0, 3.4, 0.16, 2.68, 2.8, 0));
    v.push(bx(1.84, 0.1, 0.22, 0.25, 1.0, 1.45, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawers_and_lid_open_only_where_there_are_some() {
        assert!(PropKind::ToolChest.openable() && PropKind::ChestFreezer.openable());
        for k in [PropKind::Workbench, PropKind::TireStack, PropKind::Boxes, PropKind::CarLift] {
            assert!(!k.openable(), "{k:?} ne s'ouvre pas");
        }
    }
    #[test]
    fn freezer_lid_rises_and_drawers_slide_out() {
        let top = |open| shapes(PropKind::ChestFreezer, open, 3.0).iter().map(|p| p.at.y).fold(f32::MIN, f32::max);
        assert!(top(1.0) > top(0.0) + 0.2, "le couvercle se lève");
        let front = |open| shapes(PropKind::ToolChest, open, 3.0).iter().map(|p| p.at.z).fold(f32::MIN, f32::max);
        assert!((front(1.0) - front(0.0) - 0.3).abs() < 1e-4, "les tiroirs sortent de 30 cm");
    }
    #[test]
    fn tires_lie_flat_and_ladder_legs_lean() {
        let wheels: Vec<_> =
            shapes(PropKind::TireStack, 0.0, 3.0).into_iter().filter(|p| p.shape == Shape::Wheel).collect();
        assert_eq!(wheels.len(), 4);
        assert!(wheels.iter().all(|p| (p.rot * Vec3::X).y.abs() > 0.99), "axe des pneus vertical");
        let leaning = shapes(PropKind::Stepladder, 0.0, 3.0).iter().filter(|p| (p.rot * Vec3::Y).y < 0.99).count();
        assert!(leaning >= 4, "montants et pieds penchés");
    }
}
