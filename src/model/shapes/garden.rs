//! Jardin, terrasse et jeux. Les briques propres aux cabanes (toit à deux pans, pignons, fenêtre
//! plaquée, bordure) servent aux deux catégories.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::{Quat, Vec3};

use super::{Piece, ball, bx, clumps, cyl, door, light, span};
use crate::model::{DARK, LEAF, METAL, PropKind, Rgb, Shape, Slot, WHITE};

const WOOD: Rgb = [0.6, 0.42, 0.26];
const DARK_WOOD: Rgb = [0.42, 0.29, 0.18];
const SOIL: Rgb = [0.33, 0.22, 0.14];
const HOLE: Rgb = [0.08, 0.06, 0.05];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Hedge => const { &[Slot::textured("Feuillage", "Feuillage")] },
        PropKind::Fence => const { &[Slot::plain("Bois", [0.6, 0.45, 0.3])] },
        PropKind::PicketFence => const { &[Slot::plain("Lattes", WHITE), Slot::plain("Poteaux et traverses", WHITE)] },
        PropKind::LowWall => {
            const { &[Slot::textured("Pierres", "Moellons"), Slot::textured("Chaperon", "Pierre grise")] }
        }
        PropKind::Gate => {
            const {
                &[
                    Slot::plain("Battants", [0.16, 0.22, 0.2]),
                    Slot::textured("Piliers", "Pierre de taille"),
                    Slot::plain("Chapeaux", [0.85, 0.83, 0.78]),
                ]
            }
        }
        PropKind::GardenShed => {
            const {
                &[
                    Slot::textured("Bardage", "Bardage bois"),
                    Slot::textured("Toit", "Bardeaux bois"),
                    Slot::plain("Porte", [0.3, 0.45, 0.35]),
                    Slot::plain("Poignée", METAL),
                    Slot::textured("Plancher", "Plancher brut"),
                    Slot::plain("Fenêtre", [0.35, 0.45, 0.55]),
                    Slot::plain("Encadrement", WHITE),
                ]
            }
        }
        PropKind::Greenhouse => {
            const {
                &[
                    Slot::plain("Armature", [0.92, 0.93, 0.92]),
                    Slot::textured("Socle", "Brique rouge"),
                    Slot::textured("Tablettes", "Bois clair"),
                    Slot::textured("Plantes", "Feuillage"),
                ]
            }
        }
        PropKind::Pergola => const { &[Slot::plain("Poteaux", WOOD), Slot::plain("Poutres", WOOD)] },
        PropKind::Well => {
            const {
                &[
                    Slot::textured("Pierres", "Moellons"),
                    Slot::textured("Margelle", "Pierre de taille"),
                    Slot::plain("Charpente", DARK_WOOD),
                    Slot::textured("Toit", "Tuiles terre cuite"),
                    Slot::plain("Seau", METAL),
                    Slot::plain("Corde", [0.8, 0.7, 0.5]),
                    Slot::plain("Eau", [0.1, 0.2, 0.25]),
                ]
            }
        }
        PropKind::Fountain => {
            const {
                &[
                    Slot::textured("Pierre", "Pierre de taille"),
                    Slot::plain("Eau", [0.35, 0.6, 0.8]),
                    Slot::plain("Jet d'eau", [0.8, 0.9, 1.0]),
                ]
            }
        }
        PropKind::Pond => {
            const {
                &[
                    Slot::plain("Eau", [0.25, 0.45, 0.55]),
                    Slot::textured("Pierres", "Pierre grise"),
                    Slot::plain("Nénuphars", [0.3, 0.55, 0.25]),
                    Slot::plain("Roseaux", [0.45, 0.55, 0.3]),
                ]
            }
        }
        _ => decor_slots(kind),
    }
}

