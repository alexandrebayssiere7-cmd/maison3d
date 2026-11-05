//! Terrasse et jeux : mobilier de jardin, barbecue, spa, jeux d'enfants.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::{Quat, Vec3};

use super::super::{Piece, ball, bx, cyl, legs, light, span};
use super::{WOOD, border, cabin, window};
use crate::model::{DARK, METAL, PropKind, Rgb, Shape, Slot, WHITE};

const ROPE: Rgb = [0.82, 0.74, 0.56];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::GardenTable => const { &[Slot::plain("Plateau", WOOD), Slot::plain("Pieds", DARK)] },
        PropKind::GardenChair => const { &[Slot::plain("Lattes", WOOD), Slot::plain("Structure", DARK)] },
        PropKind::Parasol => {
            const {
                &[
                    Slot::plain("Toile", [0.93, 0.9, 0.8]),
                    Slot::plain("Mât et baleines", WOOD),
                    Slot::plain("Pied", DARK),
                ]
            }
        }
        PropKind::Lounger => const { &[Slot::plain("Matelas", [0.25, 0.5, 0.7]), Slot::plain("Cadre", WOOD)] },
        PropKind::LoungeSet => {
            const {
                &[
                    Slot::textured("Coussins", "Tissu beige"),
                    Slot::plain("Structure", [0.32, 0.3, 0.28]),
                    Slot::plain("Plateau", [0.85, 0.85, 0.82]),
                    Slot::plain("Coussins déco", [0.2, 0.45, 0.5]),
                ]
            }
        }
        PropKind::Barbecue => {
            const { &[Slot::plain("Cuve", [0.1, 0.1, 0.11]), Slot::plain("Pieds", METAL), Slot::plain("Poignées", WOOD)] }
        }
        PropKind::Firepit => {
            const {
                &[
                    Slot::plain("Vasque", [0.35, 0.2, 0.12]),
                    Slot::textured("Bûches", "Écorce"),
                    Slot::plain("Flammes", [1.0, 0.55, 0.15]),
                ]
            }
        }
        PropKind::Hammock => {
            const { &[Slot::plain("Toile", [0.9, 0.55, 0.2]), Slot::plain("Bois", WOOD), Slot::plain("Cordes", ROPE)] }
        }
        PropKind::HotTub => {
            const {
                &[
                    Slot::textured("Habillage", "Bardage gris"),
                    Slot::plain("Coque", [0.9, 0.92, 0.94]),
                    Slot::plain("Eau", [0.35, 0.72, 0.85]),
                    Slot::plain("Appuie-tête", DARK),
                ]
            }
        }
        _ => game_slots(kind),
    }
}

