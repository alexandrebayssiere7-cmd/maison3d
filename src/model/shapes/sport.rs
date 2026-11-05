//! Équipements sportifs.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::math::{Quat, Vec3};

use super::{Piece, ball, bx, cyl, light, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const NET: [f32; 3] = [0.9, 0.9, 0.88];
const RED: [f32; 3] = [0.8, 0.1, 0.1];
const ORANGE: [f32; 3] = [0.85, 0.38, 0.1];
const STEEL: [f32; 3] = [0.25, 0.26, 0.28];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    gear_slots(kind)
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    gear(kind)
}

/// Buts, ballons, filets, poteaux et tables.
fn gear_slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::SoccerGoal => {
            const { &[Slot::plain("Poteaux", WHITE), Slot::plain("Filet", NET), Slot::plain("Structure", METAL)] }
        }
        PropKind::HandballGoal => {
            const {
                &[
                    Slot::plain("Poteaux", WHITE),
                    Slot::plain("Filet", NET),
                    Slot::plain("Structure", METAL),
                    Slot::plain("Bandes", RED),
                ]
            }
        }
        PropKind::SoccerBall => const { &[Slot::plain("Cuir", WHITE), Slot::plain("Pentagones", DARK)] },
        PropKind::Basketball => const { &[Slot::plain("Cuir", ORANGE), Slot::plain("Coutures", DARK)] },
        PropKind::BasketHoop => {
            const {
                &[
                    Slot::plain("Pied", STEEL),
                    Slot::plain("Planche", WHITE),
                    Slot::plain("Carré", RED),
                    Slot::plain("Cercle", ORANGE),
                    Slot::plain("Filet", WHITE),
                    Slot::plain("Socle", DARK),
                ]
            }
        }
        PropKind::TennisNet => {
            const { &[Slot::plain("Poteaux", [0.1, 0.3, 0.2]), Slot::plain("Filet", DARK), Slot::plain("Bande", WHITE)] }
        }
        PropKind::VolleyNet => {
            const {
                &[
                    Slot::plain("Poteaux", METAL),
                    Slot::plain("Filet", DARK),
                    Slot::plain("Bandes", WHITE),
                    Slot::plain("Antennes", RED),
                ]
            }
        }
        PropKind::RugbyPosts => {
            const { &[Slot::plain("Poteaux", WHITE), Slot::plain("Protections", [0.1, 0.25, 0.6])] }
        }
        PropKind::PingPongTable => {
            const {
                &[
                    Slot::plain("Plateau", [0.08, 0.22, 0.45]),
                    Slot::plain("Lignes", WHITE),
                    Slot::plain("Filet", DARK),
                    Slot::plain("Pieds", STEEL),
                ]
            }
        }
        _ => venue_slots(kind),
    }
}

/// Abords du terrain, athlétisme et salle de sport.
fn venue_slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Bleachers => {
            const {
                &[
                    Slot::plain("Assises", [0.75, 0.77, 0.8]),
                    Slot::plain("Structure", STEEL),
                    Slot::plain("Garde-corps", METAL),
                ]
            }
        }
        PropKind::DugoutBench => {
            const { &[Slot::plain("Cadre", METAL), Slot::plain("Sièges", [0.1, 0.2, 0.55]), Slot::plain("Toit", WHITE)] }
        }
        PropKind::CornerFlag => const { &[Slot::plain("Mât", WHITE), Slot::plain("Fanion", [0.95, 0.8, 0.1])] },
        PropKind::Floodlight => {
            const {
                &[
                    Slot::plain("Mât", METAL),
                    Slot::plain("Rampe", STEEL),
                    Slot::plain("Projecteurs", [0.3, 0.3, 0.32]),
                    Slot::plain("Vitres", [1.0, 0.98, 0.9]),
                    Slot::plain("Lumière", [1.0, 0.97, 0.9]),
                ]
            }
        }
        PropKind::Scoreboard => {
            const {
                &[
                    Slot::plain("Pieds", STEEL),
                    Slot::plain("Caisson", [0.2, 0.2, 0.22]),
                    Slot::plain("Écran", [0.03, 0.03, 0.04]),
                    Slot::plain("Chiffres", [1.0, 0.75, 0.1]),
                    Slot::plain("Équipes", WHITE),
                ]
            }
        }
        PropKind::WeightBench => {
            const {
                &[
                    Slot::textured("Coussin", "Cuir noir"),
                    Slot::plain("Structure", STEEL),
                    Slot::textured("Barre", "Chrome"),
                    Slot::plain("Disques", [0.1, 0.1, 0.1]),
                ]
            }
        }
        PropKind::Treadmill => {
            const {
                &[
                    Slot::plain("Châssis", STEEL),
                    Slot::plain("Bande", [0.06, 0.06, 0.07]),
                    Slot::plain("Console", DARK),
                    Slot::plain("Écran", [0.2, 0.45, 0.7]),
                ]
            }
        }
        PropKind::PunchingBag => {
            const {
                &[
                    Slot::plain("Sac", [0.55, 0.08, 0.08]),
                    Slot::plain("Bandes", DARK),
                    Slot::plain("Potence", STEEL),
                    Slot::plain("Chaînes", METAL),
                ]
            }
        }
        PropKind::Hurdle => {
            const {
                &[
                    Slot::plain("Barre", WHITE),
                    Slot::plain("Bandes", DARK),
                    Slot::plain("Montants", METAL),
                    Slot::plain("Pieds", STEEL),
                ]
            }
        }
        _ => const { &[] },
    }
}