/// Parties des petits objets du jardin (plantations, décor, animaux).
fn decor_slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Planter => {
            const {
                &[
                    Slot::textured("Bac", "Bois foncé"),
                    Slot::plain("Terre", SOIL),
                    Slot::plain("Feuillage", LEAF),
                    Slot::plain("Fleurs", [0.9, 0.25, 0.45]),
                ]
            }
        }
        PropKind::VegetablePatch => {
            const {
                &[
                    Slot::textured("Bordure", "Bois clair"),
                    Slot::plain("Terre", SOIL),
                    Slot::plain("Salades", [0.55, 0.8, 0.3]),
                    Slot::plain("Plants de tomates", LEAF),
                    Slot::plain("Tomates", [0.85, 0.12, 0.08]),
                    Slot::plain("Tuteurs", [0.75, 0.65, 0.42]),
                ]
            }
        }
        PropKind::SteppingStones => const { &[Slot::textured("Pierres", "Pierre grise")] },
        PropKind::StoneLantern => {
            const { &[Slot::textured("Pierre", "Pierre grise"), Slot::plain("Lumière", [1.0, 0.8, 0.45])] }
        }
        PropKind::Torii => {
            const { &[Slot::plain("Bois peint", [0.78, 0.16, 0.1]), Slot::plain("Chapeau et socles", DARK)] }
        }
        PropKind::Woodpile => const { &[Slot::textured("Bûches", "Écorce"), Slot::plain("Piquets", DARK_WOOD)] },
        PropKind::Wheelbarrow => {
            const {
                &[
                    Slot::plain("Bac", [0.2, 0.5, 0.3]),
                    Slot::plain("Châssis", DARK),
                    Slot::plain("Roue", [0.08, 0.08, 0.09]),
                    Slot::plain("Poignées", [0.6, 0.15, 0.1]),
                ]
            }
        }
        PropKind::GardenGnome => {
            const {
                &[
                    Slot::plain("Bonnet", [0.8, 0.1, 0.1]),
                    Slot::plain("Veste", [0.15, 0.3, 0.65]),
                    Slot::plain("Visage", [0.95, 0.75, 0.6]),
                    Slot::plain("Barbe", WHITE),
                    Slot::plain("Bottes", [0.3, 0.2, 0.12]),
                ]
            }
        }
        PropKind::Doghouse => {
            const {
                &[
                    Slot::plain("Murs", [0.78, 0.6, 0.4]),
                    Slot::plain("Toit", [0.6, 0.18, 0.12]),
                    Slot::plain("Entrée", HOLE),
                    Slot::plain("Gamelle", [0.2, 0.4, 0.8]),
                ]
            }
        }
        PropKind::Birdhouse => {
            const {
                &[
                    Slot::plain("Nichoir", [0.75, 0.58, 0.38]),
                    Slot::plain("Toit", [0.35, 0.45, 0.3]),
                    Slot::plain("Poteau", DARK_WOOD),
                    Slot::plain("Entrée", HOLE),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    match kind {
        PropKind::Hedge => hedge(),
        PropKind::Fence => fence(),
        PropKind::PicketFence => picket_fence(),
        PropKind::LowWall => vec![bx(0.0, 0.0, 2.0, 0.35, 0.0, 0.55, 0), bx(0.0, 0.0, 2.0, 0.42, 0.55, 0.62, 1)],
        PropKind::Gate => gate(open),
        PropKind::GardenShed => {
            let mut v = cabin((2.4, 2.0), (2.0, 2.6), (0.0, 0.9, 1.85), open);
            v.extend(window(Vec3::new(1.21, 1.3, 0.0), (0.6, 0.45), FRAC_PI_2, (5, 6)));
            v
        }
        PropKind::Greenhouse => greenhouse(),
        PropKind::Pergola => pergola(),
        PropKind::Well => well(),
        PropKind::Fountain => fountain(),
        PropKind::Pond => pond(),
        PropKind::Planter => planter(),
        PropKind::VegetablePatch => vegetable_patch(),
        PropKind::SteppingStones => stepping_stones(),
        PropKind::StoneLantern => stone_lantern(),
        PropKind::Torii => torii(),
        PropKind::Woodpile => woodpile(),
        PropKind::Wheelbarrow => wheelbarrow(),
        PropKind::GardenGnome => gnome(),
        PropKind::Doghouse => doghouse(),
        PropKind::Birdhouse => birdhouse(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------- briques communes

/// Pans d'un toit à deux pentes dont le faîtage suit z : murs écartés de `w` (en x) et hauts de
/// `wall`, faîte à `ridge`, débord `o` sur les côtés, longueur `d` (débords avant et arrière compris).
fn gable_roof(w: f32, d: f32, (wall, ridge): (f32, f32), o: f32, slot: usize) -> [Piece; 2] {
    let (s, hw, t) = ((ridge - wall) / (w / 2.0), w / 2.0 + o, 0.05);
    let len = hw * (1.0 + s * s).sqrt();
    [-1.0, 1.0].map(|side: f32| {
        let at = Vec3::new(side * hw / 2.0, ridge - s * hw / 2.0 + t / 2.0, 0.0);
        Piece::new(Shape::Cube, at, Vec3::new(len, t, d), slot).turn(Quat::from_rotation_z(-side * s.atan()))
    })
}

/// Pignon triangulaire de largeur `w` et d'épaisseur `t`, entre `wall` et `ridge`, à la profondeur `z`.
fn gable(w: f32, z: f32, (wall, ridge): (f32, f32), t: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cone, Vec3::new(0.0, f32::midpoint(wall, ridge), z), Vec3::new(w, ridge - wall, t), slot)
}

/// Quatre planches (ou murets) d'épaisseur `t` qui bordent un rectangle `w` × `d`, entre `ys`.
fn border(w: f32, d: f32, t: f32, (y0, y1): (f32, f32), slot: usize) -> [Piece; 4] {
    let (x, z) = ((w - t) / 2.0, (d - t) / 2.0);
    [
        bx(0.0, -z, w, t, y0, y1, slot),
        bx(0.0, z, w, t, y0, y1, slot),
        bx(-x, 0.0, t, d - 2.0 * t, y0, y1, slot),
        bx(x, 0.0, t, d - 2.0 * t, y0, y1, slot),
    ]
}

/// Cabane à toit à deux pans, faîtage de l'avant vers l'arrière : `(w, d)` emprise des murs,
/// `(wall, ridge)` hauteurs, `(x, largeur, hauteur)` de la porte en façade, qui s'ouvre vers
/// l'extérieur. Parties : 0 murs, 1 toit, 2 porte, 3 poignée, 4 plancher.
fn cabin((w, d): (f32, f32), (wall, ridge): (f32, f32), (dx, dw, dh): (f32, f32, f32), open: f32) -> Vec<Piece> {
    let (t, hw) = (0.06, w / 2.0);
    let (fz, l, r) = (d / 2.0 - t / 2.0, dx - dw / 2.0, dx + dw / 2.0);
    let mut v = vec![
        bx(0.0, -fz, w, t, 0.0, wall, 0),
        bx(-hw + t / 2.0, 0.0, t, d, 0.0, wall, 0),
        bx(hw - t / 2.0, 0.0, t, d, 0.0, wall, 0),
        bx((l - hw) / 2.0, fz, l + hw, t, 0.0, wall, 0),
        bx(f32::midpoint(r, hw), fz, hw - r, t, 0.0, wall, 0),
        bx(dx, fz, dw, t, dh, wall, 0),
        gable(w, -fz, (wall, ridge), t, 0),
        gable(w, fz, (wall, ridge), t, 0),
        bx(0.0, 0.0, w - 2.0 * t, d - 2.0 * t, 0.0, 0.04, 4),
    ];
    v.extend(gable_roof(w, d + 0.3, (wall, ridge), 0.15, 1));
    v.extend(door(open, (l, d / 2.0), dw, (0.02, dh - 0.02), 1.0, (2, 3)));
    v
}

/// Fenêtre plaquée sur un mur : vitre `(w, h)` centrée en `at`, face tournée de `yaw` (0 = vers
/// +z), son encadrement et son meneau. Parties `(vitre, encadrement)`.
fn window(at: Vec3, (w, h): (f32, f32), yaw: f32, (pane, frame): (usize, usize)) -> Vec<Piece> {
    let (q, f) = (Quat::from_rotation_y(yaw), 0.05);
    [
        (Vec3::ZERO, Vec3::new(w, h, 0.02), pane),
        (Vec3::new(0.0, f32::midpoint(h, f), 0.01), Vec3::new(w + 2.0 * f, f, 0.03), frame),
        (Vec3::new(0.0, -f32::midpoint(h, f), 0.01), Vec3::new(w + 2.0 * f, f, 0.03), frame),
        (Vec3::new(-f32::midpoint(w, f), 0.0, 0.01), Vec3::new(f, h, 0.03), frame),
        (Vec3::new(f32::midpoint(w, f), 0.0, 0.01), Vec3::new(f, h, 0.03), frame),
        (Vec3::new(0.0, 0.0, 0.01), Vec3::new(0.03, h, 0.03), frame),
    ]
    .into_iter()
    .map(|(o, s, slot)| Piece::new(Shape::Cube, at + q * o, s, slot).yaw(yaw))
    .collect()
}

/// Pièces `v` pivotées de `a` radians autour de la verticale qui passe par `hinge` (battants).
fn pivot(v: Vec<Piece>, hinge: Vec3, a: f32) -> impl Iterator<Item = Piece> {
    let q = Quat::from_rotation_y(a);
    v.into_iter().map(move |p| Piece { at: hinge + q * (p.at - hinge), ..p }.yaw(a))
}

// ---------------------------------------------------------------- jardin

fn hedge() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 2.0, 0.7, 0.0, 1.2, 0)];
    v.extend(clumps(
        &[
            ([-0.6, 1.2, 0.0], [0.9, 0.45, 0.75]),
            ([0.0, 1.25, 0.0], [0.95, 0.45, 0.75]),
            ([0.6, 1.2, 0.0], [0.9, 0.45, 0.75]),
        ],
        0,
    ));
    v
}

fn fence() -> Vec<Piece> {
    let mut v: Vec<Piece> = [-1.0, 0.0, 1.0].map(|x| bx(x, 0.0, 0.08, 0.08, 0.0, 1.0, 0)).to_vec();
    v.push(bx(0.0, 0.0, 2.0, 0.04, 0.35, 0.45, 0));
    v.push(bx(0.0, 0.0, 2.0, 0.04, 0.75, 0.85, 0));
    v
}

/// Lattes pointues de 1 m devant deux traverses ; les poteaux des bouts tombent sur ceux des
/// modules voisins.
fn picket_fence() -> Vec<Piece> {
    let mut v = vec![bx(0.0, -0.03, 2.0, 0.025, 0.25, 0.33, 1), bx(0.0, -0.03, 2.0, 0.025, 0.62, 0.7, 1)];
    for x in [-1.0, 1.0] {
        v.push(bx(x, -0.03, 0.08, 0.08, 0.0, 0.95, 1));
    }
    for k in 0..10 {
        let x = -0.81 + k as f32 * 0.18;
        v.push(bx(x, 0.0, 0.09, 0.02, 0.05, 0.9, 0));
        v.push(Piece::new(Shape::Cone, Vec3::new(x, 0.95, 0.0), Vec3::new(0.09, 0.1, 0.02), 0));
    }
    v
}

/// Deux battants à barreaux de 1,5 m entre deux piliers ; ils s'ouvrent vers l'arrière (-z).
fn gate(open: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    for x in [-1.7, 1.7] {
        v.push(bx(x, 0.0, 0.4, 0.4, 0.0, 1.7, 1));
        v.push(bx(x, 0.0, 0.48, 0.48, 1.7, 1.78, 2));
    }
    for side in [1.0, -1.0] {
        let (hx, w) = (-1.5 * side, 1.48);
        let at = |u: f32| hx + side * u;
        let mut leaf = vec![bx(at(w / 2.0), 0.0, w, 0.05, 0.15, 0.25, 0), bx(at(w / 2.0), 0.0, w, 0.05, 1.25, 1.35, 0)];
        for k in 0..9 {
            leaf.push(bx(at(0.03 + k as f32 * (w - 0.06) / 8.0), 0.0, 0.035, 0.035, 0.08, 1.5, 0));
        }
        v.extend(pivot(leaf, Vec3::new(hx, 0.0, 0.0), side * open * 1.6));
    }
    v
}

/// Serre vitrée de 2,4 × 3 m sur un socle de briques, tablettes de semis le long des côtés.
fn greenhouse() -> Vec<Piece> {
    let (hw, hd, eave, ridge) = (1.2, 1.5, 1.9, 2.5);
    let glass = |at: [f32; 3], s: [f32; 3]| Piece::new(Shape::Glass, Vec3::from(at), Vec3::from(s), 0);
    let mut v = border(2.0 * hw, 2.0 * hd, 0.1, (0.0, 0.3), 1).to_vec();
    v.extend([
        glass([0.0, 1.1, -hd + 0.05], [2.0 * hw, 1.6, 0.02]),
        glass([0.0, 1.1, hd - 0.05], [2.0 * hw, 1.6, 0.02]),
        glass([-hw + 0.05, 1.1, 0.0], [0.02, 1.6, 2.0 * hd]),
        glass([hw - 0.05, 1.1, 0.0], [0.02, 1.6, 2.0 * hd]),
        bx(0.0, 0.0, 0.06, 2.0 * hd, ridge - 0.03, ridge + 0.03, 0),
        bx(0.0, hd - 0.05, 0.85, 0.05, 1.75, 1.8, 0),
    ]);
    let s = (ridge - eave) / hw;
    let len = hw * (1.0 + s * s).sqrt();
    for side in [-1.0, 1.0] {
        let (x, roof) = (side * (hw - 0.05), Vec3::new(side * hw / 2.0, f32::midpoint(eave, ridge), 0.0));
        v.push(
            Piece::new(Shape::Glass, roof, Vec3::new(len, 0.02, 2.0 * hd), 0)
                .turn(Quat::from_rotation_z(-side * s.atan())),
        );
        v.push(bx(x, 0.0, 0.06, 2.0 * hd, eave - 0.03, eave + 0.03, 0));
        for z in [-hd + 0.05, -0.5, 0.5, hd - 0.05] {
            v.push(bx(x, z, 0.05, 0.05, 0.3, eave, 0));
            v.push(span(Shape::Cube, Vec3::new(x, eave, z), Vec3::new(0.0, ridge, z), 0.05, 0));
        }
        // montant de la porte, tablette garnie
        v.push(bx(side * 0.4, hd - 0.05, 0.05, 0.05, 0.3, eave, 0));
        v.push(bx(side * 0.75, 0.0, 0.5, 2.4, 0.75, 0.8, 2));
        for z in [-1.1, 1.1] {
            v.push(bx(side * 0.75, z, 0.04, 0.04, 0.0, 0.75, 2));
        }
        v.extend(clumps(&[-0.8, 0.0, 0.8].map(|z| ([side * 0.75, 0.92, z], [0.35, 0.28, 0.35])), 3));
    }
    v
}

fn pergola() -> Vec<Piece> {
    let mut v: Vec<Piece> =
        [(-1.4, -1.4), (1.4, -1.4), (-1.4, 1.4), (1.4, 1.4)].map(|(x, z)| bx(x, z, 0.15, 0.15, 0.0, 2.4, 0)).to_vec();
    for z in [-1.4, 1.4] {
        v.push(bx(0.0, z, 3.3, 0.08, 2.4, 2.6, 1));
    }
    for k in 0..7 {
        v.push(bx(-1.5 + k as f32 * 0.5, 0.0, 0.06, 3.4, 2.6, 2.72, 1));
    }
    v
}

/// Puits : margelle ronde, deux poteaux et un treuil le long de z, manivelle à l'avant, seau.
fn well() -> Vec<Piece> {
    let along_z = Quat::from_rotation_x(FRAC_PI_2);
    let mut v = vec![
        cyl(0.0, 0.0, 1.2, 0.0, 0.7, 0),
        cyl(0.0, 0.0, 1.3, 0.7, 0.8, 1),
        cyl(0.0, 0.0, 0.95, 0.8, 0.81, 6),
        Piece::new(Shape::Cylinder, Vec3::new(0.0, 1.5, 0.0), Vec3::new(0.12, 1.25, 0.12), 2).turn(along_z),
        bx(0.0, 0.66, 0.04, 0.04, 1.3, 1.5, 2),
        Piece::new(Shape::Cylinder, Vec3::new(0.0, 1.3, 0.72), Vec3::new(0.04, 0.12, 0.04), 2).turn(along_z),
        bx(0.0, 0.0, 0.06, 1.3, 2.24, 2.3, 2),
        cyl(0.0, 0.0, 0.015, 1.08, 1.45, 5),
        cyl(0.0, 0.0, 0.24, 0.85, 1.08, 4),
    ];
    for z in [-0.55, 0.55] {
        v.push(bx(0.0, z, 0.1, 0.1, 0.8, 2.3, 2));
    }
    v.extend(gable_roof(1.0, 1.5, (1.95, 2.3), 0.2, 3));
    v
}

/// Fontaine : bassin rond, colonne, vasque d'où l'eau retombe en gerbe, jet au centre.
fn fountain() -> Vec<Piece> {
    let mut v = vec![
        cyl(0.0, 0.0, 2.4, 0.0, 0.45, 0),
        cyl(0.0, 0.0, 2.5, 0.45, 0.52, 0),
        cyl(0.0, 0.0, 2.2, 0.52, 0.53, 1),
        cyl(0.0, 0.0, 0.3, 0.53, 1.1, 0),
        Piece::new(Shape::Cone, Vec3::new(0.0, 1.2, 0.0), Vec3::new(1.0, 0.3, 1.0), 0).turn(Quat::from_rotation_x(PI)),
        cyl(0.0, 0.0, 0.9, 1.35, 1.36, 1),
        cyl(0.0, 0.0, 0.06, 1.36, 1.85, 2),
        ball([0.0, 1.88, 0.0], [0.18, 0.12, 0.18], 2),
    ];
    for k in 0..6 {
        let (s, c) = (k as f32 * TAU / 6.0).sin_cos();
        v.push(span(
            Shape::Cylinder,
            Vec3::new(0.46 * c, 1.34, 0.46 * s),
            Vec3::new(0.75 * c, 0.53, 0.75 * s),
            0.03,
            2,
        ));
    }
    v
}

/// Mare : eau ovale de 2,6 × 1,8 m au ras du sol, cernée de pierres, nénuphars et roseaux.
fn pond() -> Vec<Piece> {
    let mut v = vec![Piece::new(Shape::Cylinder, Vec3::new(0.0, 0.02, 0.0), Vec3::new(2.6, 0.02, 1.8), 0)];
    for k in 0..14 {
        let a = k as f32 * TAU / 14.0;
        let j = ((k * 37) % 5) as f32 * 0.03;
        let h = 0.14 + j;
        v.push(ball([1.35 * a.cos(), h * 0.4, -0.95 * a.sin()], [0.38 + j, h, 0.3 + j], 1).yaw(a));
    }
    for (x, z) in [(0.3, 0.2), (0.6, -0.15), (-0.1, -0.3)] {
        v.push(cyl(x, z, 0.28, 0.03, 0.04, 2));
    }
    for (x, z, h) in [(-0.9, 0.35, 0.9), (-1.0, 0.2, 1.1), (-0.8, 0.45, 0.75), (-1.05, 0.4, 0.95), (-0.85, 0.15, 0.8)] {
        v.push(cyl(x, z, 0.025, 0.0, h, 3));
    }
    v
}

fn planter() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 1.0, 0.35, 0.0, 0.35, 0), bx(0.0, 0.0, 0.92, 0.27, 0.35, 0.36, 1)];
    for k in 0..5 {
        v.push(ball([-0.36 + k as f32 * 0.18, 0.44, 0.0], [0.24, 0.2, 0.22], 2));
    }
    for k in 0..8 {
        v.push(ball([-0.39 + k as f32 * 0.11, 0.55, [0.06, -0.06][k % 2]], [0.09, 0.07, 0.09], 3));
    }
    v
}