/// Parties des jeux d'enfants.
fn game_slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Swing => {
            const {
                &[
                    Slot::textured("Portique", "Bois clair"),
                    Slot::plain("Cordes", ROPE),
                    Slot::plain("Sièges", [0.85, 0.2, 0.15]),
                ]
            }
        }
        PropKind::Slide => {
            const {
                &[
                    Slot::plain("Glissière", [0.95, 0.75, 0.1]),
                    Slot::plain("Structure", [0.2, 0.4, 0.75]),
                    Slot::textured("Marches", "Bois clair"),
                ]
            }
        }
        PropKind::Trampoline => {
            const {
                &[
                    Slot::plain("Coussin de protection", [0.15, 0.45, 0.85]),
                    Slot::plain("Toile", [0.08, 0.08, 0.08]),
                    Slot::plain("Structure", METAL),
                ]
            }
        }
        PropKind::Sandbox => {
            const {
                &[
                    Slot::textured("Bordure", "Bois clair"),
                    Slot::textured("Sable", "Sable"),
                    Slot::plain("Seau", [0.9, 0.2, 0.15]),
                    Slot::plain("Pelle", [0.15, 0.5, 0.85]),
                ]
            }
        }
        PropKind::Seesaw => {
            const {
                &[
                    Slot::plain("Planche", [0.85, 0.25, 0.15]),
                    Slot::plain("Support", [0.2, 0.4, 0.75]),
                    Slot::plain("Poignées", [0.95, 0.8, 0.1]),
                    Slot::plain("Sièges", DARK),
                ]
            }
        }
        PropKind::PlayHouse => {
            const {
                &[
                    Slot::plain("Murs", [0.95, 0.85, 0.6]),
                    Slot::plain("Toit", [0.85, 0.25, 0.2]),
                    Slot::plain("Porte", [0.3, 0.6, 0.35]),
                    Slot::plain("Poignée", METAL),
                    Slot::textured("Plancher", "Plancher brut"),
                    Slot::plain("Fenêtre", [0.55, 0.75, 0.9]),
                    Slot::plain("Encadrement", WHITE),
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, open: f32) -> Vec<Piece> {
    match kind {
        PropKind::GardenTable => garden_table(),
        PropKind::GardenChair => garden_chair(),
        PropKind::Parasol => parasol(),
        PropKind::Lounger => lounger(),
        PropKind::LoungeSet => lounge_set(),
        PropKind::Barbecue => barbecue(),
        PropKind::Firepit => firepit(),
        PropKind::Hammock => hammock(),
        PropKind::HotTub => hot_tub(),
        PropKind::Swing => swing_set(),
        PropKind::Slide => slide(),
        PropKind::Trampoline => trampoline(),
        PropKind::Sandbox => sandbox(),
        PropKind::Seesaw => seesaw(),
        PropKind::PlayHouse => {
            let mut v = cabin((1.4, 1.2), (1.3, 1.85), (-0.25, 0.55, 1.1), open);
            v.extend(window(Vec3::new(0.38, 0.8, 0.61), (0.35, 0.35), 0.0, (5, 6)));
            v
        }
        _ => Vec::new(),
    }
}

/// Table à lattes de 1,6 × 0,9 m, ceinture et pieds en métal.
fn garden_table() -> Vec<Piece> {
    let mut v: Vec<Piece> = (0..6).map(|k| bx(0.0, -0.375 + k as f32 * 0.15, 1.6, 0.13, 0.71, 0.74, 0)).collect();
    v.extend(border(1.5, 0.84, 0.04, (0.63, 0.71), 1));
    v.extend(legs(0.72, 0.38, 0.71, 1));
    v
}

/// Chaise à lattes, dossier légèrement incliné vers l'arrière (-z).
fn garden_chair() -> Vec<Piece> {
    let lean = 0.1 / 0.51;
    let mut v = legs(0.2, 0.2, 0.44, 1).to_vec();
    for k in 0..4 {
        v.push(bx(0.0, -0.18 + k as f32 * 0.12, 0.46, 0.1, 0.44, 0.47, 0));
    }
    for x in [-0.2, 0.2] {
        v.push(span(Shape::Cube, Vec3::new(x, 0.44, -0.2), Vec3::new(x, 0.95, -0.3), 0.04, 1));
    }
    for y in [0.62, 0.74, 0.86] {
        let at = Vec3::new(0.0, y, -0.17 - (y - 0.44) * lean);
        v.push(Piece::new(Shape::Cube, at, Vec3::new(0.44, 0.09, 0.02), 0).turn(Quat::from_rotation_x(-lean.atan())));
    }
    v
}

/// Parasol de 2,8 m sur pied lesté, toile penchée vers l'avant, baleines dessous.
fn parasol() -> Vec<Piece> {
    let (tilt, top) = (Quat::from_rotation_x(0.12), Vec3::new(0.0, 2.3, 0.0));
    let mut v = vec![
        cyl(0.0, 0.0, 0.5, 0.0, 0.08, 2),
        cyl(0.0, 0.0, 0.045, 0.08, 2.35, 1),
        Piece::new(Shape::Cone, top, Vec3::new(2.8, 0.45, 2.8), 0).turn(tilt),
        ball((top + tilt * Vec3::new(0.0, 0.25, 0.0)).to_array(), [0.07; 3], 1),
    ];
    for k in 0..8 {
        let (s, c) = (k as f32 * TAU / 8.0).sin_cos();
        let rim = top + tilt * Vec3::new(1.35 * c, -0.24, 1.35 * s);
        v.push(span(Shape::Cylinder, Vec3::new(0.0, 1.95, 0.0), rim, 0.015, 1));
    }
    v
}

/// Bain de soleil : matelas sur un cadre bas, dossier relevé à l'arrière (-z), pieds vers l'avant.
fn lounger() -> Vec<Piece> {
    let (b0, b1) = (Vec3::new(0.0, 0.36, -0.4), Vec3::new(0.0, 0.88, -0.85));
    let d = b1 - b0;
    let q = Quat::from_rotation_x(d.z.atan2(d.y));
    let back = (b0 + b1) / 2.0 + q * Vec3::new(0.0, 0.0, 0.04);
    let mut v = vec![
        bx(0.0, 0.25, 0.58, 1.3, 0.32, 0.4, 0),
        Piece::new(Shape::Cube, back, Vec3::new(0.58, d.length(), 0.08), 0).turn(q),
    ];
    for x in [-0.3, 0.3] {
        v.push(bx(x, 0.25, 0.04, 1.3, 0.25, 0.32, 1));
        v.push(bx(x, 0.85, 0.04, 0.04, 0.0, 0.25, 1));
        v.push(bx(x, -0.35, 0.04, 0.04, 0.0, 0.25, 1));
        v.push(span(Shape::Cube, Vec3::new(x, 0.32, -0.4), Vec3::new(x, 0.85, -0.86), 0.04, 1));
        v.push(span(Shape::Cube, Vec3::new(x, 0.6, -0.64), Vec3::new(x, 0.3, -0.25), 0.03, 1));
    }
    v
}

/// Canapé bas de 2 m à l'arrière, tourné vers l'avant, et table basse devant lui.
fn lounge_set() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, -0.45, 2.0, 0.8, 0.0, 0.32, 1),
        bx(0.0, -0.78, 2.0, 0.14, 0.32, 0.72, 1),
        bx(-0.93, -0.38, 0.14, 0.66, 0.32, 0.58, 1),
        bx(0.93, -0.38, 0.14, 0.66, 0.32, 0.58, 1),
        bx(0.0, 0.55, 1.0, 0.55, 0.0, 0.3, 1),
        bx(0.0, 0.55, 1.04, 0.59, 0.3, 0.34, 2),
        Piece::new(Shape::Cube, Vec3::new(-0.6, 0.58, -0.52), Vec3::new(0.35, 0.3, 0.1), 3).yaw(0.3),
        Piece::new(Shape::Cube, Vec3::new(0.62, 0.58, -0.52), Vec3::new(0.35, 0.3, 0.1), 3).yaw(-0.25),
    ];
    for x in [-0.57, 0.0, 0.57] {
        v.push(bx(x, -0.33, 0.56, 0.54, 0.32, 0.44, 0));
        v.push(bx(x, -0.65, 0.56, 0.12, 0.44, 0.7, 0));
    }
    v
}

