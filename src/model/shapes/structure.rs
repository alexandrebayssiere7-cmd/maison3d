//! Escaliers (marches d'environ 20 cm), rampe, piliers et autres éléments de structure, à la
//! hauteur demandée. La visite ne fait monter que des marches de moins de 45 cm et heurte une
//! pièce penchée comme sa boîte englobante droite : tout ce qui se gravit est fait de boîtes droites
//! (ou de dalles penchées assez courtes pour que leur boîte reste une petite marche), les pièces
//! penchées longues (limons, verre, mains courantes) restent sur les côtés du passage.

use std::f32::consts::PI;

use bevy::math::{Quat, Vec3};

use super::{Piece, ball, bx, cyl, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot};

const STEPS: [f32; 3] = [0.6, 0.45, 0.3];
/// Giron des escaliers d'intérieur.
const RUN: f32 = 0.28;

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Stairs | PropKind::LStairs | PropKind::UStairs => const { &[Slot::plain("Marches", STEPS)] },
        PropKind::SpiralStairs => {
            const { &[Slot::plain("Marches", [0.35, 0.36, 0.38]), Slot::plain("Mât", [0.35, 0.36, 0.38])] }
        }
        PropKind::Ladder => {
            const { &[Slot::textured("Marches", "Bois clair"), Slot::textured("Limons", "Bois foncé")] }
        }
        PropKind::FloatingStairs => {
            const { &[Slot::textured("Marches", "Bois foncé"), Slot::plain("Limon", [0.92, 0.92, 0.9])] }
        }
        PropKind::StoneSteps => {
            const { &[Slot::textured("Marches", "Pierre de taille"), Slot::textured("Murets", "Moellons")] }
        }
        PropKind::Escalator => {
            const {
                &[
                    Slot::plain("Marches", [0.32, 0.33, 0.35]),
                    Slot::plain("Bandes de sécurité", [0.95, 0.75, 0.1]),
                    Slot::textured("Habillage", "Inox brossé"),
                    Slot::plain("Mains courantes", DARK),
                ]
            }
        }
        PropKind::Ramp => {
            const {
                &[
                    Slot::textured("Rampe", "Béton brut"),
                    Slot::plain("Poteaux et lisses", METAL),
                    Slot::textured("Mains courantes", "Inox brossé"),
                ]
            }
        }
        PropKind::Pillar => const { &[Slot::plain("Pilier", [0.85, 0.85, 0.8])] },
        PropKind::Column => {
            const { &[Slot::textured("Fût", "Marbre blanc"), Slot::plain("Base et chapiteau", [0.88, 0.86, 0.8])] }
        }
        PropKind::Railing => {
            const { &[Slot::plain("Barreaux", [0.16, 0.16, 0.18]), Slot::textured("Main courante", "Bois clair")] }
        }
        PropKind::Balustrade => {
            const { &[Slot::textured("Socle et tablette", "Pierre de taille"), Slot::plain("Balustres", [0.86, 0.83, 0.76])] }
        }
        PropKind::Arch => {
            const { &[Slot::textured("Pierre", "Pierre de taille"), Slot::plain("Impostes et clé", [0.8, 0.76, 0.68])] }
        }
        _ => const { &[] },
    }
}

pub(super) fn shapes(kind: PropKind, _open: f32, height: f32) -> Vec<Piece> {
    let n = ((height / 0.2).round() as usize).max(1);
    let rise = height / n as f32;
    match kind {
        PropKind::Stairs => flight(n, rise, RUN, (0.0, -(n as f32) * RUN / 2.0), 1.0).collect(),
        PropKind::SpiralStairs => spiral(n, rise, height),
        PropKind::LStairs => quarter_turn(n, rise),
        PropKind::UStairs => half_turn(n, rise),
        PropKind::Ladder => ladder(height),
        PropKind::FloatingStairs => floating(n, rise),
        PropKind::StoneSteps => stone_steps(height),
        PropKind::Escalator => escalator(n, rise),
        PropKind::Ramp => ramp(height),
        PropKind::Pillar => vec![bx(0.0, 0.0, 0.3, 0.3, 0.0, height, 0)],
        PropKind::Column => column(height),
        PropKind::Railing => railing(),
        PropKind::Balustrade => balustrade(),
        PropKind::Arch => arch(),
        _ => Vec::new(),
    }
}

