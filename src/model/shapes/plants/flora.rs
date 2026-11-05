//! Plantes et fleurs. Les feuillages sont blancs : leur texture donne la couleur.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::{LEAVES, LIGHT_LEAVES, TRUNK, gnarled, limbs};
use crate::model::shapes::{Piece, ball, bx, clumps, cyl, span};
use crate::model::{BARK, LEAF, PropKind, Shape, Slot};

const SOIL: Slot = Slot::plain("Terre", [0.35, 0.24, 0.15]);
const TERRACOTTA: Slot = Slot::plain("Pot", [0.7, 0.35, 0.2]);

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Plant => const { &[LIGHT_LEAVES, TERRACOTTA] },
        PropKind::Bush => const { &[LIGHT_LEAVES] },
        PropKind::Flowers => {
            const {
                &[
                    Slot::plain("Fleurs", [0.85, 0.15, 0.25]),
                    Slot::plain("Fleurs blanches", [0.97, 0.97, 0.95]),
                    Slot::plain("Fleurs jaunes", [0.98, 0.85, 0.2]),
                    Slot::plain("Tiges et feuilles", LEAF),
                    SOIL,
                ]
            }
        }
        PropKind::Rosebush => const { &[Slot::plain("Roses", [0.95, 0.45, 0.6]), Slot::plain("Feuillage", LEAF)] },
        PropKind::Sunflower => {
            const {
                &[
                    Slot::plain("Pétales", [0.98, 0.8, 0.1]),
                    Slot::plain("Tiges et feuilles", LEAF),
                    Slot::plain("Cœurs", [0.35, 0.2, 0.08]),
                ]
            }
        }
        PropKind::Cactus => {
            const { &[Slot::plain("Cactus", [0.3, 0.5, 0.28]), Slot::plain("Fleurs", [0.95, 0.4, 0.55])] }
        }
        PropKind::Bamboo => const { &[Slot::plain("Cannes", [0.62, 0.68, 0.3]), LIGHT_LEAVES] },
        PropKind::Fern => const { &[LEAVES] },
        PropKind::Hydrangea => const { &[Slot::plain("Fleurs", [0.55, 0.62, 0.92]), LEAVES] },
        PropKind::TallGrass => {
            const { &[Slot::plain("Herbes", [0.5, 0.62, 0.3]), Slot::plain("Épis", [0.88, 0.82, 0.64])] }
        }
        PropKind::Reeds => {
            const { &[Slot::plain("Tiges", [0.45, 0.58, 0.28]), Slot::plain("Épis", [0.38, 0.24, 0.12])] }
        }
        PropKind::Lavender => {
            const { &[Slot::plain("Fleurs", [0.58, 0.45, 0.8]), Slot::plain("Feuillage", [0.42, 0.52, 0.4])] }
        }
        PropKind::Tulips => {
            const {
                &[
                    Slot::plain("Fleurs", [0.88, 0.1, 0.15]),
                    Slot::plain("Fleurs jaunes", [0.98, 0.85, 0.2]),
                    Slot::plain("Tiges et feuilles", LEAF),
                    SOIL,
                ]
            }
        }
        PropKind::Rock => const { &[Slot::new("Pierre", [0.8, 0.78, 0.74], "Enduit gris")] },
        PropKind::Stump => const { &[Slot::new("Écorce", BARK, "Écorce"), Slot::plain("Coupe", [0.8, 0.65, 0.45])] },
        PropKind::Mushrooms => {
            const {
                &[
                    Slot::plain("Chapeaux", [0.8, 0.12, 0.08]),
                    Slot::plain("Pieds", [0.95, 0.93, 0.86]),
                    Slot::plain("Points", [0.98, 0.97, 0.95]),
                ]
            }
        }
        PropKind::Topiary => const { &[LEAVES, Slot::new("Tige", BARK, "Écorce"), TERRACOTTA, SOIL] },
        PropKind::Ficus => const { &[LEAVES, TRUNK, Slot::plain("Pot", [0.92, 0.91, 0.88]), SOIL] },
        PropKind::Bonsai => {
            const { &[LEAVES, TRUNK, Slot::plain("Pot", [0.3, 0.22, 0.25]), Slot::plain("Mousse", [0.32, 0.45, 0.2])] }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind) -> Vec<Piece> {
    match kind {
        PropKind::Plant => {
            let mut v = vec![cyl(0.0, 0.0, 0.35, 0.0, 0.4, 1)];
            v.extend(clumps(
                &[
                    ([0.0, 0.62, 0.0], [0.5, 0.45, 0.5]),
                    ([0.08, 0.8, 0.05], [0.35, 0.35, 0.35]),
                    ([-0.1, 0.72, -0.06], [0.32, 0.3, 0.32]),
                ],
                0,
            ));
            v
        }
        PropKind::Bush => clumps(
            &[
                ([0.0, 0.45, 0.0], [1.1, 0.9, 1.1]),
                ([0.4, 0.4, 0.3], [0.8, 0.75, 0.8]),
                ([-0.35, 0.38, -0.25], [0.85, 0.7, 0.8]),
                ([-0.2, 0.42, 0.4], [0.7, 0.65, 0.7]),
                ([0.1, 0.72, -0.1], [0.7, 0.6, 0.7]),
            ],
            0,
        ),
        PropKind::Flowers => flowers(),
        PropKind::Rosebush => rosebush(),
        PropKind::Sunflower => sunflower(),
        PropKind::Cactus => cactus(),
        PropKind::Bamboo => bamboo(),
        PropKind::Fern => fern(),
        PropKind::TallGrass => tall_grass(),
        PropKind::Reeds => reeds(),
        PropKind::Lavender => lavender(),
        PropKind::Tulips => tulips(),
        PropKind::Hydrangea => hydrangea(),
        PropKind::Rock => rock(),
        PropKind::Stump => stump(),
        PropKind::Mushrooms => mushrooms(),
        PropKind::Topiary => topiary(),
        PropKind::Ficus => ficus(),
        PropKind::Bonsai => bonsai(),
        _ => Vec::new(),
    }
}