/// Barbecue boule à charbon, couvercle fermé, sur trois pieds.
fn barbecue() -> Vec<Piece> {
    let mut v = vec![
        ball([0.0, 0.8, 0.0], [0.58, 0.52, 0.58], 0),
        cyl(0.0, 0.0, 0.6, 0.79, 0.81, 1),
        cyl(0.0, 0.0, 0.48, 0.2, 0.22, 1),
        bx(0.0, 0.0, 0.16, 0.04, 1.05, 1.09, 2),
        bx(-0.33, 0.0, 0.08, 0.03, 0.76, 0.79, 2),
        bx(0.33, 0.0, 0.08, 0.03, 0.76, 0.79, 2),
    ];
    for k in 0..3 {
        let (s, c) = (k as f32 * TAU / 3.0 + FRAC_PI_2).sin_cos();
        v.push(span(Shape::Cylinder, Vec3::new(0.2 * c, 0.62, 0.2 * s), Vec3::new(0.3 * c, 0.0, 0.3 * s), 0.03, 1));
    }
    v
}

/// Brasero : vasque d'acier sur trois pieds, bûches et flammes qui éclairent.
fn firepit() -> Vec<Piece> {
    let mut v = vec![
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.45, 0.0), Vec3::new(0.8, 0.3, 0.8), 0).turn(Quat::from_rotation_x(PI)),
        cyl(0.0, 0.0, 0.84, 0.58, 0.62, 0),
        span(Shape::Cylinder, Vec3::new(-0.25, 0.66, -0.08), Vec3::new(0.25, 0.66, 0.08), 0.08, 1),
        span(Shape::Cylinder, Vec3::new(-0.1, 0.7, 0.24), Vec3::new(0.1, 0.7, -0.24), 0.08, 1),
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.88, 0.0), Vec3::new(0.28, 0.4, 0.28), 2),
        Piece::new(Shape::Cone, Vec3::new(0.09, 0.82, 0.06), Vec3::new(0.16, 0.28, 0.16), 2),
        Piece::new(Shape::Cone, Vec3::new(-0.09, 0.8, -0.05), Vec3::new(0.14, 0.24, 0.14), 2),
        light([0.0, 1.0, 0.0], 2),
    ];
    for k in 0..3 {
        let (s, c) = (k as f32 * TAU / 3.0).sin_cos();
        v.push(span(Shape::Cylinder, Vec3::new(0.2 * c, 0.45, 0.2 * s), Vec3::new(0.35 * c, 0.0, 0.35 * s), 0.04, 0));
    }
    v
}

