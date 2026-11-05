//! Primitives des meubles, escaliers, végétaux et de la voiture, dans le repère de l'objet.

use bevy::math::{Quat, Vec3};

use super::{BARK, BIRCH_BARK, DARK, LEAF, METAL, PropKind, Rgb, Shape, WHITE};

/// Primitive locale d'un meuble : forme, centre, taille, couleur fixe (`None` = couleur du meuble),
/// rotation autour de la verticale (radians).
pub(super) type Piece = (Shape, Vec3, Vec3, Option<Rgb>, f32);

/// Boules de feuillage (centre, taille) à la couleur/texture du végétal.
fn clumps(c: &[([f32; 3], [f32; 3])]) -> Vec<Piece> {
    c.iter().map(|&(p, s)| (Shape::Sphere, Vec3::from(p), Vec3::from(s), None, 0.0)).collect()
}
/// Boîte posée entre y0 et y1.
fn bx(x: f32, z: f32, sx: f32, sz: f32, y0: f32, y1: f32, c: Option<Rgb>) -> Piece {
    (Shape::Cube, Vec3::new(x, f32::midpoint(y0, y1), z), Vec3::new(sx, y1 - y0, sz), c, 0.0)
}

/// Porte de largeur w accrochée en (hx, hz) ; side = 1 si elle s'étend vers +x depuis la charnière.
fn door(open: f32, hx: f32, hz: f32, w: f32, y0: f32, y1: f32, side: f32) -> [Piece; 2] {
    let rot = -side * open * 1.7;
    let q = Quat::from_rotation_y(rot);
    let at = |x: f32, z: f32| Vec3::new(hx, f32::midpoint(y0, y1), hz) + q * Vec3::new(side * x, 0.0, z);
    [
        (Shape::Cube, at(w / 2.0, 0.02), Vec3::new(w - 0.01, y1 - y0, 0.03), None, rot),
        (Shape::Cube, at(w - 0.07, 0.06), Vec3::new(0.025, 0.2_f32.min(y1 - y0 - 0.1), 0.04), Some(METAL), rot),
    ]
}

/// Caisson ouvert à l'avant (pour voir l'intérieur quand les portes s'ouvrent).
fn carcass(w: f32, d: f32, h: f32, inside: Option<Rgb>) -> [Piece; 5] {
    let t = 0.025;
    [
        bx(0.0, -d / 2.0 + t / 2.0, w, t, 0.0, h, None),
        bx(-w / 2.0 + t / 2.0, 0.0, t, d, 0.0, h, None),
        bx(w / 2.0 - t / 2.0, 0.0, t, d, 0.0, h, None),
        bx(0.0, 0.0, w, d, h - t, h, None),
        bx(0.0, 0.0, w - 2.0 * t, d - t, 0.0, 0.08, inside),
    ]
}

fn legs(hx: f32, hz: f32, h: f32) -> [Piece; 4] {
    [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sz)| bx(sx * hx, sz * hz, 0.05, 0.05, 0.0, h, None))
}

/// Primitives d'un objet, la face avant vers +z ; `open` (0..1) ouvre ses portes.
pub(super) fn prop_shapes(kind: PropKind, open: f32, height: f32) -> Vec<Piece> {
    use PropKind::{
        AppleTree, Birch, Bush, Car, Cypress, Fence, Flowers, Hedge, LStairs, Palm, Pillar, Pine, Rosebush,
        SpiralStairs, Stairs, Sunflower, Tree,
    };
    match kind {
        Stairs | SpiralStairs | LStairs | Pillar => structure_shapes(kind, height),
        Car => car_shapes(),
        PropKind::StreetLamp | PropKind::Shelf => fixture_shapes(kind),
        Tree | Pine | Birch | Palm | Cypress | AppleTree => tree_shapes(kind),
        Bush | Hedge | Fence | Flowers | Rosebush | Sunflower => garden_shapes(kind),
        _ => furniture_shapes(kind, open),
    }
}

