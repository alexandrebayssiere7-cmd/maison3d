//! Cuisine ; la salle de bain est dans `bathroom`.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};

use super::{Piece, ball, bx, carcass, cyl, door};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

mod bathroom;

const FACADE: [f32; 3] = [0.88, 0.88, 0.85];
const BLACK: [f32; 3] = [0.06, 0.06, 0.07];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Kitchen => {
            const {
                &[
                    Slot::plain("Façades", FACADE),
                    Slot::plain("Plan de travail", DARK),
                    Slot::plain("Évier et poignées", METAL),
                    Slot::plain("Plaques", [0.05, 0.05, 0.05]),
                    Slot::plain("Vaisselle", WHITE),
                    Slot::plain("Casseroles", [0.6, 0.3, 0.2]),
                ]
            }
        }
        PropKind::Fridge => {
            const {
                &[
                    Slot::plain("Carrosserie", WHITE),
                    Slot::plain("Poignée", METAL),
                    Slot::plain("Intérieur", WHITE),
                    Slot::plain("Clayettes", [0.8, 0.9, 0.95]),
                    Slot::plain("Jus", [0.9, 0.5, 0.1]),
                    Slot::plain("Fromage", [0.95, 0.9, 0.6]),
                    Slot::plain("Légumes", [0.3, 0.6, 0.3]),
                ]
            }
        }
        PropKind::KitchenIsland => {
            const {
                &[
                    Slot::plain("Façades", FACADE),
                    Slot::textured("Plan de travail", "Marbre blanc"),
                    Slot::plain("Poignées", METAL),
                ]
            }
        }
        PropKind::BaseCabinet => {
            const {
                &[
                    Slot::plain("Façade", FACADE),
                    Slot::plain("Plan de travail", DARK),
                    Slot::plain("Poignée", METAL),
                    Slot::plain("Socle", DARK),
                ]
            }
        }
        PropKind::WallCabinet => {
            const { &[Slot::plain("Façade", FACADE), Slot::plain("Poignée", METAL), Slot::plain("Vaisselle", WHITE)] }
        }
        PropKind::BarStool => const { &[Slot::textured("Assise", "Cuir noir"), Slot::textured("Pied", "Chrome")] },
        PropKind::KitchenBin => {
            const {
                &[
                    Slot::textured("Cuve", "Inox brossé"),
                    Slot::plain("Couvercle", METAL),
                    Slot::plain("Pédale et socle", DARK),
                ]
            }
        }
        PropKind::HighTable => const { &[Slot::textured("Plateau", "Bois clair"), Slot::plain("Pied", DARK)] },
        _ => appliance_slots(kind),
    }
}

/// Parties de l'électroménager de cuisine.
fn appliance_slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Range => {
            const {
                &[
                    Slot::textured("Carrosserie", "Inox brossé"),
                    Slot::plain("Table de cuisson", BLACK),
                    Slot::textured("Poignée et boutons", "Chrome"),
                    Slot::plain("Intérieur du four", DARK),
                    Slot::plain("Foyers", [0.32, 0.12, 0.1]),
                    Slot::plain("Vitre du four", [0.1, 0.1, 0.12]),
                ]
            }
        }
        PropKind::OvenTower => {
            const {
                &[
                    Slot::plain("Façades", FACADE),
                    Slot::plain("Poignées et commandes", METAL),
                    Slot::plain("Intérieur du four", DARK),
                    Slot::plain("Four et micro-ondes", BLACK),
                    Slot::plain("Hublot du micro-ondes", [0.25, 0.27, 0.3]),
                ]
            }
        }
        PropKind::Dishwasher => {
            const {
                &[
                    Slot::plain("Carrosserie", WHITE),
                    Slot::plain("Bandeau", DARK),
                    Slot::plain("Paniers", [0.55, 0.57, 0.6]),
                    Slot::plain("Vaisselle", WHITE),
                    Slot::plain("Poignée", METAL),
                    Slot::textured("Cuve", "Inox brossé"),
                ]
            }
        }
        PropKind::RangeHood => {
            const {
                &[
                    Slot::textured("Inox", "Inox brossé"),
                    Slot::plain("Filtres", [0.45, 0.45, 0.48]),
                    Slot::plain("Boutons", BLACK),
                ]
            }
        }
        _ => bathroom::slots(kind),
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Kitchen => kitchen(&mut v, open),
        PropKind::Fridge => fridge(&mut v, open),
        PropKind::KitchenIsland => island(&mut v, open),
        PropKind::BaseCabinet => base_cabinet(&mut v, open),
        PropKind::WallCabinet => wall_cabinet(&mut v, open),
        PropKind::Range => range(&mut v, open),
        PropKind::OvenTower => oven_tower(&mut v, open),
        PropKind::Dishwasher => dishwasher(&mut v, open),
        PropKind::RangeHood => range_hood(&mut v),
        PropKind::BarStool => bar_stool(&mut v),
        PropKind::KitchenBin => kitchen_bin(&mut v),
        PropKind::HighTable => high_table(&mut v),
        _ => v.extend(bathroom::shapes(kind, open, height)),
    }
    v
}