/// Hamac tendu entre deux poteaux à 3,6 m l'un de l'autre, toile en creux.
fn hammock() -> Vec<Piece> {
    let mut v = vec![cyl(-1.8, 0.0, 0.12, 0.0, 1.7, 1), cyl(1.8, 0.0, 0.12, 0.0, 1.7, 1)];
    let sag = [(-1.15, 0.95), (-0.6, 0.68), (0.0, 0.6), (0.6, 0.68), (1.15, 0.95)].map(|(x, y)| Vec3::new(x, y, 0.0));
    for w in sag.windows(2) {
        let d = w[1] - w[0];
        let cloth = Piece::new(Shape::Cube, (w[0] + w[1]) / 2.0, Vec3::new(d.length(), 0.02, 0.8), 0);
        v.push(cloth.turn(Quat::from_rotation_z(d.y.atan2(d.x))));
    }
    for sx in [-1.0, 1.0] {
        let bar = Piece::new(Shape::Cylinder, Vec3::new(sx * 1.15, 0.95, 0.0), Vec3::new(0.03, 0.84, 0.03), 1);
        v.push(bar.turn(Quat::from_rotation_x(FRAC_PI_2)));
        for z in [-0.4, 0.0, 0.4] {
            v.push(span(Shape::Cylinder, Vec3::new(sx * 1.15, 0.95, z), Vec3::new(sx * 1.74, 1.55, 0.0), 0.015, 2));
        }
    }
    v
}

/// Spa carré de 2,1 m, 0,9 m de haut : habillage, coque, eau, quatre appuie-tête.
fn hot_tub() -> Vec<Piece> {
    let s = 2.1;
    let mut v = border(s, s, 0.08, (0.0, 0.85), 0).to_vec();
    v.extend(border(s + 0.04, s + 0.04, 0.14, (0.85, 0.9), 1));
    v.extend(border(s - 0.16, s - 0.16, 0.04, (0.55, 0.85), 1));
    v.push(bx(0.0, 0.0, s - 0.24, s - 0.24, 0.0, 0.72, 2));
    for (x, z) in [(-0.5, -0.9), (0.5, -0.9), (-0.5, 0.9), (0.5, 0.9)] {
        v.push(bx(x, z, 0.32, 0.06, 0.72, 0.86, 3));
    }
    v
}