/// Carré potager de 1,2 m : salades devant, trois pieds de tomates tuteurés au fond.
fn vegetable_patch() -> Vec<Piece> {
    let mut v = border(1.2, 1.2, 0.05, (0.0, 0.3), 0).to_vec();
    v.push(bx(0.0, 0.0, 1.1, 1.1, 0.0, 0.25, 1));
    for x in [-0.35, 0.0, 0.35] {
        for z in [0.05, 0.35] {
            v.push(ball([x, 0.32, z], [0.26, 0.16, 0.26], 2));
        }
        v.push(cyl(x, -0.35, 0.02, 0.25, 1.2, 5));
        v.push(ball([x, 0.75, -0.33], [0.3, 0.7, 0.3], 3));
        for (dx, y) in [(0.12, 0.55), (-0.13, 0.8), (0.1, 0.95)] {
            v.push(ball([x + dx, y, -0.23], [0.08; 3], 4));
        }
    }
    v
}

/// Cinq pierres plates de 4 cm espacées de 60 cm, de l'arrière vers l'avant.
fn stepping_stones() -> Vec<Piece> {
    [(0.08, 0.46, 0.38), (-0.1, 0.5, 0.4), (0.1, 0.42, 0.36), (-0.06, 0.48, 0.4), (0.05, 0.44, 0.38)]
        .into_iter()
        .enumerate()
        .map(|(k, (x, a, b))| {
            let at = Vec3::new(x, 0.02, -1.2 + k as f32 * 0.6);
            Piece::new(Shape::Cylinder, at, Vec3::new(a, 0.04, b), 0).yaw(k as f32 * 0.7)
        })
        .collect()
}