fn gear(kind: PropKind) -> Vec<Piece> {
    match kind {
        PropKind::SoccerGoal => goal(Shape::Cylinder, (3.66, 2.44, 2.0), 0.12, 0.5),
        PropKind::HandballGoal => handball_goal(),
        PropKind::SoccerBall => soccer_ball(),
        PropKind::Basketball => basketball(),
        PropKind::BasketHoop => basket_hoop(),
        PropKind::TennisNet => tennis_net(),
        PropKind::VolleyNet => volley_net(),
        PropKind::RugbyPosts => rugby_posts(),
        PropKind::PingPongTable => ping_pong_table(),
        PropKind::Bleachers => bleachers(),
        PropKind::DugoutBench => dugout(),
        PropKind::CornerFlag => vec![cyl(0.0, 0.0, 0.03, 0.0, 1.5, 0), bx(0.225, 0.0, 0.45, 0.005, 1.14, 1.49, 1)],
        PropKind::Floodlight => floodlight(),
        PropKind::Scoreboard => scoreboard(),
        PropKind::WeightBench => weight_bench(),
        PropKind::Treadmill => treadmill(),
        PropKind::PunchingBag => punching_bag(),
        PropKind::Hurdle => hurdle(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------- briques

/// Bande posée à plat le long d'un arc de cercle de centre `c` et de rayon `r`, de l'angle `a0`
/// à `a1` (radians, 0 = +x, π/2 = -z), large de `w` et épaisse de `h` ; d'autant plus de segments
/// que le cercle est grand, pour rester lisse.
fn arc(c: Vec3, r: f32, (a0, a1): (f32, f32), (w, h): (f32, f32), slot: usize) -> Vec<Piece> {
    // segments d'environ 50 cm, au moins 4 par radian (petits cercles), au plus 100 (grands)
    let n = ((a1 - a0).abs() * (2.0 * r).clamp(4.0, 100.0)).ceil().max(1.0) as usize;
    let step = (a1 - a0) / n as f32;
    // les segments se chevauchent un peu pour que la courbe reste continue
    let chord = 2.0 * r * (step / 2.0).sin().abs() + w * 0.5;
    (0..n)
        .map(|k| {
            let m = a0 + (k as f32 + 0.5) * step;
            let at = c + Vec3::new(m.cos(), 0.0, -m.sin()) * r;
            Piece::new(Shape::Cube, at, Vec3::new(chord, h, w), slot).yaw(m + FRAC_PI_2)
        })
        .collect()
}

/// But : poteaux et barre de section `p` (`shape` ronde ou carrée) autour d'une ouverture
/// `2 hw × h` tournée vers l'avant en z = `d / 2`, filet de maille `mesh` tendu jusqu'au sol en
/// z = `-d / 2`, haut du filet profond de `d / 2`. Parties : poteaux 0, filet 1, structure 2.
fn goal(shape: Shape, (hw, h, d): (f32, f32, f32), p: f32, mesh: f32) -> Vec<Piece> {
    let (zf, x, s, t) = (d / 2.0, hw + p / 2.0, 0.04, 0.012);
    let top = |x: f32| Vec3::new(x, h, 0.0);
    let back = |x: f32| Vec3::new(x, s / 2.0, -zf);
    let tube = |a: Vec3, b: Vec3| span(Shape::Cylinder, a, b, s, 2);
    let wire = |a: Vec3, b: Vec3| span(Shape::Cube, a, b, t, 1);
    let mut v = vec![
        span(shape, Vec3::new(-x, 0.0, zf), Vec3::new(-x, h + p, zf), p, 0),
        span(shape, Vec3::new(x, 0.0, zf), Vec3::new(x, h + p, zf), p, 0),
        span(shape, Vec3::new(-x - p / 2.0, h + p / 2.0, zf), Vec3::new(x + p / 2.0, h + p / 2.0, zf), p, 0),
        tube(top(-hw), top(hw)),
        tube(back(-hw), back(hw)),
    ];
    for sx in [-hw, hw] {
        v.extend([
            tube(Vec3::new(sx, h, zf), top(sx)),
            tube(top(sx), back(sx)),
            tube(Vec3::new(sx, s / 2.0, zf), back(sx)),
        ]);
    }
    // fils verticaux du toit et du fond
    let nx = (2.0 * hw / mesh).round() as usize;
    for i in 1..nx {
        let x = -hw + 2.0 * hw * i as f32 / nx as f32;
        v.extend([wire(Vec3::new(x, h, zf), top(x)), wire(top(x), back(x))]);
    }
    // fils horizontaux du fond et des côtés, à la même hauteur
    let nb = ((h * h + zf * zf).sqrt() / mesh).round() as usize;
    for k in 1..nb {
        let q = top(0.0).lerp(back(0.0), k as f32 / nb as f32);
        v.push(wire(Vec3::new(-hw, q.y, q.z), Vec3::new(hw, q.y, q.z)));
        v.extend([-hw, hw].map(|sx| wire(Vec3::new(sx, q.y, zf), Vec3::new(sx, q.y, q.z))));
    }
    // fils verticaux des côtés, jusqu'au bord penché du fond
    let ns = (d / mesh).round() as usize;
    for k in 1..ns {
        let z = zf - d * k as f32 / ns as f32;
        let y = if z >= 0.0 { h } else { h * (1.0 + z / zf) };
        v.extend([-hw, hw].map(|sx| bx(sx, z, t, t, 0.0, y, 1)));
    }
    v
}

/// Filet tendu le long de x entre ±`hw`, du bas `y0` au haut qui passe de `edge` aux bouts à
/// `mid` au centre ; fils verticaux tous les `step`, trois fils horizontaux, partie `slot`.
fn net(hw: f32, y0: f32, (edge, mid): (f32, f32), step: f32, slot: usize) -> Vec<Piece> {
    let t = 0.008;
    let n = (2.0 * hw / step).round() as usize;
    let mut v: Vec<Piece> = (0..=n)
        .map(|i| {
            let x = -hw + 2.0 * hw * i as f32 / n as f32;
            bx(x, 0.0, t, t, y0, mid + (edge - mid) * x.abs() / hw, slot)
        })
        .collect();
    for f in [0.25, 0.5, 0.75] {
        let (ye, ym) = (y0 + (edge - y0) * f, y0 + (mid - y0) * f);
        for sx in [-hw, hw] {
            v.push(span(Shape::Cube, Vec3::new(sx, ye, 0.0), Vec3::new(0.0, ym, 0.0), t, slot));
        }
    }
    v
}

// ---------------------------------------------------------------- buts et ballons

/// But de hand de 3 × 2 m aux poteaux carrés rayés de bandes de 20 cm.
fn handball_goal() -> Vec<Piece> {
    let (p, zf, s) = (0.08, 0.5, 0.084);
    let mut v = goal(Shape::Cube, (1.5, 2.0, 1.0), p, 0.3);
    for k in 0..5 {
        let y = 0.4 * k as f32;
        v.extend([-1.54, 1.54].map(|x| bx(x, zf, s, s, y, y + 0.2, 3)));
    }
    v.extend((0..8).map(|k| bx(-1.4 + 0.4 * k as f32, zf, 0.2, s, 1.998, 2.082, 3)));
    v
}

/// Ballon de 22 cm aux douze pentagones sombres (sommets d'un icosaèdre).
fn soccer_ball() -> Vec<Piece> {
    let (r, phi) = (0.11, 1.618);
    let c = Vec3::Y * r;
    let mut v = vec![ball([0.0, r, 0.0], [2.0 * r; 3], 0)];
    for (a, b) in [(1.0, phi), (1.0, -phi), (-1.0, phi), (-1.0, -phi)] {
        for d in [Vec3::new(0.0, a, b), Vec3::new(a, b, 0.0), Vec3::new(b, 0.0, a)] {
            let d = d.normalize();
            let patch = Piece::new(Shape::Sphere, c + d * (r - 0.006), Vec3::new(0.075, 0.016, 0.075), 1);
            v.push(patch.turn(Quat::from_rotation_arc(Vec3::Y, d)));
        }
    }
    v
}

/// Ballon de 24 cm ; ses coutures sont des disques à peine plus larges que lui, dont seul le
/// bord dépasse.
fn basketball() -> Vec<Piece> {
    let (d, y) = (0.24, 0.12);
    let seam = |q: Quat| Piece::new(Shape::Cylinder, Vec3::Y * y, Vec3::new(d + 0.004, 0.006, d + 0.004), 1).turn(q);
    vec![
        ball([0.0, y, 0.0], [d; 3], 0),
        seam(Quat::IDENTITY),
        seam(Quat::from_rotation_x(FRAC_PI_2)),
        seam(Quat::from_rotation_z(FRAC_PI_2)),
    ]
}

/// Panier sur pied : socle et pied à l'arrière, planche en z = 0,35, cercle de 45 cm à 3,05 m
/// dont le bord intérieur est à 15 cm de la planche.
fn basket_hoop() -> Vec<Piece> {
    let (zp, zb, y, r) = (-0.9, 0.35, 3.04, 0.235);
    let zr = zb + 0.025 + 0.15 + r - 0.01;
    let face = zb + 0.027;
    let mut v = vec![
        bx(0.0, zp, 0.8, 1.0, 0.0, 0.25, 5),
        cyl(0.0, zp, 0.16, 0.25, 3.45, 0),
        span(Shape::Cube, Vec3::new(0.0, 3.4, zp), Vec3::new(0.0, 3.4, zb - 0.025), 0.1, 0),
        span(Shape::Cube, Vec3::new(0.0, 2.6, zp), Vec3::new(0.0, 3.35, zb - 0.3), 0.06, 0),
        bx(0.0, zb, 1.8, 0.05, 2.9, 3.95, 1),
        // carré de visée de 59 × 45 cm au-dessus du cercle
        bx(0.0, face, 0.59, 0.004, 3.05, 3.1, 2),
        bx(0.0, face, 0.59, 0.004, 3.45, 3.5, 2),
        bx(-0.27, face, 0.05, 0.004, 3.05, 3.5, 2),
        bx(0.27, face, 0.05, 0.004, 3.05, 3.5, 2),
        bx(0.0, (face + zr - r) / 2.0, 0.12, zr - r - face, y - 0.05, y + 0.01, 3),
    ];
    v.extend(arc(Vec3::new(0.0, y, zr), r, (0.0, TAU), (0.02, 0.02), 3));
    // filet : fils croisés du cercle vers un anneau plus étroit 40 cm plus bas
    let at = |a: f32, r: f32, y: f32| Vec3::new(a.cos() * r, y, zr - a.sin() * r);
    for k in 0..12 {
        let a = k as f32 * TAU / 12.0;
        for da in [-0.5, 0.5] {
            v.push(span(Shape::Cube, at(a, r - 0.01, y), at(a + da, 0.15, y - 0.4), 0.008, 4));
        }
    }
    v
}

// ---------------------------------------------------------------- filets, poteaux, tables

/// Filet de tennis de 12,8 m poteaux compris, 1,07 m aux poteaux, 0,914 m au centre.
fn tennis_net() -> Vec<Piece> {
    let (hw, edge, mid) = (6.4, 1.07, 0.914);
    let mut v = net(hw - 0.05, 0.0, (edge, mid), 0.4, 1);
    v.extend([
        cyl(-hw, 0.0, 0.08, 0.0, edge + 0.03, 0),
        cyl(hw, 0.0, 0.08, 0.0, edge + 0.03, 0),
        span(Shape::Cube, Vec3::new(-hw, edge, 0.0), Vec3::new(0.0, mid, 0.0), 0.05, 2),
        span(Shape::Cube, Vec3::new(hw, edge, 0.0), Vec3::new(0.0, mid, 0.0), 0.05, 2),
        bx(0.0, 0.0, 0.05, 0.015, 0.0, mid, 2),
    ]);
    v
}

/// Filet de volley de 10 m × 1 m, haut à 2,43 m, poteaux à ±5,5 m, antennes à ±4,5 m.
fn volley_net() -> Vec<Piece> {
    let (y0, y1) = (1.43, 2.43);
    let mut v = net(5.0, y0, (y1, y1), 0.4, 1);
    v.extend([bx(0.0, 0.0, 10.0, 0.02, y1 - 0.07, y1, 2), bx(0.0, 0.0, 10.0, 0.02, y0, y0 + 0.05, 2)]);
    for sx in [-1.0, 1.0] {
        v.extend([
            cyl(5.5 * sx, 0.0, 0.1, 0.0, 2.55, 0),
            bx(4.475 * sx, 0.0, 0.05, 0.022, y0, y1, 2),
            cyl(4.5 * sx, 0.0, 0.01, y0, y1 + 0.8, 3),
            bx(5.25 * sx, 0.0, 0.5, 0.008, y1 - 0.04, y1 - 0.03, 1),
            bx(5.25 * sx, 0.0, 0.5, 0.008, y0 + 0.02, y0 + 0.03, 1),
        ]);
    }
    v
}

/// H de rugby : poteaux de 11 m espacés de 5,6 m, barre à 3 m, protections au pied.
fn rugby_posts() -> Vec<Piece> {
    let x = 2.875;
    vec![
        cyl(-x, 0.0, 0.15, 0.0, 11.0, 0),
        cyl(x, 0.0, 0.15, 0.0, 11.0, 0),
        span(Shape::Cylinder, Vec3::new(-x, 3.0, 0.0), Vec3::new(x, 3.0, 0.0), 0.12, 0),
        bx(-x, 0.0, 0.5, 0.5, 0.0, 2.0, 1),
        bx(x, 0.0, 0.5, 0.5, 0.0, 2.0, 1),
    ]
}

/// Table de 2,74 × 1,525 m, plateau à 76 cm, filet de 15,25 cm le long de z.
fn ping_pong_table() -> Vec<Piece> {
    let (l, w, y) = (2.74, 1.525, 0.76);
    let line = |x: f32, z: f32, sx: f32, sz: f32| bx(x, z, sx, sz, y, y + 0.002, 1);
    let mut v = vec![
        bx(0.0, 0.0, l, w, y - 0.025, y, 0),
        line(0.0, w / 2.0 - 0.01, l, 0.02),
        line(0.0, -w / 2.0 + 0.01, l, 0.02),
        line(l / 2.0 - 0.01, 0.0, 0.02, w),
        line(-l / 2.0 + 0.01, 0.0, 0.02, w),
        line(0.0, 0.0, l, 0.005),
        bx(0.0, 0.0, 0.01, 1.83, y, y + 0.1525, 2),
        bx(0.0, 0.0, 0.012, 1.83, y + 0.135, y + 0.1525, 1),
        bx(0.0, -0.915, 0.04, 0.04, y - 0.03, y + 0.16, 3),
        bx(0.0, 0.915, 0.04, 0.04, y - 0.03, y + 0.16, 3),
    ];
    for x in [-0.95, 0.95] {
        v.extend([
            bx(x, -0.6, 0.05, 0.05, 0.0, y - 0.025, 3),
            bx(x, 0.6, 0.05, 0.05, 0.0, y - 0.025, 3),
            bx(x, 0.0, 0.04, 1.2, 0.25, 0.29, 3),
        ]);
    }
    v
}

// ---------------------------------------------------------------- abords du terrain

/// Cinq rangs de 80 cm qui montent de 40 cm vers l'arrière : repose-pieds devant, assise
/// derrière, joues en escalier, garde-corps en haut.
fn bleachers() -> Vec<Piece> {
    let (l, depth, rise) = (6.0, 0.8, 0.4);
    let mut v = Vec::new();
    for i in 0..5 {
        let (zc, y) = (1.6 - depth * i as f32, rise * i as f32);
        v.push(bx(0.0, zc + 0.15, l, 0.4, y, y + 0.04, 0));
        v.push(bx(0.0, zc - 0.22, l, 0.3, y + 0.42, y + 0.46, 0));
        v.extend([-2.95, 0.0, 2.95].map(|x| bx(x, zc, 0.06, depth, 0.0, y + 0.42, 1)));
    }
    v.extend([-2.95, 0.0, 2.95].map(|x| bx(x, -1.97, 0.06, 0.06, 0.0, 3.0, 2)));
    v.push(bx(0.0, -1.97, l, 0.05, 2.92, 3.0, 2));
    v.push(bx(0.0, -1.97, l, 0.04, 2.48, 2.52, 2));
    v
}

/// Abri de touche de 4 × 1,5 m vitré au fond et sur les côtés, six sièges baquets.
fn dugout() -> Vec<Piece> {
    let (hw, hd, h) = (2.0, 0.75, 2.2);
    let glass =
        |x: f32, z: f32, sx: f32, sz: f32| Piece::new(Shape::Glass, Vec3::new(x, 1.15, z), Vec3::new(sx, 1.9, sz), 0);
    let mut v = vec![
        glass(0.0, -hd, 2.0 * hw, 0.02),
        glass(-hw, 0.0, 0.02, 2.0 * hd),
        glass(hw, 0.0, 0.02, 2.0 * hd),
        bx(0.0, 0.0, 2.0 * hw + 0.1, 2.0 * hd + 0.1, h - 0.05, h, 2),
        bx(0.0, -hd, 2.0 * hw, 0.06, 0.12, 0.2, 0),
        bx(0.0, -0.4, 2.0 * hw - 0.3, 0.1, 0.0, 0.4, 0),
    ];
    for (x, z) in [(-hw, -hd), (hw, -hd), (-hw, hd), (hw, hd)] {
        v.push(bx(x, z, 0.06, 0.06, 0.0, h - 0.05, 0));
    }
    for i in 0..6 {
        let x = -1.5 + 0.6 * i as f32;
        v.push(bx(x, -0.2, 0.48, 0.45, 0.4, 0.47, 1));
        v.push(bx(x, -0.44, 0.48, 0.06, 0.47, 0.95, 1));
    }
    v
}

/// Mât de 20 m et rampe de six projecteurs inclinés vers l'avant et le bas.
fn floodlight() -> Vec<Piece> {
    let top = 20.0;
    let tilt = Quat::from_rotation_x(0.6);
    let mut v = vec![
        bx(0.0, 0.0, 1.0, 1.0, 0.0, 0.4, 0),
        cyl(0.0, 0.0, 0.5, 0.4, 8.0, 0),
        cyl(0.0, 0.0, 0.35, 8.0, top, 0),
        bx(-1.9, 0.3, 0.1, 0.1, top - 0.6, top + 1.4, 1),
        bx(1.9, 0.3, 0.1, 0.1, top - 0.6, top + 1.4, 1),
        // la lumière (portée 15 m au rendu) est placée plus bas, devant, pour atteindre le sol
        light([0.0, 12.0, 5.0], 4),
    ];
    for y in [top - 0.55, top + 0.4, top + 1.35] {
        v.push(bx(0.0, 0.3, 3.8, 0.12, y - 0.05, y + 0.05, 1));
    }
    for (x, y) in [(-1.2, 0.0), (0.0, 0.0), (1.2, 0.0), (-1.2, 0.95), (0.0, 0.95), (1.2, 0.95)] {
        let at = Vec3::new(x, top - 0.1 + y, 0.6);
        v.push(Piece::new(Shape::Cube, at, Vec3::new(0.7, 0.7, 0.3), 2).turn(tilt));
        v.push(Piece::new(Shape::Cube, at + tilt * Vec3::Z * 0.155, Vec3::new(0.6, 0.6, 0.01), 3).turn(tilt));
    }
    v
}

/// Tableau de 4 × 2 m sur deux pieds, score « 2 - 1 » en segments lumineux, noms des équipes.
fn scoreboard() -> Vec<Piece> {
    let f = 0.125;
    let mut v = vec![
        bx(-1.6, -0.1, 0.2, 0.2, 0.0, 3.0, 0),
        bx(1.6, -0.1, 0.2, 0.2, 0.0, 3.0, 0),
        bx(0.0, -0.05, 4.2, 0.3, 2.9, 5.1, 1),
        bx(0.0, 0.11, 4.0, 0.02, 3.0, 5.0, 2),
        bx(0.0, f, 0.3, 0.01, 3.87, 3.93, 3),
        bx(-1.0, f, 1.2, 0.01, 4.6, 4.75, 4),
        bx(1.0, f, 1.2, 0.01, 4.6, 4.75, 4),
    ];
    v.extend(digit(-1.0, 3.9, f, &[0, 1, 6, 4, 3]));
    v.extend(digit(1.0, 3.9, f, &[1, 2]));
    v
}

/// Chiffre d'afficheur haut de 1 m centré en (x, y), dans le plan z ; `segs` parmi a…g (0…6).
fn digit(x: f32, y: f32, z: f32, segs: &[usize]) -> Vec<Piece> {
    // (dx, dy, horizontal) des segments a (haut), b, c (droite), d (bas), e, f (gauche), g (milieu)
    const SEGS: [(f32, f32, bool); 7] = [
        (0.0, 0.5, true),
        (0.25, 0.25, false),
        (0.25, -0.25, false),
        (0.0, -0.5, true),
        (-0.25, -0.25, false),
        (-0.25, 0.25, false),
        (0.0, 0.0, true),
    ];
    segs.iter()
        .filter_map(|&i| SEGS.get(i))
        .map(|&(dx, dy, flat)| {
            let size = if flat { Vec3::new(0.42, 0.07, 0.01) } else { Vec3::new(0.07, 0.44, 0.01) };
            Piece::new(Shape::Cube, Vec3::new(x + dx, y + dy, z), size, 3)
        })
        .collect()
}

// ---------------------------------------------------------------- salle de sport

/// Banc plat, chandelles à l'arrière et barre olympique de 2,2 m chargée de deux disques par côté.
fn weight_bench() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 0.05, 0.3, 1.2, 0.38, 0.45, 0),
        bx(0.0, 0.05, 0.08, 1.1, 0.3, 0.38, 1),
        bx(0.0, 0.55, 0.5, 0.06, 0.0, 0.04, 1),
        bx(0.0, 0.55, 0.06, 0.06, 0.0, 0.3, 1),
        bx(0.0, -0.55, 1.2, 0.08, 0.0, 0.05, 1),
        bx(0.0, -0.45, 0.06, 0.06, 0.0, 0.3, 1),
        Piece::new(Shape::Wheel, Vec3::new(0.0, 1.095, -0.5), Vec3::new(2.2, 0.03, 0.03), 2),
    ];
    for x in [-0.55, 0.55] {
        v.push(bx(x, -0.55, 0.06, 0.06, 0.0, 1.25, 1));
        v.push(bx(x, -0.5, 0.06, 0.08, 1.05, 1.08, 1));
    }
    for x in [-0.86, -0.8, 0.8, 0.86] {
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 1.095, -0.5), Vec3::new(0.05, 0.45, 0.45), 3));
    }
    v
}

