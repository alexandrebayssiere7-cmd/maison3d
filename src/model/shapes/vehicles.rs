//! Véhicules. Seule la voiture (`Car`) se conduit pendant la visite ; les autres sont des décors
//! dont les vitres sont des pans teintés (à peindre comme les autres parties), sauf celles du bus.

use bevy::math::Vec3;

use super::{Piece, ball, bx, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const TYRES: Slot = Slot::plain("Pneus", [0.08, 0.08, 0.09]);
const HEADLIGHTS: Slot = Slot::plain("Phares", [1.0, 0.97, 0.85]);
const TAILLIGHTS: Slot = Slot::plain("Feux arrière", [0.8, 0.05, 0.05]);
const TINTED: Slot = Slot::plain("Vitres", [0.12, 0.14, 0.18]);

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Car => {
            const {
                &[
                    Slot::plain("Carrosserie", [0.72, 0.08, 0.1]),
                    Slot::plain("Tableau de bord et pare-chocs", DARK),
                    Slot::plain("Sièges", [0.25, 0.25, 0.28]),
                    Slot::plain("Phares", [1.0, 0.97, 0.85]),
                    Slot::plain("Feux arrière", [0.8, 0.05, 0.05]),
                    Slot::plain("Plaques", WHITE),
                    Slot::plain("Pneus", [0.08, 0.08, 0.09]),
                ]
            }
        }
        PropKind::Motorcycle => {
            const {
                &[
                    Slot::plain("Carrosserie", [0.12, 0.2, 0.45]),
                    Slot::plain("Selle", [0.08, 0.08, 0.09]),
                    Slot::plain("Moteur et cadre", [0.3, 0.3, 0.32]),
                    Slot::textured("Chromes", "Chrome"),
                    Slot::plain("Phare", [1.0, 0.97, 0.85]),
                    Slot::plain("Feu arrière", [0.8, 0.05, 0.05]),
                    TYRES,
                ]
            }
        }
        PropKind::Bicycle => {
            const {
                &[
                    Slot::plain("Cadre", [0.1, 0.45, 0.6]),
                    Slot::plain("Selle et poignées", [0.1, 0.1, 0.1]),
                    Slot::plain("Jantes et pédalier", METAL),
                    TYRES,
                ]
            }
        }
        PropKind::Van => {
            const {
                &[
                    Slot::plain("Carrosserie", WHITE),
                    TINTED,
                    Slot::plain("Pare-chocs", DARK),
                    HEADLIGHTS,
                    TAILLIGHTS,
                    Slot::plain("Plaques", WHITE),
                    TYRES,
                ]
            }
        }
        PropKind::Bus => {
            const {
                &[
                    Slot::plain("Carrosserie", [0.1, 0.45, 0.7]),
                    Slot::plain("Sièges", [0.2, 0.25, 0.5]),
                    Slot::plain("Montants et pare-chocs", DARK),
                    Slot::plain("Portes", [0.3, 0.36, 0.4]),
                    HEADLIGHTS,
                    TAILLIGHTS,
                    Slot::plain("Girouette", [1.0, 0.65, 0.1]),
                    TYRES,
                ]
            }
        }
        PropKind::Truck => {
            const {
                &[
                    Slot::plain("Cabine", [0.15, 0.3, 0.6]),
                    Slot::plain("Caisse", WHITE),
                    TINTED,
                    Slot::plain("Châssis et pare-chocs", DARK),
                    HEADLIGHTS,
                    TAILLIGHTS,
                    TYRES,
                ]
            }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, _height: f32) -> Vec<Piece> {
    match kind {
        PropKind::Car => car(),
        PropKind::Motorcycle => motorcycle(),
        PropKind::Bicycle => bicycle(),
        PropKind::Van => van(),
        PropKind::Bus => bus(),
        PropKind::Truck => truck(),
        _ => Vec::new(),
    }
}

/// Berline de 4,2 m, l'avant vers +z ; habitacle vitré pour conduire en vue intérieure.
fn car() -> Vec<Piece> {
    let glass =
        |x: f32, z: f32, sx: f32, sz: f32| Piece::new(Shape::Glass, Vec3::new(x, 1.2, z), Vec3::new(sx, 0.5, sz), 0);
    let mut v = vec![bx(0.0, 0.0, 1.8, 4.2, 0.3, 0.95, 0), bx(0.0, -0.15, 1.72, 2.1, 1.45, 1.52, 0)];
    for (x, z) in [(-0.84, 0.88), (0.84, 0.88), (-0.84, -1.18), (0.84, -1.18), (-0.84, -0.15), (0.84, -0.15)] {
        v.push(bx(x, z, 0.06, 0.08, 0.95, 1.45, 0));
    }
    v.push(glass(0.0, 0.88, 1.62, 0.03));
    v.push(glass(0.0, -1.18, 1.62, 0.03));
    v.push(glass(-0.85, -0.15, 0.03, 2.0));
    v.push(glass(0.85, -0.15, 0.03, 2.0));
    // intérieur : planche de bord, volant (place conducteur à gauche, x > 0), sièges
    v.push(bx(0.0, 0.72, 1.62, 0.3, 0.95, 1.05, 1));
    v.push(bx(0.4, 0.5, 0.36, 0.05, 1.0, 1.28, 1));
    for x in [-0.4, 0.4] {
        v.push(bx(x, -0.45, 0.5, 0.12, 0.95, 1.35, 2));
    }
    v.push(bx(0.0, -0.95, 1.5, 0.12, 0.95, 1.3, 2));
    // phares, feux, pare-chocs, plaques
    for x in [-0.6, 0.6] {
        v.push(bx(x, 2.1, 0.35, 0.03, 0.62, 0.76, 3));
        v.push(bx(x, -2.1, 0.35, 0.03, 0.66, 0.78, 4));
    }
    v.push(bx(0.0, 2.13, 1.82, 0.08, 0.3, 0.46, 1));
    v.push(bx(0.0, -2.13, 1.82, 0.08, 0.3, 0.46, 1));
    v.push(bx(0.0, 2.15, 0.5, 0.02, 0.47, 0.58, 5));
    v.push(bx(0.0, -2.15, 0.5, 0.02, 0.5, 0.61, 5));
    for (x, z) in [(-0.82, 1.35), (0.82, 1.35), (-0.82, -1.35), (0.82, -1.35)] {
        v.push(Piece::new(Shape::Wheel, Vec3::new(x, 0.34, z), Vec3::new(0.24, 0.68, 0.68), 6));
    }
    v
}

/// Roues de diamètre `d` et de largeur `w` posées au sol aux points (x, z).
fn wheels(at: &[(f32, f32)], (w, d): (f32, f32), slot: usize) -> Vec<Piece> {
    at.iter().map(|&(x, z)| Piece::new(Shape::Wheel, Vec3::new(x, d / 2.0, z), Vec3::new(w, d, d), slot)).collect()
}

/// Pan incliné (nez, pare-brise) large de `w` selon x : sa face avant suit la droite de `a` (en
/// bas, devant) à `b` (en haut, derrière) et son épaisseur `t` part vers l'arrière.
fn slab(a: Vec3, b: Vec3, (w, t): (f32, f32), slot: usize) -> Piece {
    let p = span(Shape::Cube, a, b, t, slot);
    Piece { at: p.at - p.rot * Vec3::Z * (t / 2.0), size: Vec3::new(w, p.size.y, t), ..p }
}

/// Moto routière de 2,1 m, l'avant vers +z.
fn motorcycle() -> Vec<Piece> {
    let v3 = Vec3::new;
    let head = v3(0.0, 0.92, 0.5);
    let mut v = wheels(&[(0.0, 0.72)], (0.12, 0.62), 6);
    v.extend(wheels(&[(0.0, -0.72)], (0.16, 0.64), 6));
    // fourche et guidon chromés, phare rond devant la colonne de direction
    for x in [-0.07, 0.07] {
        v.push(span(Shape::Cylinder, v3(x, 0.31, 0.72), v3(x, 0.92, 0.5), 0.04, 3));
        v.push(span(Shape::Cylinder, v3(x * 1.2, 0.36, -0.1), v3(x * 1.2, 0.31, -0.72), 0.05, 2));
    }
    v.push(span(Shape::Cylinder, head, v3(0.0, 1.02, 0.42), 0.06, 2));
    v.push(span(Shape::Cylinder, v3(-0.36, 1.02, 0.42), v3(0.36, 1.02, 0.42), 0.03, 3));
    v.push(ball([0.0, 0.88, 0.6], [0.17, 0.17, 0.08], 4));
    // cadre, moteur et échappement
    v.push(span(Shape::Cylinder, head, v3(0.0, 0.45, -0.1), 0.06, 2));
    v.push(span(Shape::Cylinder, head, v3(0.0, 0.8, -0.5), 0.06, 2));
    v.push(bx(0.0, 0.02, 0.3, 0.45, 0.28, 0.66, 2));
    v.push(span(Shape::Cylinder, v3(0.17, 0.35, 0.1), v3(0.2, 0.5, -0.85), 0.09, 3));
    // garde-boue, réservoir, selle, coque arrière et feu
    v.push(ball([0.0, 0.66, 0.72], [0.16, 0.06, 0.5], 0));
    v.push(ball([0.0, 0.88, 0.18], [0.34, 0.26, 0.55], 0));
    v.push(bx(0.0, -0.32, 0.3, 0.6, 0.8, 0.88, 1));
    v.push(bx(0.0, -0.7, 0.22, 0.35, 0.74, 0.86, 0));
    v.push(bx(0.0, -0.89, 0.12, 0.03, 0.78, 0.84, 5));
    v
}

/// Vélo de ville de 1,75 m, l'avant vers +z ; cadre en tubes, roues fines.
fn bicycle() -> Vec<Piece> {
    let v3 = Vec3::new;
    let (rear, front, crank) = (v3(0.0, 0.34, -0.52), v3(0.0, 0.34, 0.52), v3(0.0, 0.3, -0.06));
    let (seat, top, low) = (v3(0.0, 0.82, -0.24), v3(0.0, 0.88, 0.36), v3(0.0, 0.72, 0.41));
    let tube = |a: Vec3, b: Vec3| span(Shape::Cylinder, a, b, 0.035, 0);
    // pneus fins autour de jantes pleines un peu plus larges, qui dépassent de chaque côté
    let mut v = wheels(&[(0.0, -0.52), (0.0, 0.52)], (0.035, 0.68), 3);
    v.extend(wheels(&[(0.0, -0.52), (0.0, 0.52)], (0.04, 0.58), 2));
    v.extend([tube(seat, top), tube(crank, low), tube(crank, seat), tube(crank, rear), tube(seat, rear)]);
    v.push(span(Shape::Cylinder, low, top, 0.05, 0));
    v.push(span(Shape::Cylinder, low, front, 0.03, 0));
    // selle, potence et guidon à poignées
    v.push(span(Shape::Cylinder, seat, v3(0.0, 0.95, -0.29), 0.025, 2));
    v.push(bx(0.0, -0.3, 0.14, 0.26, 0.95, 1.0, 1));
    v.push(span(Shape::Cylinder, top, v3(0.0, 0.98, 0.32), 0.03, 2));
    v.push(span(Shape::Cylinder, v3(-0.24, 0.98, 0.32), v3(0.24, 0.98, 0.32), 0.025, 2));
    for x in [-1.0, 1.0] {
        v.push(span(Shape::Cylinder, v3(0.19 * x, 0.98, 0.32), v3(0.26 * x, 0.98, 0.32), 0.035, 1));
        // manivelles opposées, pédale au bout
        let end = crank + v3(0.07 * x, -0.14 * x, 0.1 * x);
        v.push(span(Shape::Cylinder, crank + v3(0.06 * x, 0.0, 0.0), end, 0.025, 2));
        v.push(bx(end.x + 0.04 * x, end.z, 0.09, 0.05, end.y - 0.01, end.y + 0.01, 1));
    }
    v.push(Piece::new(Shape::Wheel, crank + v3(0.05, 0.0, 0.0), v3(0.01, 0.2, 0.2), 2));
    v
}

/// Camionnette utilitaire de 5,3 m, l'avant vers +z : nez incliné, caisse aveugle, portes arrière vitrées.
fn van() -> Vec<Piece> {
    let v3 = Vec3::new;
    let (a, b) = (v3(0.0, 1.15, 2.05), v3(0.0, 2.38, 1.4));
    let out = Vec3::X.cross(b - a).normalize() * 0.015;
    let mut v = vec![
        bx(0.0, -0.6, 2.0, 4.0, 0.42, 2.4, 0),
        bx(0.0, 1.95, 2.0, 1.1, 0.42, 1.15, 0),
        slab(a, b, (2.0, 0.6), 0),
        slab(a.lerp(b, 0.08) + out, a.lerp(b, 0.92) + out, (1.8, 0.02), 1),
    ];
    for x in [-1.0, 1.0] {
        v.push(bx(x * 1.005, 1.0, 0.02, 0.75, 1.3, 2.1, 1));
        v.push(bx(x * 0.45, -2.605, 0.8, 0.02, 1.5, 2.15, 1));
        v.push(bx(x * 1.1, 1.35, 0.18, 0.06, 1.55, 1.75, 2));
        v.push(bx(x * 0.72, 2.51, 0.36, 0.03, 0.85, 1.02, 3));
        v.push(bx(x * 0.92, -2.615, 0.12, 0.03, 0.9, 1.6, 4));
    }
    v.extend([
        bx(0.0, 2.53, 2.02, 0.12, 0.35, 0.6, 2),
        bx(0.0, -2.63, 2.02, 0.1, 0.35, 0.6, 2),
        bx(0.0, 2.505, 1.0, 0.02, 0.7, 1.0, 2),
        bx(0.0, 2.6, 0.5, 0.02, 0.42, 0.53, 5),
        bx(0.0, -2.69, 0.5, 0.02, 0.4, 0.51, 5),
    ]);
    v.extend(wheels(&[(-0.9, 1.75), (0.9, 1.75), (-0.9, -1.55), (0.9, -1.55)], (0.24, 0.7), 6));
    v
}

/// Bus urbain de 12 m, l'avant vers +z : bandeau vitré continu (verre), portes du côté droit
/// (-x), sièges visibles à travers les vitres, girouette au-dessus du pare-brise.
fn bus() -> Vec<Piece> {
    let glass = |at: Vec3, size: Vec3| Piece::new(Shape::Glass, at, size, 0);
    let mut v = vec![
        bx(0.0, 0.0, 2.55, 12.0, 0.3, 1.1, 0),
        bx(0.0, 0.0, 2.55, 12.0, 2.7, 3.05, 0),
        bx(0.0, -5.95, 2.55, 0.1, 1.1, 2.7, 0),
        glass(Vec3::new(-1.265, 1.9, -0.05), Vec3::new(0.02, 1.6, 11.8)),
        glass(Vec3::new(1.265, 1.9, -0.05), Vec3::new(0.02, 1.6, 11.8)),
        glass(Vec3::new(0.0, 1.9, 5.985), Vec3::new(2.48, 1.6, 0.03)),
    ];
    for i in 0..9 {
        let z = -5.9 + 1.475 * i as f32;
        v.push(bx(-1.27, z, 0.03, 0.12, 1.1, 2.7, 2));
        v.push(bx(1.27, z, 0.03, 0.12, 1.1, 2.7, 2));
    }
    v.extend([bx(-1.285, 5.2, 0.03, 1.2, 0.35, 2.65, 3), bx(-1.285, 0.2, 0.03, 1.2, 0.35, 2.65, 3)]);
    // rangées de sièges : dossiers et banquettes de part et d'autre de l'allée, sauf face à la porte du milieu
    for x in [-0.75, 0.75] {
        for z in [-5.3, -4.45, -3.6, -2.75, -1.9, 1.3, 2.15, 3.0] {
            v.push(bx(x, z, 0.9, 0.1, 1.1, 1.85, 1));
        }
        v.push(bx(x, -3.4, 0.9, 4.2, 1.1, 1.5, 1));
        v.push(bx(x, 2.4, 0.9, 1.9, 1.1, 1.5, 1));
    }
    v.extend([bx(0.75, 4.95, 0.5, 0.1, 1.1, 1.9, 1), bx(0.0, 5.75, 2.4, 0.3, 1.1, 1.4, 2)]);
    // girouette, phares, pare-chocs, feux, rétroviseurs
    v.push(bx(0.0, 6.0, 1.8, 0.03, 2.72, 2.98, 6));
    for x in [-1.0, 1.0] {
        v.push(bx(x * 0.95, 6.01, 0.35, 0.03, 0.55, 0.75, 4));
        v.push(bx(x, -6.01, 0.2, 0.03, 0.8, 1.3, 5));
        v.push(span(Shape::Cylinder, Vec3::new(x * 1.27, 2.4, 5.9), Vec3::new(x * 1.45, 2.25, 6.12), 0.04, 2));
        v.push(bx(x * 1.45, 6.12, 0.08, 0.05, 1.8, 2.25, 2));
    }
    v.extend([bx(0.0, 6.03, 2.57, 0.1, 0.3, 0.5, 2), bx(0.0, -6.03, 2.57, 0.1, 0.3, 0.5, 2)]);
    v.extend(wheels(&[(-1.15, 3.4), (1.15, 3.4), (-1.15, -2.8), (1.15, -2.8)], (0.3, 1.0), 7));
    v
}

/// Camion porteur de 8,5 m, l'avant vers +z : cabine avancée, caisse fourgon, deux essieux arrière.
fn truck() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 3.15, 2.45, 2.1, 1.0, 3.0, 0),
        bx(0.0, 3.875, 2.45, 0.65, 0.6, 1.0, 0),
        bx(0.0, -1.1, 2.5, 6.2, 1.1, 3.8, 1),
        bx(0.0, -0.9, 1.0, 6.6, 0.6, 1.1, 3),
        bx(0.0, 4.205, 2.2, 0.02, 1.85, 2.75, 2),
        bx(0.0, 4.21, 1.6, 0.02, 0.9, 1.6, 3),
        bx(0.0, 4.25, 2.47, 0.12, 0.4, 0.75, 3),
        bx(0.0, -4.205, 0.03, 0.02, 1.15, 3.75, 3),
        bx(0.0, -4.15, 2.3, 0.1, 0.45, 0.6, 3),
    ];
    for x in [-1.0, 1.0] {
        v.push(bx(x * 1.23, 3.55, 0.02, 0.9, 1.85, 2.7, 2));
        v.push(bx(x * 0.95, 4.215, 0.35, 0.03, 0.8, 1.0, 4));
        v.push(bx(x * 1.0, -4.215, 0.25, 0.03, 1.2, 1.4, 5));
        v.push(bx(x * 1.1, 2.3, 0.3, 0.35, 0.45, 0.55, 3));
        v.push(span(Shape::Cylinder, Vec3::new(x * 1.22, 2.6, 3.9), Vec3::new(x * 1.42, 2.5, 3.95), 0.04, 3));
        v.push(bx(x * 1.42, 3.95, 0.06, 0.1, 1.9, 2.5, 3));
    }
    v.extend(wheels(
        &[(-1.08, 3.0), (1.08, 3.0), (-1.08, -2.0), (1.08, -2.0), (-1.08, -3.1), (1.08, -3.1)],
        (0.3, 1.0),
        6,
    ));
    v
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::{Category, Prop};

    #[test]
    fn vehicles_roll_on_their_wheels() {
        for k in Category::Vehicles.kinds() {
            let pieces = shapes(k, 0.0, 3.0);
            let tyres: Vec<_> = pieces.iter().filter(|p| p.shape == Shape::Wheel && p.size.y > 0.5).collect();
            assert!(tyres.len() >= 2, "{k:?} sans roues");
            assert!(tyres.iter().all(|p| (p.at.y - p.size.y / 2.0).abs() < 1e-4), "{k:?} : roue en l'air");
            let (lo, hi) = Prop::new(k, Vec2::ZERO, 0, 0.0).local_box();
            assert!(lo.y > -0.03, "{k:?} s'enfonce : {lo}");
            let (c, s) = ((lo + hi) / 2.0, hi - lo);
            assert!(c.x.abs() < 0.05 * s.x && c.z.abs() < 0.05 * s.z, "{k:?} décentré : {c}");
        }
    }
    #[test]
    fn sizes_are_realistic() {
        assert!((PropKind::Bus.size().y - 12.0).abs() < 0.3, "bus de 12 m");
        assert!(PropKind::Truck.size().z > PropKind::Van.size().z && PropKind::Van.size().z > PropKind::Car.size().z);
        let thin =
            shapes(PropKind::Bicycle, 0.0, 3.0).iter().filter(|p| p.shape == Shape::Wheel).all(|p| p.size.x < 0.05);
        assert!(thin, "roues de vélo fines");
    }
    #[test]
    fn slab_front_face_follows_its_line() {
        let (a, b) = (Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 2.0, 0.0));
        let p = slab(a, b, (2.0, 0.4), 0);
        let front = p.rot * Vec3::Z;
        assert!(front.z > 0.0 && front.y > 0.0, "la face avant regarde devant et vers le haut");
        assert!((p.at + front * 0.2).abs_diff_eq((a + b) / 2.0, 1e-5));
        assert!((p.size - Vec3::new(2.0, 2.0_f32.sqrt(), 0.4)).abs().max_element() < 1e-5);
    }
}