/// Lampadaire de rue et étagère : mobilier sans porte.
fn fixture_shapes(kind: PropKind) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::StreetLamp => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.15, 0.0), Vec3::new(0.3, 0.3, 0.3), None, 0.0));
            v.push((Shape::Cylinder, Vec3::new(0.0, 2.3, 0.0), Vec3::new(0.12, 4.4, 0.12), None, 0.0));
            v.push(bx(0.0, 0.35, 0.08, 0.8, 4.4, 4.48, None));
            v.push(bx(0.0, 0.75, 0.35, 0.45, 4.25, 4.45, None));
            v.push(bx(0.0, 0.75, 0.3, 0.4, 4.2, 4.25, Some([1.0, 0.95, 0.8])));
            v.push((Shape::Light, Vec3::new(0.0, 4.0, 0.75), Vec3::ONE, None, 0.0));
        }
        PropKind::Shelf => {
            for x in [-0.49, 0.49] {
                v.push(bx(x, 0.0, 0.02, 0.4, 0.0, 1.8, None));
            }
            v.push(bx(0.0, -0.19, 1.0, 0.02, 0.0, 1.8, None));
            for k in 0..5 {
                let y = 0.08 + k as f32 * 0.42;
                v.push(bx(0.0, 0.0, 0.96, 0.38, y, y + 0.025, None));
            }
            // marchandises colorées sur les tablettes
            for (k, c) in [[0.8, 0.2, 0.2], [0.2, 0.5, 0.8], [0.9, 0.7, 0.1], [0.3, 0.7, 0.3]].into_iter().enumerate() {
                let y = 0.105 + k as f32 * 0.42;
                v.push(bx(-0.2 + (k % 2) as f32 * 0.4, 0.02, 0.4, 0.28, y, y + 0.25, Some(c)));
            }
        }
        _ => {}
    }
    v
}