fn flowers() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 1.3, 0.9, 0.0, 0.1, 4)];
    for k in 0..15 {
        let jitter = ((k * 37) % 7) as f32 * 0.02;
        let (x, z) = ((k % 5) as f32 * 0.24 - 0.48 + jitter, (k / 5) as f32 * 0.26 - 0.26 - jitter);
        let h = 0.35 + jitter * 2.0;
        v.push(cyl(x, z, 0.03, 0.0, h, 3));
        v.push(ball([x, h, z], [0.14, 0.08, 0.14], flower_head(k)));
    }
    v.push(ball([0.0, 0.12, 0.0], [1.2, 0.25, 0.8], 3));
    v
}

/// Partie du cœur de la k-ième fleur d'un massif : rouge, blanche ou jaune.
fn flower_head(k: usize) -> usize {
    match k % 4 {
        3 => 2,
        1 => 1,
        _ => 0,
    }
}

fn rosebush() -> Vec<Piece> {
    let mut v = clumps(
        &[
            ([0.0, 0.45, 0.0], [1.0, 0.9, 1.0]),
            ([0.3, 0.6, 0.2], [0.7, 0.7, 0.7]),
            ([-0.3, 0.55, -0.2], [0.7, 0.7, 0.7]),
        ],
        1,
    );
    for k in 0..12 {
        let (a, e) = (k as f32 * 2.4, [-0.2_f32, 0.3, 0.7][k % 3]);
        v.push(ball([0.5 * e.cos() * a.cos(), 0.45 + 0.45 * e.sin(), -0.5 * e.cos() * a.sin()], [0.15; 3], 0));
    }
    v
}