/// Tapis de 0,8 × 1,9 m, capot moteur et console inclinée à l'arrière, mains courantes.
fn treadmill() -> Vec<Piece> {
    let tilt = Quat::from_rotation_x(0.6);
    let console = Vec3::new(0.0, 1.3, -0.68);
    let mut v = vec![
        bx(0.0, 0.1, 0.8, 1.7, 0.0, 0.16, 0),
        bx(0.0, 0.15, 0.5, 1.5, 0.16, 0.18, 1),
        bx(0.0, -0.8, 0.8, 0.3, 0.0, 0.3, 0),
        Piece::new(Shape::Cube, console, Vec3::new(0.8, 0.08, 0.3), 2).turn(tilt),
        Piece::new(Shape::Cube, console + tilt * Vec3::Y * 0.045, Vec3::new(0.4, 0.01, 0.2), 3).turn(tilt),
    ];
    for x in [-0.36, 0.36] {
        v.extend([
            bx(x * 0.92, 0.15, 0.14, 1.5, 0.16, 0.2, 0),
            span(Shape::Cube, Vec3::new(x, 0.3, -0.8), Vec3::new(x, 1.27, -0.7), 0.07, 0),
            span(Shape::Cylinder, Vec3::new(x, 1.1, -0.72), Vec3::new(x, 1.05, -0.25), 0.04, 0),
        ]);
    }
    v
}