// ---------------------------------------------------------------- briques des abattants

/// Pièces fermées qui pivotent ensemble de `q` autour d'une charnière passant par `hinge` :
/// abattant de four (`from_rotation_x(a)`, le haut part vers l'avant), couvercle
/// (`from_rotation_x(-a)`, l'avant se lève), hublot ou porte vitrée (`from_rotation_y`).
pub(super) fn swing<const N: usize>(pieces: [Piece; N], hinge: Vec3, q: Quat) -> [Piece; N] {
    pieces.map(|p| Piece { at: hinge + q * (p.at - hinge), ..p }.turn(q))
}

/// Pièces décalées de `d` (caisson posé plus haut ou plus en avant).
pub(super) fn shift<const N: usize>(pieces: [Piece; N], d: Vec3) -> [Piece; N] {
    pieces.map(|p| Piece { at: p.at + d, ..p })
}

/// Disque de diamètre `d` et d'épaisseur `t` tourné vers l'avant (+z) : bouton, hublot, tambour.
fn disc(at: [f32; 3], d: f32, t: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cylinder, Vec3::from(at), Vec3::new(d, t, d), slot).turn(Quat::from_rotation_x(FRAC_PI_2))
}

/// Porte qui s'abat vers l'avant autour de son bord bas (four, lave-vaisselle).
fn drop_door<const N: usize>(open: f32, pieces: [Piece; N], hinge: Vec3) -> [Piece; N] {
    swing(pieces, hinge, Quat::from_rotation_x(open * 1.5))
}

// ---------------------------------------------------------------- cuisine

fn kitchen(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(2.4, 0.6, 0.86, 0, 0));
    v.push(bx(0.0, 0.0, 2.35, 0.55, 0.42, 0.44, 0));
    v.push(bx(0.0, 0.0, 2.42, 0.62, 0.86, 0.9, 1));
    v.push(bx(0.5, 0.0, 0.6, 0.4, 0.9, 0.91, 2));
    v.push(bx(-0.6, 0.0, 0.6, 0.5, 0.9, 0.91, 3));
    // vaisselle
    v.push(bx(-0.8, 0.0, 0.3, 0.3, 0.44, 0.56, 4));
    v.push(bx(0.3, -0.05, 0.4, 0.25, 0.08, 0.3, 5));
    for (hx, side) in [(-1.2, 1.0), (0.0, -1.0), (0.0, 1.0), (1.2, -1.0)] {
        v.extend(door(open, (hx, 0.3), 0.6, (0.06, 0.84), side, (0, 2)));
    }
}

fn fridge(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.7, 0.7, 1.85, 0, 2));
    for y in [0.5, 0.9, 1.3] {
        v.push(bx(0.0, 0.0, 0.64, 0.6, y, y + 0.02, 3));
    }
    v.push(bx(-0.15, 0.0, 0.15, 0.15, 0.92, 1.1, 4));
    v.push(bx(0.12, 0.05, 0.25, 0.12, 1.32, 1.4, 5));
    v.push(bx(0.0, 0.0, 0.2, 0.15, 0.52, 0.75, 6));
    v.extend(door(open, (-0.35, 0.35), 0.7, (0.02, 1.84), 1.0, (0, 1)));
}

/// Caissons à l'avant, plan qui déborde de 40 cm à l'arrière pour les genoux des tabourets.
fn island(v: &mut Vec<Piece>, open: f32) {
    v.extend(shift(carcass(1.8, 0.6, 0.88, 0, 0), Vec3::Z * 0.15));
    v.push(bx(0.0, 0.15, 1.75, 0.55, 0.44, 0.46, 0));
    v.push(bx(0.0, -0.05, 1.9, 1.0, 0.88, 0.92, 1));
    for (hx, side) in [(-0.9, 1.0), (-0.3, 1.0), (0.9, -1.0)] {
        v.extend(door(open, (hx, 0.45), 0.6, (0.06, 0.86), side, (0, 2)));
    }
}

fn base_cabinet(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.6, 0.58, 0.86, 0, 0));
    v.push(bx(0.0, 0.0, 0.55, 0.53, 0.44, 0.46, 0));
    v.push(bx(0.0, 0.01, 0.6, 0.62, 0.86, 0.9, 1));
    v.push(bx(0.0, 0.29, 0.6, 0.02, 0.0, 0.06, 3));
    v.extend(door(open, (-0.3, 0.29), 0.6, (0.06, 0.84), 1.0, (0, 2)));
}