fn furniture_shapes(kind: PropKind, open: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Table => {
            v.push(bx(0.0, 0.0, 1.6, 0.9, 0.72, 0.77, None));
            v.extend(legs(0.72, 0.38, 0.72));
        }
        PropKind::Chair => {
            v.push(bx(0.0, 0.0, 0.45, 0.45, 0.43, 0.48, None));
            v.push(bx(0.0, -0.2, 0.45, 0.05, 0.48, 0.95, None));
            v.extend(legs(0.19, 0.19, 0.43));
        }
        PropKind::Sofa => {
            v.push(bx(0.0, 0.0, 2.0, 0.9, 0.1, 0.45, None));
            v.push(bx(0.0, -0.35, 2.0, 0.2, 0.45, 0.85, None));
            v.push(bx(-0.9, 0.0, 0.2, 0.9, 0.45, 0.65, None));
            v.push(bx(0.9, 0.0, 0.2, 0.9, 0.45, 0.65, None));
            v.extend(legs(0.9, 0.38, 0.1).map(|(s, c, z, _, r)| (s, c, z, Some(DARK), r)));
        }
        PropKind::Bed => {
            v.push(bx(0.0, 0.0, 1.6, 2.0, 0.1, 0.35, None));
            v.push(bx(0.0, 0.05, 1.5, 1.85, 0.35, 0.55, Some(WHITE)));
            v.push(bx(0.0, -0.98, 1.6, 0.06, 0.1, 1.0, None));
            v.push(bx(-0.38, -0.75, 0.6, 0.3, 0.55, 0.65, Some(WHITE)));
            v.push(bx(0.38, -0.75, 0.6, 0.3, 0.55, 0.65, Some(WHITE)));
        }
        PropKind::Wardrobe => {
            v.extend(carcass(1.2, 0.6, 2.0, None));
            v.push(bx(0.0, 0.0, 1.15, 0.55, 1.5, 1.53, None));
            v.push(bx(0.0, -0.05, 1.15, 0.03, 1.38, 1.4, Some(METAL)));
            // linge plié et vêtements suspendus
            v.push(bx(-0.3, 0.0, 0.35, 0.35, 1.53, 1.68, Some([0.7, 0.3, 0.3])));
            v.push(bx(0.25, 0.0, 0.35, 0.35, 1.53, 1.65, Some([0.3, 0.4, 0.7])));
            for (i, c) in
                [[0.2, 0.25, 0.4], [0.85, 0.85, 0.8], [0.4, 0.5, 0.3], [0.6, 0.2, 0.25]].into_iter().enumerate()
            {
                v.push(bx(-0.35 + i as f32 * 0.22, -0.05, 0.05, 0.45, 0.6, 1.38, Some(c)));
            }
            v.extend(door(open, -0.6, 0.3, 0.6, 0.02, 1.98, 1.0));
            v.extend(door(open, 0.6, 0.3, 0.6, 0.02, 1.98, -1.0));
        }
        PropKind::Kitchen => {
            v.extend(carcass(2.4, 0.6, 0.86, None));
            v.push(bx(0.0, 0.0, 2.35, 0.55, 0.42, 0.44, None));
            v.push(bx(0.0, 0.0, 2.42, 0.62, 0.86, 0.9, Some(DARK)));
            v.push(bx(0.5, 0.0, 0.6, 0.4, 0.9, 0.91, Some(METAL)));
            v.push(bx(-0.6, 0.0, 0.6, 0.5, 0.9, 0.91, Some([0.05, 0.05, 0.05])));
            // vaisselle
            v.push(bx(-0.8, 0.0, 0.3, 0.3, 0.44, 0.56, Some(WHITE)));
            v.push(bx(0.3, -0.05, 0.4, 0.25, 0.08, 0.3, Some([0.6, 0.3, 0.2])));
            for (hx, side) in [(-1.2, 1.0), (0.0, -1.0), (0.0, 1.0), (1.2, -1.0)] {
                v.extend(door(open, hx, 0.3, 0.6, 0.06, 0.84, side));
            }
        }
        PropKind::Fridge => {
            v.extend(carcass(0.7, 0.7, 1.85, Some(WHITE)));
            for y in [0.5, 0.9, 1.3] {
                v.push(bx(0.0, 0.0, 0.64, 0.6, y, y + 0.02, Some([0.8, 0.9, 0.95])));
            }
            v.push(bx(-0.15, 0.0, 0.15, 0.15, 0.92, 1.1, Some([0.9, 0.5, 0.1])));
            v.push(bx(0.12, 0.05, 0.25, 0.12, 1.32, 1.4, Some([0.95, 0.9, 0.6])));
            v.push(bx(0.0, 0.0, 0.2, 0.15, 0.52, 0.75, Some([0.3, 0.6, 0.3])));
            v.extend(door(open, -0.35, 0.35, 0.7, 0.02, 1.84, 1.0));
        }
        PropKind::Bathtub => {
            v.push(bx(0.0, 0.0, 0.75, 1.7, 0.0, 0.55, None));
            v.push(bx(0.0, 0.0, 0.6, 1.5, 0.5, 0.56, Some([0.55, 0.75, 0.9])));
        }
        PropKind::Toilet => {
            v.push(bx(0.0, 0.05, 0.4, 0.5, 0.0, 0.42, None));
            v.push(bx(0.0, -0.22, 0.4, 0.18, 0.42, 0.8, None));
        }
        PropKind::Lamp => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.3, 0.04, 0.3), Some(DARK), 0.0));
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.04, 1.6, 0.04), Some(DARK), 0.0));
            v.push((Shape::Sphere, Vec3::new(0.0, 1.7, 0.0), Vec3::new(0.4, 0.3, 0.4), None, 0.0));
            v.push((Shape::Light, Vec3::new(0.0, 1.5, 0.0), Vec3::ONE, None, 0.0));
        }
        PropKind::Plant => {
            v.push((
                Shape::Cylinder,
                Vec3::new(0.0, 0.2, 0.0),
                Vec3::new(0.35, 0.4, 0.35),
                Some([0.7, 0.35, 0.2]),
                0.0,
            ));
            v.extend(clumps(&[
                ([0.0, 0.62, 0.0], [0.5, 0.45, 0.5]),
                ([0.08, 0.8, 0.05], [0.35, 0.35, 0.35]),
                ([-0.1, 0.72, -0.06], [0.32, 0.3, 0.32]),
            ]));
        }
        // les autres sortes passent par les autres familles (voir prop_shapes)
        _ => {}
    }
    v
}