/// `n` marches pleines de giron `run` et de largeur `w`, centrées en `x`, qui montent vers +z
/// depuis `z0` ; la première monte de `rise`.
fn flight(n: usize, rise: f32, run: f32, (x, z0): (f32, f32), w: f32) -> impl Iterator<Item = Piece> {
    (0..n).map(move |i| bx(x, z0 + (i as f32 + 0.5) * run, w, run, 0.0, (i + 1) as f32 * rise, 0))
}

/// Planche de section `(w, d)` (largeur en x, épaisseur dans le plan vertical) tendue de `a` à `b`.
fn board(shape: Shape, a: Vec3, b: Vec3, (w, d): (f32, f32), slot: usize) -> Piece {
    let p = span(shape, a, b, w, slot);
    Piece { size: Vec3::new(w, p.size.y, d), ..p }
}

/// Limon de section `(w, d)` en `x`, le long d'une volée qui monte vers +z : son axe suit
/// `y = (z - z0) * slope + lift`, du sol (coin bas posé dessus) jusqu'à la hauteur `top`.
fn stringer(x: f32, (z0, slope, lift): (f32, f32, f32), top: f32, (w, d): (f32, f32), slot: usize) -> Piece {
    let y0 = d / 2.0 / slope.hypot(1.0);
    let at = |y: f32| Vec3::new(x, y, z0 + (y - lift) / slope);
    board(Shape::Cube, at(y0), at(top), (w, d), slot)
}

fn spiral(n: usize, rise: f32, height: f32) -> Vec<Piece> {
    let mut v = vec![cyl(0.0, 0.0, 0.16, 0.0, height + 1.0, 1)];
    for i in 0..n {
        let a = i as f32 * 0.5;
        let top = (i + 1) as f32 * rise;
        let dir = Vec3::new(a.cos(), 0.0, -a.sin());
        v.push(Piece::new(Shape::Cube, dir * 0.55 + Vec3::Y * (top - 0.03), Vec3::new(0.9, 0.06, 0.42), 0).yaw(a));
    }
    v
}

/// Première volée vers le palier (centré sur l'origine), seconde volée vers +x.
fn quarter_turn(n: usize, rise: f32) -> Vec<Piece> {
    let run = RUN;
    let n1 = n / 2;
    let mut v = Vec::new();
    for i in 0..n1 {
        v.push(bx(0.0, -0.5 - (n1 - i) as f32 * run + run / 2.0, 1.0, run, 0.0, (i + 1) as f32 * rise, 0));
    }
    v.push(bx(0.0, 0.0, 1.0, 1.0, 0.0, (n1 + 1) as f32 * rise, 0));
    for j in 0..n.saturating_sub(n1 + 1) {
        v.push(bx(0.5 + (j as f32 + 0.5) * run, 0.0, run, 1.0, 0.0, (n1 + 2 + j) as f32 * rise, 0));
    }
    v
}

/// Volée vers l'avant côté -x, palier au fond sur toute la largeur, volée de retour côté +x.
fn half_turn(n: usize, rise: f32) -> Vec<Piece> {
    let n1 = n / 2;
    let half = f32::midpoint(n1 as f32 * RUN, 1.0);
    let mut v: Vec<Piece> = flight(n1, rise, RUN, (-0.55, -half), 1.0).collect();
    v.push(bx(0.0, half - 0.5, 2.1, 1.0, 0.0, (n1 + 1) as f32 * rise, 0));
    for j in 0..n.saturating_sub(n1 + 1) {
        v.push(bx(0.55, half - 1.0 - (j as f32 + 0.5) * RUN, 1.0, RUN, 0.0, (n1 + 2 + j) as f32 * rise, 0));
    }
    v
}

/// Échelle de meunier vers +z : marches minces sans contremarche entre deux limons. Marches de
/// 20 cm au plus (`ceil`) : la visite en franchit deux d'un pas, sans quoi la marche à hauteur de
/// tête bloquerait sur un giron aussi court.
fn ladder(height: f32) -> Vec<Piece> {
    let n = ((height / 0.2).ceil() as usize).max(1);
    let (rise, run) = (height / n as f32, 0.16);
    let z0 = -(n as f32) * run / 2.0;
    let mut v: Vec<Piece> = (1..=n)
        .map(|i| {
            let top = i as f32 * rise;
            bx(0.0, z0 + (i as f32 - 0.5) * run, 0.62, 0.2, top - 0.04, top, 0)
        })
        .collect();
    for x in [-0.33, 0.33] {
        v.push(stringer(x, (z0, rise / run, rise / 2.0 - 0.02), height, (0.05, 0.2), 1));
    }
    v
}