fn sunflower() -> Vec<Piece> {
    let mut v = Vec::new();
    for (x, z, h) in [(0.0, 0.0, 1.7), (0.35, 0.2, 1.4), (-0.3, 0.25, 1.55), (0.1, -0.3, 1.25)] {
        v.push(cyl(x, z, 0.04, 0.0, h, 1));
        v.push(ball([x, h * 0.55, z + 0.1], [0.28, 0.04, 0.16], 1));
        v.push(cyl(x, z, 0.45, h - 0.02, h + 0.02, 0));
        v.push(cyl(x, z, 0.22, h, h + 0.04, 2));
    }
    v
}

/// Brin, feuille ou épi : forme `shape` de section `d` posée en `base` et tendue sur `len` mètres
/// selon `dir` (unitaire).
fn blade(shape: Shape, base: Vec3, dir: Vec3, len: f32, d: f32, slot: usize) -> Piece {
    span(shape, base, base + dir * len, d, slot)
}

/// Direction unitaire d'azimut `a` (depuis +z vers +x) et d'élévation `e` au-dessus de l'horizontale.
fn heading(a: f32, e: f32) -> Vec3 {
    Vec3::new(e.cos() * a.sin(), e.sin(), e.cos() * a.cos())
}

fn cactus() -> Vec<Piece> {
    // cierge à deux bras : colonne arrondie, bras coudés qui remontent, fleurs au sommet
    let mut v = vec![cyl(0.0, 0.0, 0.36, 0.0, 1.9, 0), ball([0.0, 1.9, 0.0], [0.36; 3], 0)];
    for (x, y, top) in [(0.42, 0.7, 1.35), (-0.4, 1.0, 1.55)] {
        let elbow = Vec3::new(x, y + 0.12, 0.0);
        v.push(span(Shape::Cylinder, Vec3::new(0.0, y, 0.0), elbow, 0.22, 0));
        v.push(ball(elbow.to_array(), [0.22; 3], 0));
        v.push(cyl(x, 0.0, 0.22, y + 0.12, top, 0));
        v.push(ball([x, top, 0.0], [0.22; 3], 0));
        v.push(ball([x, top + 0.1, 0.0], [0.1, 0.07, 0.1], 1));
    }
    v.push(ball([0.0, 2.06, 0.0], [0.14, 0.09, 0.14], 1));
    v
}

fn bamboo() -> Vec<Piece> {
    // touffe de cannes qui s'écartent en montant, trois nœuds chacune, rameaux feuillus en haut
    let mut v = Vec::new();
    for k in 0..7 {
        let a = k as f32 * TAU / 7.0;
        let r = 0.08 + 0.04 * (k % 3) as f32;
        let base = Vec3::new(r * a.sin(), 0.0, r * a.cos());
        let top = Vec3::new(5.0 * base.x, 3.6 + 0.35 * ((k * 2) % 3) as f32, 5.0 * base.z);
        v.push(span(Shape::Cylinder, base, top, 0.06, 0));
        for t in [0.25, 0.5, 0.75] {
            let p = base.lerp(top, t);
            v.push(span(Shape::Cylinder, p, p + (top - base) * 0.012, 0.075, 0));
        }
        // rameaux feuillus qui partent de côté en retombant, alternés de part et d'autre
        for (i, t) in [0.55, 0.7, 0.85, 1.0].into_iter().enumerate() {
            let side = if i % 2 == 0 { 1.1 } else { -1.1 };
            v.push(blade(Shape::Sphere, base.lerp(top, t), heading(a + side, -0.35), 0.7, 0.22, 1));
        }
    }
    v
}

fn fern() -> Vec<Piece> {
    // rosette de frondes : ellipsoïdes plats qui partent du cœur, les extérieures plus couchées
    let mut v = Vec::new();
    for (n, len, tilt, a0) in [(9, 0.75, 0.45, 0.0), (5, 0.6, 0.95, 0.3)] {
        for k in 0..n {
            let a = a0 + k as f32 * TAU / n as f32;
            let at = Vec3::new(0.0, 0.02, 0.0) + heading(a, tilt) * (len / 2.0);
            let q = Quat::from_rotation_y(a) * Quat::from_rotation_x(-tilt);
            v.push(Piece::new(Shape::Sphere, at, Vec3::new(0.2, 0.03, len), 0).turn(q));
        }
    }
    v
}