/// Sac de 1,2 m pendu à une potence : mât et pied à l'arrière, trois chaînes.
fn punching_bag() -> Vec<Piece> {
    let (zb, zm) = (0.3, -0.45);
    let mut v = vec![
        cyl(0.0, zb, 0.36, 0.45, 1.65, 0),
        cyl(0.0, zb, 0.365, 1.55, 1.62, 1),
        cyl(0.0, zb, 0.365, 0.5, 0.57, 1),
        bx(0.0, zm, 0.08, 0.08, 0.0, 2.3, 2),
        bx(0.0, f32::midpoint(zm, zb) + 0.02, 0.08, zb - zm + 0.12, 2.22, 2.3, 2),
        bx(0.0, zm, 1.0, 0.08, 0.0, 0.06, 2),
        bx(0.0, zm, 0.08, 0.6, 0.0, 0.06, 2),
        span(Shape::Cylinder, Vec3::new(0.0, 2.05, zb), Vec3::new(0.0, 2.22, zb), 0.015, 3),
    ];
    for a in [0.0, TAU / 3.0, 2.0 * TAU / 3.0] {
        let rim = Vec3::new(0.15 * a.cos(), 1.65, zb - 0.15 * a.sin());
        v.push(span(Shape::Cylinder, rim, Vec3::new(0.0, 2.05, zb), 0.012, 3));
    }
    v
}