fn wall_cabinet(v: &mut Vec<Piece>, open: f32) {
    v.extend(shift(carcass(0.6, 0.35, 0.7, 0, 0), Vec3::Y * 1.4));
    v.push(bx(0.0, 0.0, 0.55, 0.3, 1.74, 1.76, 0));
    // pile d'assiettes et verres
    v.push(cyl(-0.12, 0.0, 0.24, 1.48, 1.56, 2));
    for x in [0.1, 0.19] {
        v.push(cyl(x, 0.0, 0.07, 1.76, 1.88, 2));
    }
    v.extend(door(open, (-0.3, 0.175), 0.6, (1.41, 2.09), 1.0, (0, 1)));
}

fn range(v: &mut Vec<Piece>, open: f32) {
    v.push(bx(0.0, 0.0, 0.6, 0.6, 0.0, 0.14, 0));
    for x in [-0.29, 0.29] {
        v.push(bx(x, 0.0, 0.02, 0.6, 0.14, 0.72, 0));
    }
    v.push(bx(0.0, -0.02, 0.56, 0.52, 0.14, 0.72, 3));
    v.push(bx(0.0, 0.0, 0.6, 0.6, 0.72, 0.88, 0));
    v.push(bx(0.0, 0.0, 0.6, 0.6, 0.88, 0.9, 1));
    for (x, z, d) in [(-0.14, 0.12, 0.2), (0.14, 0.12, 0.15), (-0.14, -0.15, 0.15), (0.14, -0.15, 0.2)] {
        v.push(cyl(x, z, d, 0.9, 0.903, 4));
    }
    for i in 0..5 {
        v.push(disc([-0.2 + i as f32 * 0.1, 0.8, 0.31], 0.04, 0.03, 2));
    }
    v.extend(drop_door(
        open,
        [
            bx(0.0, 0.315, 0.56, 0.03, 0.15, 0.71, 0),
            bx(0.0, 0.3325, 0.4, 0.005, 0.3, 0.6, 5),
            bx(0.0, 0.35, 0.44, 0.04, 0.64, 0.67, 2),
        ],
        Vec3::new(0.0, 0.15, 0.3),
    ));
}

/// Colonne : placard bas, four encastré (abattant), micro-ondes, placard haut.
fn oven_tower(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.6, 0.58, 2.2, 0, 0));
    v.extend(door(open, (-0.3, 0.29), 0.6, (0.06, 0.82), 1.0, (0, 1)));
    v.push(bx(0.0, -0.02, 0.55, 0.5, 0.84, 1.44, 2));
    v.extend(drop_door(
        open,
        [bx(0.0, 0.305, 0.58, 0.03, 0.84, 1.44, 3), bx(0.0, 0.34, 0.45, 0.04, 1.37, 1.4, 1)],
        Vec3::new(0.0, 0.84, 0.29),
    ));
    v.push(bx(0.0, 0.025, 0.55, 0.59, 1.46, 1.84, 3));
    v.push(bx(-0.06, 0.322, 0.36, 0.005, 1.5, 1.8, 4));
    v.push(bx(0.2, 0.322, 0.08, 0.005, 1.52, 1.78, 1));
    v.extend(door(open, (-0.3, 0.29), 0.6, (1.86, 2.18), 1.0, (0, 1)));
}

fn dishwasher(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(0.6, 0.6, 0.85, 0, 5));
    v.push(bx(0.0, 0.0, 0.52, 0.5, 0.12, 0.3, 2));
    v.push(bx(0.0, 0.0, 0.52, 0.5, 0.5, 0.6, 2));
    // assiettes debout en bas, tasses en haut
    for i in 0..6 {
        v.push(bx(-0.2 + i as f32 * 0.08, 0.05, 0.012, 0.24, 0.2, 0.44, 3));
    }
    for x in [-0.15, 0.0, 0.15] {
        v.push(cyl(x, -0.08, 0.08, 0.6, 0.7, 3));
    }
    v.extend(drop_door(
        open,
        [
            bx(0.0, 0.315, 0.6, 0.03, 0.02, 0.84, 0),
            bx(0.0, 0.335, 0.58, 0.012, 0.74, 0.82, 1),
            bx(0.0, 0.35, 0.3, 0.04, 0.68, 0.71, 4),
        ],
        Vec3::new(0.0, 0.02, 0.3),
    ));
}