fn tall_grass() -> Vec<Piece> {
    // touffe en fontaine : brins fins penchés vers l'extérieur, épis clairs sur de plus hautes tiges
    let mut v = Vec::new();
    for k in 0..22 {
        let a = k as f32 * 2.4;
        let r = 0.05 * (k % 3) as f32;
        let base = Vec3::new(r * a.sin(), 0.0, r * a.cos());
        let dir = heading(a, 1.4 - 0.1 * (k % 4) as f32);
        let len = 0.9 + 0.1 * ((k * 3) % 4) as f32;
        v.push(blade(Shape::Cone, base, dir, len, 0.04, 0));
        if k % 3 == 0 {
            v.push(blade(Shape::Cylinder, base, dir, len + 0.1, 0.012, 0));
            v.push(blade(Shape::Sphere, base + dir * (len + 0.05), dir, 0.4, 0.08, 1));
        }
    }
    v
}

fn reeds() -> Vec<Piece> {
    // massettes : hautes tiges presque droites, épis bruns en cigare, feuilles en lame
    let mut v = Vec::new();
    for k in 0..12 {
        let a = k as f32 * 2.4;
        let r = 0.08 + 0.05 * (k % 4) as f32;
        let base = Vec3::new(r * a.sin(), 0.0, r * a.cos());
        let dir = heading(a, 1.47);
        let len = 1.6 + 0.12 * ((k * 5) % 4) as f32;
        v.push(blade(Shape::Cylinder, base, dir, len, 0.025, 0));
        if k % 2 == 0 {
            v.push(blade(Shape::Sphere, base + dir * (len * 0.74), dir, 0.28, 0.065, 1));
        }
    }
    for k in 0..8 {
        let a = k as f32 * TAU / 8.0 + 0.2;
        v.push(blade(Shape::Cone, Vec3::ZERO, heading(a, 1.3), 1.2 + 0.1 * (k % 3) as f32, 0.05, 0));
    }
    v
}

fn lavender() -> Vec<Piece> {
    // touffe ronde gris-vert hérissée d'épis violets au bout de fines tiges
    let mut v = vec![ball([0.0, 0.22, 0.0], [0.7, 0.45, 0.7], 1)];
    let base = Vec3::new(0.0, 0.15, 0.0);
    for k in 0..18 {
        let dir = heading(k as f32 * 2.4, 0.6 + 0.15 * (k % 6) as f32);
        v.push(blade(Shape::Cylinder, base, dir, 0.55, 0.012, 1));
        v.push(blade(Shape::Sphere, base + dir * 0.55, dir, 0.14, 0.045, 0));
    }
    v
}

fn tulips() -> Vec<Piece> {
    // carré de seize tulipes, rouges et jaunes, feuilles en lame au pied
    let mut v = vec![bx(0.0, 0.0, 1.0, 0.7, 0.0, 0.08, 3)];
    for k in 0..16 {
        let jitter = ((k * 37) % 7) as f32 * 0.015;
        let (x, z) = ((k % 4) as f32 * 0.22 - 0.33 + jitter, (k / 4) as f32 * 0.16 - 0.24 - jitter);
        let h = 0.38 + jitter * 2.0;
        v.push(cyl(x, z, 0.015, 0.0, h, 2));
        v.push(ball([x, h + 0.03, z], [0.07, 0.1, 0.07], usize::from(k % 3 == 1)));
    }
    for k in 0..8 {
        let a = k as f32 * 2.4;
        let base = Vec3::new(0.4 * (k % 4) as f32 / 3.0 - 0.2, 0.08, 0.3 * (k / 4) as f32 - 0.15);
        v.push(blade(Shape::Sphere, base, heading(a, 1.1), 0.3, 0.05, 2));
    }
    v
}