/// Lanterne de pierre (tōrō) de 1,2 m : socle, fût, foyer éclairé, chapeau et bouton.
fn stone_lantern() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 0.0, 0.5, 0.5, 0.0, 0.1, 0),
        cyl(0.0, 0.0, 0.2, 0.1, 0.55, 0),
        bx(0.0, 0.0, 0.45, 0.45, 0.55, 0.65, 0),
        bx(0.0, 0.0, 0.26, 0.26, 0.67, 0.88, 1),
        bx(0.0, 0.0, 0.6, 0.6, 0.9, 0.95, 0),
        Piece::new(Shape::Cone, Vec3::new(0.0, 1.05, 0.0), Vec3::new(0.75, 0.2, 0.75), 0),
        ball([0.0, 1.18, 0.0], [0.12; 3], 0),
        light([0.0, 0.78, 0.0], 1),
    ];
    for (x, z) in [(-0.15, -0.15), (0.15, -0.15), (-0.15, 0.15), (0.15, 0.15)] {
        v.push(bx(x, z, 0.06, 0.06, 0.65, 0.9, 0));
    }
    v
}

/// Torii : deux piliers, deux linteaux dont le chapeau noir relevé aux bouts ; on passe dessous.
fn torii() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 0.0, 3.6, 0.22, 2.65, 2.85, 0),
        bx(0.0, 0.0, 0.25, 0.18, 2.85, 3.3, 0),
        bx(0.0, 0.0, 4.0, 0.36, 3.3, 3.5, 0),
        bx(0.0, 0.0, 3.6, 0.42, 3.5, 3.65, 1),
    ];
    for x in [-1.4, 1.4] {
        v.push(cyl(x, 0.0, 0.36, 0.0, 3.3, 0));
        v.push(cyl(x, 0.0, 0.44, 0.0, 0.35, 1));
    }
    for side in [-1.0, 1.0] {
        let end = Vec3::new(side * 2.15, 3.62, 0.0);
        v.push(Piece::new(Shape::Cube, end, Vec3::new(0.8, 0.15, 0.42), 1).turn(Quat::from_rotation_z(side * 0.15)));
    }
    v
}