/// Portique en A de 2,8 m, poutre à 2,1 m, deux sièges suspendus.
fn swing_set() -> Vec<Piece> {
    let (hw, top) = (1.3, 2.1);
    let mut v = vec![Piece::new(Shape::Wheel, Vec3::new(0.0, top, 0.0), Vec3::new(2.8, 0.1, 0.1), 0)];
    for x in [-hw, hw] {
        for z in [-0.8, 0.8] {
            v.push(span(Shape::Cylinder, Vec3::new(x, top, 0.0), Vec3::new(x * 1.06, 0.0, z), 0.08, 0));
        }
        v.push(bx(x * 1.04, 0.0, 0.05, 1.1, 0.72, 0.78, 0));
    }
    for sx in [-0.5, 0.5] {
        for dx in [-0.2, 0.2] {
            v.push(span(Shape::Cylinder, Vec3::new(sx + dx, top, 0.0), Vec3::new(sx + dx, 0.46, 0.0), 0.015, 1));
        }
        v.push(bx(sx, 0.0, 0.46, 0.18, 0.42, 0.46, 2));
    }
    v
}

/// Toboggan : échelle à l'arrière (-z), plateforme à 1,2 m, glissière qui descend vers l'avant.
fn slide() -> Vec<Piece> {
    let (deck, rail) = (1.2, 1.7);
    let ladder = |y: f32| -1.55 + y * 0.55 / deck;
    let mut v = vec![bx(0.0, -0.75, 0.55, 0.5, deck - 0.04, deck, 2)];
    for x in [-0.25, 0.25] {
        v.push(span(Shape::Cube, Vec3::new(x, 0.0, ladder(0.0)), Vec3::new(x, rail, ladder(rail)), 0.05, 1));
        v.push(span(Shape::Cube, Vec3::new(x, rail, ladder(rail)), Vec3::new(x, rail, -0.52), 0.05, 1));
        v.push(bx(x, -0.52, 0.05, 0.05, 0.0, rail, 1));
    }
    for k in 1..5 {
        let y = k as f32 * 0.24;
        v.push(bx(0.0, ladder(y), 0.5, 0.08, y - 0.02, y + 0.02, 2));
    }
    // glissière et ses rebords, de la plateforme jusqu'à 30 cm du sol, puis un replat
    let (a, b) = (Vec3::new(0.0, deck, -0.5), Vec3::new(0.0, 0.3, 1.25));
    let d = b - a;
    let q = Quat::from_rotation_x(-d.y.atan2(d.z));
    for (x, w, h) in [(0.0, 0.45, 0.03), (-0.24, 0.03, 0.15), (0.24, 0.03, 0.15)] {
        let at = (a + b) / 2.0 + Vec3::new(x, h / 2.0, 0.0);
        v.push(Piece::new(Shape::Cube, at, Vec3::new(w, h, d.length()), 0).turn(q));
    }
    v.push(bx(0.0, 1.4, 0.45, 0.3, 0.27, 0.3, 0));
    v.push(bx(0.0, 1.45, 0.4, 0.05, 0.0, 0.27, 1));
    v
}

/// Trampoline rond de 3 m, toile à 87 cm sur six pieds.
fn trampoline() -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 3.0, 0.8, 0.86, 0), cyl(0.0, 0.0, 2.5, 0.86, 0.87, 1)];
    for k in 0..6 {
        let (s, c) = (k as f32 * TAU / 6.0).sin_cos();
        v.push(cyl(1.4 * c, 1.4 * s, 0.05, 0.0, 0.8, 2));
    }
    v
}

/// Bac à sable de 1,6 m, sièges d'angle, seau et pelle.
fn sandbox() -> Vec<Piece> {
    let mut v = border(1.6, 1.6, 0.1, (0.0, 0.25), 0).to_vec();
    v.push(bx(0.0, 0.0, 1.4, 1.4, 0.0, 0.17, 1));
    v.push(ball([-0.25, 0.17, -0.2], [0.5, 0.18, 0.4], 1));
    for (x, z) in [(-0.65, -0.65), (0.65, -0.65), (-0.65, 0.65), (0.65, 0.65)] {
        v.push(bx(x, z, 0.3, 0.3, 0.25, 0.28, 0));
    }
    v.push(cyl(0.3, 0.15, 0.16, 0.17, 0.33, 2));
    v.push(span(Shape::Cylinder, Vec3::new(-0.1, 0.19, 0.35), Vec3::new(0.15, 0.19, 0.45), 0.025, 3));
    v.push(bx(-0.16, 0.33, 0.1, 0.12, 0.17, 0.2, 3));
    v
}