/// Escaliers (marches d'environ 20 cm) et piliers, à la hauteur demandée.
fn structure_shapes(kind: PropKind, height: f32) -> Vec<Piece> {
    let n = ((height / 0.2).round() as usize).max(1);
    let rise = height / n as f32;
    let mut v = Vec::new();
    match kind {
        PropKind::Stairs => {
            let run = 0.28;
            for i in 0..n {
                let z = -(n as f32) * run / 2.0 + (i as f32 + 0.5) * run;
                v.push(bx(0.0, z, 1.0, run, 0.0, (i + 1) as f32 * rise, None));
            }
        }
        PropKind::SpiralStairs => {
            v.push((
                Shape::Cylinder,
                Vec3::new(0.0, f32::midpoint(height, 1.0), 0.0),
                Vec3::new(0.16, height + 1.0, 0.16),
                None,
                0.0,
            ));
            for i in 0..n {
                let a = i as f32 * 0.5;
                let top = (i + 1) as f32 * rise;
                let dir = Vec3::new(a.cos(), 0.0, -a.sin());
                v.push((Shape::Cube, dir * 0.55 + Vec3::Y * (top - 0.03), Vec3::new(0.9, 0.06, 0.42), None, a));
            }
        }
        PropKind::LStairs => {
            // première volée vers le palier (centré sur l'origine), seconde volée vers +x
            let run = 0.28;
            let n1 = n / 2;
            for i in 0..n1 {
                v.push(bx(0.0, -0.5 - (n1 - i) as f32 * run + run / 2.0, 1.0, run, 0.0, (i + 1) as f32 * rise, None));
            }
            v.push(bx(0.0, 0.0, 1.0, 1.0, 0.0, (n1 + 1) as f32 * rise, None));
            for j in 0..n.saturating_sub(n1 + 1) {
                v.push(bx(0.5 + (j as f32 + 0.5) * run, 0.0, run, 1.0, 0.0, (n1 + 2 + j) as f32 * rise, None));
            }
        }
        PropKind::Pillar => v.push(bx(0.0, 0.0, 0.3, 0.3, 0.0, height, None)),
        _ => {}
    }
    v
}