/// Marches épaisses sans contremarche vers +z, scellées dans un limon mural côté -x.
fn floating(n: usize, rise: f32) -> Vec<Piece> {
    let z0 = -(n as f32) * RUN / 2.0;
    let mut v: Vec<Piece> = (1..=n)
        .map(|i| {
            let top = i as f32 * rise;
            bx(0.0, z0 + (i as f32 - 0.5) * RUN, 0.9, RUN, top - 0.08, top, 0)
        })
        .collect();
    v.push(stringer(-0.47, (z0, rise / RUN, rise / 2.0 + 0.03), n as f32 * rise + 0.1, (0.06, 0.3), 1));
    v
}

/// Perron : marches de pierre d'environ 16 cm qui montent vers l'arrière (-z) jusqu'à un palier
/// d'un mètre contre la façade, entre deux murets.
fn stone_steps(height: f32) -> Vec<Piece> {
    let n = ((height / 0.16).round() as usize).max(1);
    let (rise, run, land, w) = (height / n as f32, 0.35, 1.0, 2.4);
    let half = f32::midpoint((n - 1) as f32 * run, land);
    let mut v = Vec::new();
    for k in 1..=n {
        let top = k as f32 * rise;
        // la k-ième marche depuis l'avant ; la dernière est le palier
        let (back, front) =
            if k == n { (-half, -half + land) } else { (half - k as f32 * run, half - (k - 1) as f32 * run) };
        let (z, d) = (f32::midpoint(back, front), front - back);
        v.push(bx(0.0, z, w, d, 0.0, top, 0));
        for x in [-1.0, 1.0] {
            v.push(bx(x * (w / 2.0 + 0.15), z, 0.3, d, 0.0, top + 0.15, 1));
        }
    }
    v
}

/// Escalator à 30° vers +z : marches au nez jaune, paliers d'un mètre en bas et en haut,
/// habillage, balustrades vitrées et mains courantes des deux côtés.
fn escalator(n: usize, rise: f32) -> Vec<Piece> {
    let run = 0.35;
    let len = n as f32 * run;
    let (z0, h, slope) = (-len / 2.0, n as f32 * rise, rise / run);
    let mut v = Vec::new();
    for i in 0..n {
        let (front, top) = (z0 + i as f32 * run, (i + 1) as f32 * rise);
        v.push(bx(0.0, front + f32::midpoint(run, 0.04), 1.0, run - 0.04, (top - 0.5).max(0.0), top, 0));
        v.push(bx(0.0, front + 0.02, 1.0, 0.04, top - 0.03, top, 1));
    }
    v.push(bx(0.0, z0 - 0.5, 1.1, 1.0, 0.0, 0.02, 2));
    v.push(bx(0.0, z0 + len + 0.5, 1.1, 1.0, h - 0.3, h, 2));
    let glass = |x: f32, z: f32, y: f32| Piece { shape: Shape::Glass, ..bx(x, z, 0.02, 1.0, y + 0.15, y + 0.95, 0) };
    let cos = 1.0 / slope.hypot(1.0);
    for x in [-0.56_f32, 0.56] {
        v.push(stringer(x.signum() * 0.53, (z0, slope, 0.1), h, (0.06, 0.9), 2));
        v.push(glass(x, z0 - 0.5, 0.0));
        v.push(glass(x, z0 + len + 0.5, h));
        let (foot, head) = (Vec3::new(x, 0.0, z0), Vec3::new(x, h, z0 + len));
        v.push(board(Shape::Glass, foot + Vec3::Y * 0.55, head + Vec3::Y * 0.55, (0.02, 0.8 * cos), 0));
        let rail = Vec3::Y * 1.0;
        for (a, b) in [(foot - Vec3::Z, foot), (foot, head), (head, head + Vec3::Z)] {
            v.push(span(Shape::Cylinder, a + rail, b + rail, 0.08, 3));
        }
    }
    v
}