/// Hotte cheminée murale, bas à 1,6 m (65 cm au-dessus des plaques).
fn range_hood(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 0.6, 0.5, 1.6, 1.68, 0));
    v.push(bx(0.0, -0.07, 0.42, 0.36, 1.68, 1.76, 0));
    v.push(bx(0.0, -0.125, 0.26, 0.25, 1.76, 2.4, 0));
    v.push(bx(0.0, 0.0, 0.5, 0.4, 1.595, 1.6, 1));
    for x in [-0.22, -0.16, -0.1] {
        v.push(bx(x, 0.255, 0.03, 0.01, 1.62, 1.66, 2));
    }
}

fn bar_stool(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, 0.0, 0.4, 0.0, 0.03, 1));
    v.push(cyl(0.0, 0.0, 0.05, 0.03, 0.66, 1));
    v.push(cyl(0.0, 0.0, 0.32, 0.28, 0.3, 1));
    v.push(cyl(0.0, 0.0, 0.38, 0.66, 0.72, 0));
}

fn kitchen_bin(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, 0.0, 0.31, 0.0, 0.04, 2));
    v.push(cyl(0.0, 0.0, 0.3, 0.04, 0.6, 0));
    v.push(cyl(0.0, 0.0, 0.31, 0.6, 0.63, 1));
    v.push(ball([0.0, 0.63, 0.0], [0.3, 0.06, 0.3], 1));
    v.push(bx(0.0, -0.15, 0.08, 0.04, 0.56, 0.63, 2));
    v.push(bx(0.0, 0.18, 0.12, 0.08, 0.01, 0.04, 2));
}

fn high_table(v: &mut Vec<Piece>) {
    v.push(cyl(0.0, 0.0, 0.5, 0.0, 0.03, 1));
    v.push(cyl(0.0, 0.0, 0.08, 0.03, 1.05, 1));
    v.push(cyl(0.0, 0.0, 0.36, 0.3, 0.32, 1));
    v.push(cyl(0.0, 0.0, 0.7, 1.05, 1.09, 0));
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::Prop;

    fn heights(kind: PropKind) -> (f32, f32) {
        let (lo, hi) = Prop::new(kind, Vec2::ZERO, 0, 0.0).local_box();
        (lo.y, hi.y)
    }

    #[test]
    fn doors_and_flaps_open_only_where_there_are_some() {
        use PropKind::{
            BarStool, BaseCabinet, Dishwasher, HighTable, KitchenBin, KitchenIsland, OvenTower, Range, RangeHood,
            WallCabinet,
        };
        for k in [KitchenIsland, BaseCabinet, WallCabinet, Range, OvenTower, Dishwasher] {
            assert!(k.openable(), "{k:?} s'ouvre");
        }
        for k in [RangeHood, BarStool, KitchenBin, HighTable] {
            assert!(!k.openable(), "{k:?} ne s'ouvre pas");
        }
    }
    #[test]
    fn oven_and_dishwasher_doors_drop_forward() {
        for k in [PropKind::Range, PropKind::Dishwasher, PropKind::OvenTower] {
            let lying = |open| shapes(k, open, 3.0).iter().filter(|p| (p.rot * Vec3::Y).z > 0.95).count();
            assert!(lying(1.0) >= lying(0.0) + 2, "{k:?} : la porte et sa poignée s'abattent vers l'avant");
            assert!(shapes(k, 1.0, 3.0).iter().all(|p| (p.rot * Vec3::Y).z > -0.1), "{k:?} : rien ne part en arrière");
        }
    }
    #[test]
    fn modules_align_every_60_cm_and_wall_units_hang_high() {
        for k in [PropKind::BaseCabinet, PropKind::WallCabinet, PropKind::Range, PropKind::Dishwasher] {
            assert!((k.size().x - 0.6).abs() < 0.01, "{k:?} : {}", k.size());
        }
        let (lo, hi) = heights(PropKind::WallCabinet);
        assert!((lo - 1.4).abs() < 0.01 && (hi - 2.1).abs() < 0.01, "meuble haut entre 1,4 et 2,1 m : {lo} {hi}");
        assert!(heights(PropKind::RangeHood).0 > 1.5, "la hotte est au-dessus des plaques");
    }
    #[test]
    fn swing_turns_about_the_hinge_and_shift_moves() {
        let p = bx(0.0, 0.0, 0.6, 0.02, 0.0, 1.0, 0);
        let [q] = swing([p], Vec3::ZERO, Quat::from_rotation_x(FRAC_PI_2));
        assert!(q.at.abs_diff_eq(Vec3::new(0.0, 0.0, 0.5), 1e-5), "le centre tourne autour de la charnière");
        let [s] = shift([p], Vec3::Y);
        assert_eq!((s.at.y, s.rot), (1.5, p.rot));
    }
}