fn hydrangea() -> Vec<Piece> {
    // buisson rond couvert de grosses têtes de fleurs
    let mut v = clumps(
        &[
            ([0.0, 0.47, 0.0], [1.2, 0.95, 1.2]),
            ([0.35, 0.45, 0.25], [0.8, 0.75, 0.8]),
            ([-0.35, 0.45, -0.25], [0.85, 0.75, 0.8]),
            ([-0.2, 0.5, 0.35], [0.7, 0.7, 0.7]),
        ],
        1,
    );
    for k in 0..14 {
        let dir = heading(k as f32 * 2.4, [0.1, 0.5, 0.9][k % 3]);
        v.push(ball([0.6 * dir.x, 0.47 + 0.48 * dir.y, 0.6 * dir.z], [0.26, 0.22, 0.26], 0));
    }
    v
}

fn rock() -> Vec<Piece> {
    // bloc arrondi un peu enfoncé dans le sol, deux pierres contre lui et une arête vive
    let edge = Quat::from_rotation_y(0.6) * Quat::from_rotation_x(0.35) * Quat::from_rotation_z(0.25);
    vec![
        ball([0.0, 0.4, 0.0], [1.5, 1.0, 1.1], 0).yaw(0.3),
        ball([0.55, 0.28, 0.3], [0.9, 0.7, 0.8], 0).yaw(1.0),
        ball([-0.75, 0.14, 0.35], [0.5, 0.36, 0.45], 0),
        Piece::new(Shape::Cube, Vec3::new(-0.15, 0.55, -0.15), Vec3::new(0.7, 0.5, 0.6), 0).turn(edge),
    ]
}

fn stump() -> Vec<Piece> {
    // souche sciée net, racines qui plongent dans le sol tout autour
    let mut v = vec![cyl(0.0, 0.0, 0.6, 0.0, 0.45, 0), cyl(0.0, 0.0, 0.54, 0.45, 0.47, 1)];
    for k in 0..5 {
        let (s, c) = (k as f32 * TAU / 5.0 + 0.3).sin_cos();
        v.push(span(Shape::Cylinder, Vec3::new(0.2 * s, 0.25, 0.2 * c), Vec3::new(0.6 * s, 0.03, 0.6 * c), 0.14, 0));
    }
    v
}

fn mushrooms() -> Vec<Piece> {
    // amanites tue-mouches : pied blanc, chapeau rouge bombé semé de points blancs
    let mut v = Vec::new();
    let caps = [(0.0, 0.0, 0.16, 0.16), (0.14, 0.08, 0.1, 0.1), (-0.1, 0.1, 0.12, 0.12), (0.06, -0.12, 0.07, 0.08)];
    for (x, z, h, d) in caps.into_iter().chain([(-0.16, -0.06, 0.05, 0.06)]) {
        v.push(cyl(x, z, d * 0.22, 0.0, h, 1));
        v.push(ball([x, h, z], [d, d * 0.55, d], 0));
        for k in 0..3 {
            let (s, c) = (k as f32 * 2.1 + x * 10.0).sin_cos();
            v.push(ball([x + d * 0.25 * s, h + d * 0.24, z + d * 0.25 * c], [d * 0.12; 3], 2));
        }
    }
    v
}

/// Pot rond de diamètre `d` et de hauteur `h`, à rebord, rempli de terre (partie `soil`).
fn pot(d: f32, h: f32, slot: usize, soil: usize) -> [Piece; 3] {
    [
        cyl(0.0, 0.0, d, 0.0, h - 0.06, slot),
        cyl(0.0, 0.0, d + 0.06, h - 0.1, h, slot),
        cyl(0.0, 0.0, d, h, h + 0.01, soil),
    ]
}