/// Haie de 1,18 m, barre rayée à 1,067 m, pieds vers l'avant.
fn hurdle() -> Vec<Piece> {
    let (h, z) = (1.067, -0.3);
    let mut v = vec![bx(0.0, z, 1.18, 0.02, h - 0.07, h, 0)];
    v.extend([-0.4, 0.0, 0.4].map(|x| bx(x, z, 0.2, 0.024, h - 0.072, h + 0.002, 1)));
    for x in [-0.6, 0.6] {
        v.push(bx(x, z, 0.03, 0.03, 0.03, h - 0.07, 2));
        v.push(bx(x, 0.05, 0.04, 0.7, 0.0, 0.03, 3));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_follows_its_circle_and_refines_with_the_radius() {
        let a = arc(Vec3::Y, 2.0, (0.0, FRAC_PI_2), (0.1, 0.01), 0);
        for p in &a {
            let radial = p.at - Vec3::Y;
            assert!((radial.length() - 2.0).abs() < 1e-4, "{p:?} sur le cercle");
            assert!((p.rot * Vec3::X).dot(radial).abs() < 1e-4, "{p:?} tangent au cercle");
        }
        let (first, last) = (a[0].at, a[a.len() - 1].at);
        assert!(first.x > 1.9 && last.z < -1.9, "de +x vers -z : {first} {last}");
        let count = |r: f32| arc(Vec3::ZERO, r, (0.0, TAU), (0.1, 0.01), 0).len();
        assert!(count(0.2) >= 24 && count(9.15) > 100 && count(150.0) <= 700);
    }
    #[test]
    fn goals_have_regulation_openings() {
        let s = PropKind::SoccerGoal.size();
        assert!((s.x - 7.56).abs() < 0.01 && (s.z - 2.56).abs() < 0.01 && (s.y - 2.08).abs() < 0.01, "{s}");
        let h = PropKind::HandballGoal.size();
        assert!((h.x - 3.164).abs() < 0.01 && (h.z - 2.082).abs() < 0.01, "{h}");
    }
    #[test]
    fn hoop_ring_is_at_regulation_height_and_floodlight_shines() {
        let ring = gear(PropKind::BasketHoop).into_iter().filter(|p| p.slot == 3).fold(0.0_f32, |m, p| m.max(p.at.y));
        assert!((ring - 3.05).abs() < 0.02, "cercle à {ring}");
        assert!(gear(PropKind::Floodlight).iter().any(|p| p.shape == Shape::Light));
        assert!(PropKind::Floodlight.size().z > 20.0);
    }
    #[test]
    fn digits_light_their_segments_only() {
        assert_eq!(digit(0.0, 0.0, 0.0, &[1, 2]).len(), 2);
        assert!(digit(0.0, 0.0, 0.0, &[7]).is_empty(), "segment inconnu ignoré");
    }
}