/// Rampe PMR à 5 % vers +z puis palier de 1,4 m. La pente est faite de dalles penchées d'un mètre
/// au plus, chacune sur un bloc plein : la visite heurte une dalle comme sa boîte droite, dont le
/// dessus est le haut de la dalle, soit une marche de 5 cm.
fn ramp(height: f32) -> Vec<Piece> {
    let len = height / 0.05;
    let m = (len.ceil() as usize).max(1);
    let z0 = -(len + 1.4) / 2.0;
    let y = |z: f32| ((z - z0) * 0.05).clamp(0.0, height);
    let (t, cos) = (0.1, 1.0 / 0.05_f32.hypot(1.0));
    // la dalle pend sous la ligne de pente, de son épaisseur perpendiculaire
    let under = Vec3::new(0.0, -cos, 0.05 * cos) * (t / 2.0);
    let mut v = Vec::new();
    for j in 0..m {
        let (za, zb) = (z0 + len * j as f32 / m as f32, z0 + len * (j + 1) as f32 / m as f32);
        let (a, b) = (Vec3::new(0.0, y(za), za), Vec3::new(0.0, y(zb), zb));
        v.push(board(Shape::Cube, a + under, b + under, (1.4, t), 0));
        if y(za) > 0.01 {
            v.push(bx(0.0, f32::midpoint(za, zb), 1.4, zb - za, 0.0, y(za), 0));
        }
    }
    let end = z0 + len + 1.4;
    v.push(bx(0.0, end - 0.7, 1.4, 1.4, 0.0, height, 0));
    let posts = (len / 2.0).ceil() as usize;
    for x in [-0.65, 0.65] {
        for k in 0..=posts {
            let z = z0 + len * k as f32 / posts.max(1) as f32;
            v.push(cyl(x, z, 0.05, y(z), y(z) + 1.0, 1));
        }
        v.push(cyl(x, end - 0.05, 0.05, height, height + 1.0, 1));
        let (foot, top, head) = (Vec3::new(x, 0.0, z0), Vec3::new(x, height, z0 + len), Vec3::new(x, height, end));
        for (dy, d, slot) in [(1.0, 0.05, 2), (0.45, 0.03, 1)] {
            let up = Vec3::Y * dy;
            v.push(span(Shape::Cylinder, foot + up, top + up, d, slot));
            v.push(span(Shape::Cylinder, top + up, head + up, d, slot));
        }
    }
    v
}

/// Colonne ronde : plinthe et tore, fût, astragale, échine et abaque, jusqu'à `h` exactement.
fn column(h: f32) -> Vec<Piece> {
    let e = (h * 0.04).min(0.08);
    vec![
        bx(0.0, 0.0, 0.5, 0.5, 0.0, e, 1),
        cyl(0.0, 0.0, 0.44, e, 2.0 * e, 1),
        cyl(0.0, 0.0, 0.34, 2.0 * e, h - 2.0 * e, 0),
        cyl(0.0, 0.0, 0.38, h - 2.5 * e, h - 2.0 * e, 1),
        cyl(0.0, 0.0, 0.44, h - 2.0 * e, h - e, 1),
        bx(0.0, 0.0, 0.5, 0.5, h - e, h, 1),
    ]
}

/// Garde-corps de 2 m selon x : deux montants, lisse basse, barreaux tous les 11 cm, main courante à 1 m.
fn railing() -> Vec<Piece> {
    let mut v = vec![bx(-0.97, 0.0, 0.05, 0.05, 0.0, 0.95, 0), bx(0.97, 0.0, 0.05, 0.05, 0.0, 0.95, 0)];
    v.push(bx(0.0, 0.0, 1.9, 0.03, 0.05, 0.09, 0));
    v.extend((0..17).map(|i| cyl(-0.88 + i as f32 * 0.11, 0.0, 0.016, 0.09, 0.95, 0)));
    v.push(bx(0.0, 0.0, 2.0, 0.07, 0.95, 1.0, 1));
    v
}