fn tree_shapes(kind: PropKind) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Tree => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 1.3, 0.0), Vec3::new(0.32, 2.6, 0.32), Some(BARK), 0.0));
            // couronne en grappes de boules de tailles variées, pour une silhouette irrégulière
            v.extend(clumps(&[
                ([0.0, 3.3, 0.0], [2.6, 2.3, 2.6]),
                ([0.9, 2.9, 0.3], [1.8, 1.6, 1.8]),
                ([-0.8, 3.0, -0.4], [1.9, 1.7, 1.9]),
                ([0.2, 4.1, -0.5], [1.7, 1.5, 1.7]),
                ([-0.3, 4.2, 0.6], [1.6, 1.4, 1.6]),
                ([0.5, 2.6, -0.9], [1.4, 1.2, 1.4]),
                ([-0.6, 2.7, 0.9], [1.3, 1.2, 1.3]),
                ([0.0, 4.8, 0.0], [1.1, 1.0, 1.1]),
            ]));
        }
        PropKind::Pine => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.7, 0.0), Vec3::new(0.3, 1.4, 0.3), Some(BARK), 0.0));
            for (y, r, h) in [(1.7, 2.8, 1.9), (2.7, 2.2, 1.8), (3.6, 1.6, 1.6), (4.4, 1.0, 1.3), (5.0, 0.5, 0.9)] {
                v.push((Shape::Cone, Vec3::new(0.0, y, 0.0), Vec3::new(r, h, r), None, 0.0));
            }
        }
        PropKind::Birch => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.22, 4.0, 0.22), Some(BIRCH_BARK), 0.0));
            v.extend(clumps(&[
                ([0.0, 4.2, 0.0], [1.6, 2.6, 1.6]),
                ([0.45, 3.6, 0.2], [1.2, 1.8, 1.2]),
                ([-0.4, 3.8, -0.25], [1.2, 2.0, 1.2]),
                ([0.1, 5.3, 0.1], [1.0, 1.6, 1.0]),
            ]));
        }
        PropKind::Palm => {
            // tronc en tronçons, légèrement penché, et palmes en étoile
            for i in 0..6 {
                let k = i as f32;
                v.push((
                    Shape::Cylinder,
                    Vec3::new(k * 0.06, 0.4 + k * 0.8, 0.0),
                    Vec3::new(0.34 - k * 0.02, 0.85, 0.34 - k * 0.02),
                    Some(BARK),
                    0.0,
                ));
            }
            let top = Vec3::new(0.36, 4.9, 0.0);
            for k in 0..7 {
                let a = k as f32 * std::f32::consts::TAU / 7.0;
                v.push((
                    Shape::Sphere,
                    top + Vec3::new(a.cos(), -0.15, -a.sin()) * 1.1,
                    Vec3::new(2.4, 0.12, 0.55),
                    None,
                    a,
                ));
            }
            v.push((Shape::Sphere, top, Vec3::splat(0.5), Some([0.45, 0.3, 0.15]), 0.0));
        }
        PropKind::Cypress => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.25, 0.6, 0.25), Some(BARK), 0.0));
            v.extend(clumps(&[
                ([0.0, 3.2, 0.0], [1.3, 5.6, 1.3]),
                ([0.0, 5.7, 0.0], [0.7, 1.6, 0.7]),
                ([0.1, 1.8, 0.1], [1.1, 2.6, 1.1]),
            ]));
        }
        PropKind::AppleTree => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.26, 1.8, 0.26), Some(BARK), 0.0));
            v.extend(clumps(&[
                ([0.0, 2.6, 0.0], [2.6, 1.9, 2.6]),
                ([0.8, 2.3, 0.3], [1.5, 1.3, 1.5]),
                ([-0.7, 2.4, -0.4], [1.6, 1.4, 1.6]),
                ([0.1, 3.2, -0.2], [1.6, 1.2, 1.6]),
            ]));
            // pommes à moitié dans la couronne
            for k in 0..10 {
                let (a, e) = (k as f32 * 2.4, [-0.4_f32, 0.1, 0.5][k % 3]);
                let p = Vec3::new(1.3 * e.cos() * a.cos(), 2.6 + 0.95 * e.sin(), -1.3 * e.cos() * a.sin());
                v.push((Shape::Sphere, p, Vec3::splat(0.16), Some([0.8, 0.1, 0.08]), 0.0));
            }
        }
        _ => {}
    }
    v
}