/// Bûches de 50 cm couchées vers l'avant (on voit leurs bouts), tenues par quatre piquets.
fn woodpile() -> Vec<Piece> {
    let (d, along_z) = (0.16, Quat::from_rotation_x(FRAC_PI_2));
    let mut v = Vec::new();
    for (row, n) in [8, 7, 6, 5].into_iter().enumerate() {
        let y = d / 2.0 + row as f32 * d * 0.87;
        for k in 0..n {
            let x = (k as f32 - (n - 1) as f32 / 2.0) * d;
            v.push(Piece::new(Shape::Cylinder, Vec3::new(x, y, 0.0), Vec3::new(d, 0.5, d), 0).turn(along_z));
        }
    }
    for (x, z) in [(-0.68, -0.2), (0.68, -0.2), (-0.68, 0.2), (0.68, 0.2)] {
        v.push(bx(x, z, 0.05, 0.05, 0.0, 0.7, 1));
    }
    v
}

/// Brouette : roue à l'avant (+z), bac évasé, brancards et poignées à l'arrière.
fn wheelbarrow() -> Vec<Piece> {
    let mut v = vec![
        Piece::new(Shape::Wheel, Vec3::new(0.0, 0.19, 0.6), Vec3::new(0.08, 0.38, 0.38), 2),
        bx(0.0, 0.0, 0.42, 0.6, 0.42, 0.45, 0),
        Piece::new(Shape::Cube, Vec3::new(0.0, 0.56, -0.32), Vec3::new(0.5, 0.28, 0.03), 0)
            .turn(Quat::from_rotation_x(-0.2)),
        Piece::new(Shape::Cube, Vec3::new(0.0, 0.56, 0.38), Vec3::new(0.5, 0.32, 0.03), 0)
            .turn(Quat::from_rotation_x(0.6)),
    ];
    for side in [-1.0, 1.0] {
        let wall = Vec3::new(side * 0.26, 0.56, 0.02);
        v.push(Piece::new(Shape::Cube, wall, Vec3::new(0.03, 0.28, 0.72), 0).turn(Quat::from_rotation_z(-side * 0.25)));
        let (axle, back) = (Vec3::new(side * 0.06, 0.19, 0.6), Vec3::new(side * 0.27, 0.6, -0.65));
        v.push(span(Shape::Cylinder, axle, back, 0.035, 1));
        v.push(span(Shape::Cylinder, back, Vec3::new(side * 0.28, 0.62, -0.85), 0.04, 3));
        v.push(span(Shape::Cylinder, Vec3::new(side * 0.22, 0.45, -0.3), Vec3::new(side * 0.24, 0.0, -0.36), 0.03, 1));
    }
    v
}