/// Balustrade de 2 m selon x : socle, dés aux deux bouts, six balustres (pied, panse, col, tête), tablette.
fn balustrade() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 2.0, 0.34, 0.0, 0.15, 0), bx(0.0, 0.0, 2.0, 0.36, 0.78, 0.9, 0)];
    for x in [-0.9, 0.9] {
        v.push(bx(x, 0.0, 0.2, 0.3, 0.15, 0.78, 0));
    }
    for i in 0..6 {
        let x = -0.625 + i as f32 * 0.25;
        v.push(bx(x, 0.0, 0.13, 0.13, 0.15, 0.21, 1));
        v.push(ball([x, 0.36, 0.0], [0.16, 0.3, 0.16], 1));
        v.push(cyl(x, 0.0, 0.08, 0.2, 0.72, 1));
        v.push(bx(x, 0.0, 0.12, 0.12, 0.72, 0.78, 1));
    }
    v
}

/// Arche en plein cintre : deux jambages sur leur base, impostes, et neuf voussoirs tournés autour
/// de l'axe avant-arrière (rayon intérieur 0,6 m, naissance à 2,1 m), la clé au milieu. Les
/// voussoirs alternent d'épaisseur pour que leurs faces ne se confondent pas.
fn arch() -> Vec<Piece> {
    let (r, ring, spring, n) = (0.6, 0.3, 2.1, 9);
    let mut v = Vec::new();
    for x in [-0.8, 0.8] {
        v.push(bx(x, 0.0, 0.5, 0.5, 0.0, 0.15, 0));
        v.push(bx(x, 0.0, 0.4, 0.4, 0.15, spring - 0.12, 0));
        v.push(bx(x, 0.0, 0.5, 0.46, spring - 0.12, spring, 1));
    }
    let chord = 2.0 * (r + ring) * (PI / (2.0 * n as f32)).sin();
    for i in 0..n {
        let a = PI * (i as f32 + 0.5) / n as f32;
        let key = i == n / 2;
        let (depth, radial) = if key { (0.44, ring + 0.08) } else { (0.4 - 0.03 * (i % 2) as f32, ring) };
        let at = Vec3::new(a.cos(), a.sin(), 0.0) * (r + ring / 2.0) + Vec3::Y * spring;
        let piece = Piece::new(Shape::Cube, at, Vec3::new(chord, radial, depth), usize::from(key));
        v.push(piece.turn(Quat::from_rotation_z(a - PI / 2.0)));
    }
    v
}

#[cfg(test)]
mod tests {
    use bevy::math::{Mat3, Vec2};

    use super::*;

    /// Règles de la visite (`visit`) : marche franchissable, rayon et taille du visiteur.
    const STEP: f32 = 0.45;
    const RADIUS: f32 = 0.25;
    const TALL: f32 = 1.75;

    /// Boîte droite (min, max) d'une pièce, comme la collision de la visite (une pièce tournée y
    /// devient sa boîte englobante : plus prudent pour les marches tournées en plan).
    fn upright(p: &Piece) -> (Vec3, Vec3) {
        let m = Mat3::from_quat(p.rot);
        let half = (m.x_axis.abs() * p.size.x + m.y_axis.abs() * p.size.y + m.z_axis.abs() * p.size.z) / 2.0;
        (p.at - half, p.at + half)
    }

    fn bounds(pieces: &[Piece]) -> (Vec3, Vec3) {
        pieces.iter().map(upright).fold((Vec3::MAX, Vec3::MIN), |(lo, hi), (a, b)| (lo.min(a), hi.max(b)))
    }

    /// Marche le long de `path` (points `[x, z]` du repère de l'objet) par pas de 5 cm comme la
    /// visite : monte sur le plus haut dessus à moins de `STEP`, échoue contre tout ce qui est plus
    /// haut. Renvoie la plus haute altitude atteinte.
    fn walk(kind: PropKind, height: f32, path: &[[f32; 2]]) -> f32 {
        let boxes: Vec<_> = shapes(kind, 0.0, height).iter().filter(|p| p.shape != Shape::Light).map(upright).collect();
        let (mut y, mut best) = (0.0_f32, 0.0_f32);
        for leg in path.windows(2) {
            let (a, b) = (Vec2::from(leg[0]), Vec2::from(leg[1]));
            let steps = (a.distance(b) / 0.05).ceil() as usize;
            for i in 1..=steps {
                let at = a.lerp(b, i as f32 / steps as f32);
                for (lo, hi) in &boxes {
                    let gap = at - at.clamp(Vec2::new(lo.x, lo.z), Vec2::new(hi.x, hi.z));
                    let blocks = hi.y > y + STEP && lo.y < y + TALL && gap.length() < RADIUS;
                    assert!(!blocks, "{kind:?} ({height} m) bloqué en {at} à {y} m par {lo} {hi}");
                }
                // 1 mm de marge : entre deux marches jointives, la visite ne tombe pas d'un coup (gravité)
                let under = |b: &&(Vec3, Vec3)| {
                    at.cmpge(Vec2::new(b.0.x, b.0.z) - 1e-3).all() && at.cmple(Vec2::new(b.1.x, b.1.z) + 1e-3).all()
                };
                y = boxes.iter().filter(under).map(|b| b.1.y).filter(|&top| top <= y + STEP).fold(0.0, f32::max);
                best = best.max(y);
            }
        }
        best
    }