/// Tape-cul de 3 m, un bout au sol ; sièges et poignées aux deux bouts.
fn seesaw() -> Vec<Piece> {
    let (h, half) = (0.5_f32, 1.5);
    let tilt = (h / half).asin();
    let (dir, q, axle) = (Vec3::new(tilt.cos(), tilt.sin(), 0.0), Quat::from_rotation_z(tilt), Vec3::new(0.0, h, 0.0));
    let mut v = vec![
        Piece::new(Shape::Cube, axle + Vec3::new(0.0, 0.03, 0.0), Vec3::new(2.0 * half, 0.05, 0.25), 0).turn(q),
        Piece::new(Shape::Cylinder, axle, Vec3::new(0.1, 0.45, 0.1), 1).turn(Quat::from_rotation_x(FRAC_PI_2)),
    ];
    for z in [-0.2, 0.2] {
        for sx in [-1.0, 1.0] {
            v.push(span(Shape::Cube, Vec3::new(0.0, h, z), Vec3::new(sx * 0.35, 0.0, z * 1.6), 0.06, 1));
        }
    }
    for s in [-1.3, 1.3] {
        v.push(
            Piece::new(Shape::Cube, axle + dir * s + Vec3::new(0.0, 0.08, 0.0), Vec3::new(0.35, 0.05, 0.3), 3).turn(q),
        );
        let grip = axle + dir * (s - s.signum() * 0.3) + Vec3::new(0.0, 0.05, 0.0);
        v.push(span(Shape::Cylinder, grip, grip + Vec3::new(0.0, 0.3, 0.0), 0.04, 2));
        v.push(span(Shape::Cylinder, grip + Vec3::new(0.0, 0.3, -0.15), grip + Vec3::new(0.0, 0.3, 0.15), 0.035, 2));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use crate::model::shapes::prop_shapes;

    #[test]
    fn only_the_firepit_lights_and_only_the_playhouse_opens() {
        for k in Category::Leisure.kinds() {
            let lights = prop_shapes(k, 0.0, 3.0).iter().filter(|p| p.shape == Shape::Light).count();
            assert_eq!(lights, usize::from(k == PropKind::Firepit), "{k:?}");
            assert_eq!(k.openable(), k == PropKind::PlayHouse, "{k:?}");
        }
    }
    #[test]
    fn parasol_shades_above_head_height() {
        let canopy = parasol()[2];
        let rim = |z: f32| canopy.at + canopy.rot * Vec3::new(0.0, -canopy.size.y / 2.0, z * canopy.size.z / 2.0);
        assert!(rim(1.0).y > 1.85 && rim(1.0).y < rim(-1.0).y, "toile penchée vers l'avant, au-dessus des têtes");
    }
    #[test]
    fn slide_goes_down_to_the_front_and_seesaw_touches_the_ground() {
        let chute = slide().into_iter().find(|p| p.slot == 0).expect("glissière");
        let end = chute.at + chute.rot * Vec3::new(0.0, 0.0, chute.size.z / 2.0);
        assert!(end.z > 1.0 && (end.y - 0.3).abs() < 0.02, "bas de la glissière : {end}");
        let plank = seesaw()[0];
        let low = plank.at - plank.rot * Vec3::new(plank.size.x / 2.0, plank.size.y / 2.0, 0.0);
        assert!(low.y.abs() < 0.03, "bout bas du tape-cul : {low}");
    }
    #[test]
    fn swing_seats_hang_under_the_beam() {
        let seats: Vec<_> = swing_set().into_iter().filter(|p| p.slot == 2).collect();
        assert_eq!(seats.len(), 2);
        assert!(seats.iter().all(|p| p.at.y < 0.5));
    }
}