/// Nain de jardin de 47 cm, tourné vers l'avant.
fn gnome() -> Vec<Piece> {
    let down = Quat::from_rotation_x(PI);
    vec![
        ball([-0.045, 0.025, 0.02], [0.07, 0.05, 0.1], 4),
        ball([0.045, 0.025, 0.02], [0.07, 0.05, 0.1], 4),
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.15, 0.0), Vec3::new(0.17, 0.22, 0.15), 1),
        ball([-0.075, 0.18, 0.02], [0.05, 0.09, 0.05], 1),
        ball([0.075, 0.18, 0.02], [0.05, 0.09, 0.05], 1),
        ball([0.0, 0.27, 0.0], [0.12; 3], 2),
        ball([0.0, 0.275, 0.06], [0.03; 3], 2),
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.215, 0.035), Vec3::new(0.11, 0.11, 0.06), 3).turn(down),
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.39, -0.01), Vec3::new(0.13, 0.16, 0.13), 0),
    ]
}

/// Niche de 0,8 × 1 m, entrée en arc à l'avant, gamelle devant.
fn doghouse() -> Vec<Piece> {
    let (d, ys) = (1.0, (0.6, 0.9));
    let mut v = vec![
        bx(0.0, 0.0, 0.8, d, 0.0, ys.0, 0),
        gable(0.8, -d / 2.0 + 0.02, ys, 0.04, 0),
        gable(0.8, d / 2.0 - 0.02, ys, 0.04, 0),
        bx(0.0, d / 2.0 + 0.005, 0.32, 0.01, 0.0, 0.32, 2),
        Piece::new(Shape::Wheel, Vec3::new(0.0, 0.32, d / 2.0 + 0.005), Vec3::new(0.01, 0.32, 0.32), 2).yaw(FRAC_PI_2),
        cyl(0.25, d / 2.0 + 0.12, 0.2, 0.0, 0.07, 3),
    ];
    v.extend(gable_roof(0.8, d + 0.2, ys, 0.1, 1));
    v
}