    /// Parcours qui gravit l'escalier : de son pied jusqu'à son bout.
    fn route(kind: PropKind, height: f32) -> Vec<[f32; 2]> {
        let (lo, hi) = bounds(&shapes(kind, 0.0, height));
        match kind {
            PropKind::StoneSteps => vec![[0.0, hi.z + 0.5], [0.0, lo.z + 0.2]],
            PropKind::LStairs => vec![[0.0, lo.z - 0.5], [0.0, 0.0], [hi.x - 0.2, 0.0]],
            PropKind::UStairs => vec![[-0.55, lo.z - 0.5], [-0.55, hi.z - 0.5], [0.55, hi.z - 0.5], [0.55, lo.z + 0.2]],
            _ => vec![[0.0, lo.z - 0.5], [0.0, hi.z - 0.05]],
        }
    }

    #[test]
    fn every_stair_is_climbed_to_exactly_its_height() {
        use PropKind::*;
        assert!([UStairs, Ladder, FloatingStairs, StoneSteps, Escalator, Ramp].iter().all(|k| k.climbs()));
        // colimaçon : marches tournées en plan, parcours en hélice, hors de ce marcheur en ligne
        for kind in PropKind::ALL.into_iter().filter(|&k| k.climbs() && k != SpiralStairs) {
            for height in [1.0, 3.0, 4.5] {
                let top = walk(kind, height, &route(kind, height));
                assert!((top - height).abs() < 1e-3, "{kind:?} {height} → {top}");
            }
        }
    }

    #[test]
    fn one_walks_under_the_arch_and_columns_reach_their_height() {
        assert_eq!(walk(PropKind::Arch, 3.0, &[[0.0, -1.0], [0.0, 1.0]]), 0.0);
        for h in [0.1, 1.0, 3.0, 4.5] {
            let pieces = shapes(PropKind::Column, 0.0, h);
            assert!((bounds(&pieces).1.y - h).abs() < 1e-4, "colonne de {h} m");
            assert!(pieces.iter().all(|p| p.size.min_element() > 0.0), "colonne de {h} m : pièce retournée");
        }
    }

    #[test]
    fn railings_are_two_metre_modules_at_guard_height() {
        for kind in [PropKind::Railing, PropKind::Balustrade] {
            let size = kind.size();
            assert!((size.x - 2.0).abs() < 0.01 && (0.85..=1.0).contains(&size.z), "{kind:?} {size}");
            assert!(!kind.climbs() && kind.dim().is_none());
        }
        assert!(!PropKind::Column.climbs() && !PropKind::Arch.climbs());
    }

    #[test]
    #[should_panic(expected = "bloqué")]
    fn the_walker_bumps_into_what_it_cannot_climb() {
        walk(PropKind::Pillar, 3.0, &[[0.0, -1.0], [0.0, 1.0]]);
    }

    #[test]
    fn ramp_keeps_a_gentle_slope_and_stone_steps_stay_low() {
        let (lo, hi) = bounds(&shapes(PropKind::Ramp, 0.0, 0.5));
        assert!((hi.z - lo.z - 11.4).abs() < 0.1, "5 % : 10 m de pente et le palier ({lo} {hi})");
        let risers = shapes(PropKind::StoneSteps, 0.0, 0.5).iter().filter(|p| p.slot == 0).count();
        assert_eq!(risers, 3, "un seuil à 50 cm : trois marches");
    }
}