fn garden_shapes(kind: PropKind) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Bush => v.extend(clumps(&[
            ([0.0, 0.45, 0.0], [1.1, 0.9, 1.1]),
            ([0.4, 0.4, 0.3], [0.8, 0.75, 0.8]),
            ([-0.35, 0.38, -0.25], [0.85, 0.7, 0.8]),
            ([-0.2, 0.42, 0.4], [0.7, 0.65, 0.7]),
            ([0.1, 0.72, -0.1], [0.7, 0.6, 0.7]),
        ])),
        PropKind::Hedge => {
            v.push(bx(0.0, 0.0, 2.0, 0.7, 0.0, 1.2, None));
            v.extend(clumps(&[
                ([-0.6, 1.2, 0.0], [0.9, 0.45, 0.75]),
                ([0.0, 1.25, 0.0], [0.95, 0.45, 0.75]),
                ([0.6, 1.2, 0.0], [0.9, 0.45, 0.75]),
            ]));
        }
        PropKind::Fence => {
            for x in [-1.0, 0.0, 1.0] {
                v.push(bx(x, 0.0, 0.08, 0.08, 0.0, 1.0, None));
            }
            v.push(bx(0.0, 0.0, 2.0, 0.04, 0.35, 0.45, None));
            v.push(bx(0.0, 0.0, 2.0, 0.04, 0.75, 0.85, None));
        }
        PropKind::Flowers => {
            v.push(bx(0.0, 0.0, 1.3, 0.9, 0.0, 0.1, Some([0.35, 0.24, 0.15])));
            for k in 0..15 {
                let jitter = ((k * 37) % 7) as f32 * 0.02;
                let (x, z) = ((k % 5) as f32 * 0.24 - 0.48 + jitter, (k / 5) as f32 * 0.26 - 0.26 - jitter);
                let h = 0.35 + jitter * 2.0;
                v.push((Shape::Cylinder, Vec3::new(x, h / 2.0, z), Vec3::new(0.03, h, 0.03), Some(LEAF), 0.0));
                let head = flower_head(k);
                v.push((Shape::Sphere, Vec3::new(x, h, z), Vec3::new(0.14, 0.08, 0.14), head, 0.0));
            }
            v.push((Shape::Sphere, Vec3::new(0.0, 0.12, 0.0), Vec3::new(1.2, 0.25, 0.8), Some(LEAF), 0.0));
        }
        PropKind::Rosebush => {
            for (p, s) in [
                ([0.0, 0.45, 0.0], [1.0, 0.9, 1.0]),
                ([0.3, 0.6, 0.2], [0.7, 0.7, 0.7]),
                ([-0.3, 0.55, -0.2], [0.7, 0.7, 0.7]),
            ] {
                v.push((Shape::Sphere, Vec3::from(p), Vec3::from(s), Some(LEAF), 0.0));
            }
            for k in 0..12 {
                let (a, e) = (k as f32 * 2.4, [-0.2_f32, 0.3, 0.7][k % 3]);
                let p = Vec3::new(0.5 * e.cos() * a.cos(), 0.45 + 0.45 * e.sin(), -0.5 * e.cos() * a.sin());
                v.push((Shape::Sphere, p, Vec3::splat(0.15), None, 0.0));
            }
        }
        PropKind::Sunflower => {
            for (x, z, h) in [(0.0, 0.0, 1.7), (0.35, 0.2, 1.4), (-0.3, 0.25, 1.55), (0.1, -0.3, 1.25)] {
                v.push((Shape::Cylinder, Vec3::new(x, h / 2.0, z), Vec3::new(0.04, h, 0.04), Some(LEAF), 0.0));
                v.push((Shape::Sphere, Vec3::new(x, h * 0.55, z + 0.1), Vec3::new(0.28, 0.04, 0.16), Some(LEAF), 0.0));
                v.push((Shape::Cylinder, Vec3::new(x, h, z), Vec3::new(0.45, 0.04, 0.45), None, 0.0));
                v.push((
                    Shape::Cylinder,
                    Vec3::new(x, h + 0.02, z),
                    Vec3::new(0.22, 0.04, 0.22),
                    Some([0.35, 0.2, 0.08]),
                    0.0,
                ));
            }
        }
        _ => {}
    }
    v
}