fn topiary() -> Vec<Piece> {
    // buis taillé en boule sur une tige, en pot de terre cuite
    let mut v = pot(0.42, 0.44, 2, 3).to_vec();
    v.push(cyl(0.0, 0.0, 0.05, 0.44, 1.05, 1));
    v.push(ball([0.0, 1.3, 0.0], [0.62; 3], 0));
    v
}

fn ficus() -> Vec<Piece> {
    // trois troncs fins qui s'écartent sous une couronne touffue, grand pot blanc
    let mut v = pot(0.46, 0.48, 2, 3).to_vec();
    v.extend(limbs(Vec3::new(0.0, 0.45, 0.0), &[[0.12, 1.3, 0.05], [-0.1, 1.3, 0.08], [0.0, 1.3, -0.12]], 0.05, 1));
    v.extend(clumps(
        &[
            ([0.0, 1.55, 0.0], [0.8, 0.6, 0.8]),
            ([0.25, 1.35, 0.15], [0.55, 0.45, 0.55]),
            ([-0.25, 1.4, -0.1], [0.55, 0.5, 0.55]),
            ([0.1, 1.85, -0.1], [0.5, 0.4, 0.5]),
            ([-0.15, 1.75, 0.2], [0.45, 0.4, 0.45]),
            ([0.2, 1.65, -0.25], [0.4, 0.35, 0.4]),
        ],
        0,
    ));
    v
}

fn bonsai() -> Vec<Piece> {
    // tronc noueux en S dans une coupe basse, plateaux de feuillage taillés à plat
    let knots = [
        Vec3::new(-0.05, 0.1, 0.0),
        Vec3::new(0.06, 0.22, 0.02),
        Vec3::new(-0.04, 0.34, -0.01),
        Vec3::new(0.03, 0.44, 0.0),
    ];
    let mut v = vec![bx(0.0, 0.0, 0.5, 0.34, 0.0, 0.09, 2), bx(0.0, 0.0, 0.46, 0.3, 0.09, 0.1, 3)];
    v.extend(gnarled(&knots, 0.07, 0.012, 1));
    v.push(span(Shape::Cylinder, knots[1], Vec3::new(0.16, 0.31, 0.04), 0.03, 1));
    v.push(span(Shape::Cylinder, knots[2], Vec3::new(-0.16, 0.38, -0.03), 0.03, 1));
    v.extend(clumps(
        &[
            ([0.03, 0.5, 0.0], [0.24, 0.09, 0.2]),
            ([0.17, 0.33, 0.04], [0.22, 0.08, 0.18]),
            ([-0.16, 0.4, -0.03], [0.24, 0.08, 0.18]),
        ],
        0,
    ));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;

    #[test]
    fn flower_heads_mix_three_colors() {
        assert_eq!([flower_head(0), flower_head(1), flower_head(3)], [0, 1, 2]);
    }
    #[test]
    fn heading_points_from_south_to_east_and_up() {
        assert!(heading(0.0, 0.0).abs_diff_eq(Vec3::Z, 1e-6));
        assert!(heading(TAU / 4.0, 0.0).abs_diff_eq(Vec3::X, 1e-6));
        assert!(heading(1.0, TAU / 4.0).abs_diff_eq(Vec3::Y, 1e-6));
    }
    #[test]
    fn pots_are_filled_up_to_the_rim() {
        let [body, rim, soil] = pot(0.4, 0.5, 0, 1);
        let top = |p: &Piece| p.at.y + p.size.y / 2.0;
        assert!((top(&rim) - 0.5).abs() < 1e-6 && top(&body) < top(&rim) && top(&soil) > top(&rim));
        assert!(soil.size.x < rim.size.x, "la terre reste dans le rebord");
    }
    #[test]
    fn indoor_plants_fit_under_a_ceiling() {
        for k in Category::Plants.kinds().filter(|k| !k.outdoor()) {
            assert!(k.size().z < 2.4, "{k:?} touche le plafond");
        }
    }
}