/// Nichoir sur un poteau de 1,5 m, trou d'envol et perchoir vers l'avant.
fn birdhouse() -> Vec<Piece> {
    let ys = (1.75, 1.87);
    let mut v = vec![
        bx(0.0, 0.0, 0.07, 0.07, 0.0, 1.5, 2),
        bx(0.0, 0.0, 0.2, 0.2, 1.5, ys.0, 0),
        gable(0.2, -0.09, ys, 0.02, 0),
        gable(0.2, 0.09, ys, 0.02, 0),
        Piece::new(Shape::Wheel, Vec3::new(0.0, 1.66, 0.101), Vec3::new(0.005, 0.05, 0.05), 3).yaw(FRAC_PI_2),
        Piece::new(Shape::Cylinder, Vec3::new(0.0, 1.58, 0.13), Vec3::new(0.012, 0.06, 0.012), 2)
            .turn(Quat::from_rotation_x(FRAC_PI_2)),
    ];
    v.extend(gable_roof(0.2, 0.3, ys, 0.04, 1));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::shapes::prop_shapes;

    #[test]
    fn fence_modules_are_two_meters_long() {
        for k in [PropKind::PicketFence, PropKind::LowWall, PropKind::Hedge, PropKind::Fence] {
            assert!((k.size().x - 2.0).abs() < 0.1, "{k:?} : {}", k.size());
        }
    }
    #[test]
    fn gate_opens_backwards_and_shed_door_forwards() {
        assert!(PropKind::Gate.openable() && PropKind::GardenShed.openable());
        assert!(!PropKind::Pergola.openable() && !PropKind::Well.openable());
        let open = gate(1.0);
        assert!(open.iter().any(|p| p.at.z < -1.0) && open.iter().all(|p| p.at.z < 0.05), "battants côté jardin");
        let shed = cabin((2.4, 2.0), (2.0, 2.6), (0.0, 0.9, 1.85), 1.0);
        assert!(shed.iter().filter(|p| p.slot == 2).all(|p| p.at.z > 1.2), "porte ouverte devant l'abri");
    }
    #[test]
    fn roof_panels_meet_at_the_ridge() {
        let [left, right] = gable_roof(2.0, 2.0, (2.0, 2.5), 0.2, 1);
        for (p, side) in [(left, -1.0), (right, 1.0)] {
            let half = side * (p.rot * Vec3::new(p.size.x / 2.0, 0.0, 0.0));
            let (inner, outer) = (p.at - half, p.at + half);
            assert!(inner.x.abs() < 1e-4 && (inner.y - 2.525).abs() < 1e-4, "faîte : {inner}");
            assert!((outer.x - side * 1.2).abs() < 1e-4 && outer.y < 2.0, "égout : {outer}");
        }
    }
    #[test]
    fn stepping_stones_lie_flat_and_lantern_lights() {
        assert!(PropKind::SteppingStones.flat() && !PropKind::Pond.flat());
        assert!(PropKind::SteppingStones.size().y > 2.5, "chemin de l'arrière vers l'avant");
        let lantern = prop_shapes(PropKind::StoneLantern, 0.0, 3.0);
        assert_eq!(lantern.iter().filter(|p| p.shape == Shape::Light).count(), 1);
        assert!(prop_shapes(PropKind::Greenhouse, 0.0, 3.0).iter().any(|p| p.shape == Shape::Glass));
    }
    #[test]
    fn window_lies_on_its_wall() {
        let side = window(Vec3::new(1.0, 1.0, 0.0), (0.6, 0.4), FRAC_PI_2, (0, 1));
        assert!(side.iter().all(|p| (p.at.x - 1.0).abs() < 0.02), "vitre et cadre plaqués sur le mur x = 1");
        assert!(side.iter().any(|p| (p.at.z + 0.325).abs() < 1e-3), "montant au bord de la vitre");
    }
}