/// Couleur du cœur de la k-ième fleur d'un massif (`None` : celle du massif).
fn flower_head(k: usize) -> Option<Rgb> {
    match k % 4 {
        3 => Some([0.98, 0.85, 0.2]),
        1 => Some([0.97, 0.97, 0.95]),
        _ => None,
    }
}

/// Berline de 4,2 m, l'avant vers +z ; habitacle vitré pour conduire en vue intérieure.
fn car_shapes() -> Vec<Piece> {
    let mut v = Vec::new();
    // berline de 4,2 m, l'avant vers +z ; habitacle vitré pour conduire en vue intérieure
    let glass =
        |x: f32, z: f32, sx: f32, sz: f32| (Shape::Glass, Vec3::new(x, 1.2, z), Vec3::new(sx, 0.5, sz), None, 0.0);
    v.push(bx(0.0, 0.0, 1.8, 4.2, 0.3, 0.95, None));
    v.push(bx(0.0, -0.15, 1.72, 2.1, 1.45, 1.52, None));
    for (x, z) in [(-0.84, 0.88), (0.84, 0.88), (-0.84, -1.18), (0.84, -1.18), (-0.84, -0.15), (0.84, -0.15)] {
        v.push(bx(x, z, 0.06, 0.08, 0.95, 1.45, None));
    }
    v.push(glass(0.0, 0.88, 1.62, 0.03));
    v.push(glass(0.0, -1.18, 1.62, 0.03));
    v.push(glass(-0.85, -0.15, 0.03, 2.0));
    v.push(glass(0.85, -0.15, 0.03, 2.0));
    // intérieur : planche de bord, volant (place conducteur à gauche, x > 0), sièges
    v.push(bx(0.0, 0.72, 1.62, 0.3, 0.95, 1.05, Some(DARK)));
    v.push(bx(0.4, 0.5, 0.36, 0.05, 1.0, 1.28, Some(DARK)));
    for x in [-0.4, 0.4] {
        v.push(bx(x, -0.45, 0.5, 0.12, 0.95, 1.35, Some([0.25, 0.25, 0.28])));
    }
    v.push(bx(0.0, -0.95, 1.5, 0.12, 0.95, 1.3, Some([0.25, 0.25, 0.28])));
    // phares, feux, pare-chocs, plaques
    for x in [-0.6, 0.6] {
        v.push(bx(x, 2.1, 0.35, 0.03, 0.62, 0.76, Some([1.0, 0.97, 0.85])));
        v.push(bx(x, -2.1, 0.35, 0.03, 0.66, 0.78, Some([0.8, 0.05, 0.05])));
    }
    v.push(bx(0.0, 2.13, 1.82, 0.08, 0.3, 0.46, Some(DARK)));
    v.push(bx(0.0, -2.13, 1.82, 0.08, 0.3, 0.46, Some(DARK)));
    v.push(bx(0.0, 2.15, 0.5, 0.02, 0.47, 0.58, Some(WHITE)));
    v.push(bx(0.0, -2.15, 0.5, 0.02, 0.5, 0.61, Some(WHITE)));
    for (x, z) in [(-0.82, 1.35), (0.82, 1.35), (-0.82, -1.35), (0.82, -1.35)] {
        v.push((Shape::Wheel, Vec3::new(x, 0.34, z), Vec3::new(0.24, 0.68, 0.68), Some([0.08, 0.08, 0.09]), 0.0));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_shapes_and_doors_swing() {
        for k in PropKind::ALL {
            assert!(!prop_shapes(k, 0.0, 3.0).is_empty(), "{k:?}");
        }
        let closed = prop_shapes(PropKind::Wardrobe, 0.0, 3.0);
        let open = prop_shapes(PropKind::Wardrobe, 1.0, 3.0);
        assert!(closed.iter().zip(&open).any(|(a, b)| (a.4 - b.4).abs() > 1e-3), "les portes pivotent");
        assert_eq!([flower_head(0), flower_head(1), flower_head(3)].map(|h| h.is_some()), [false, true, true]);
    }
}
