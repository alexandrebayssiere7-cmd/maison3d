//! Tracés de terrain : lignes et courbes blanches de 1 cm d'épaisseur posées juste au-dessus du
//! sol, et terrains complets aux cotes officielles (longueur le long de x) sur leur revêtement.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::Vec3;

use super::super::{Piece, bx, cyl};
use super::arc;
use crate::model::{PropKind, Slot, WHITE};

/// Revêtement de 5 mm ; les lignes, de 1 cm, sont posées dessus (entre ces deux hauteurs).
const SURF: f32 = 0.005;
const LINE: (f32, f32) = (SURF, SURF + 0.01);
/// Largeur des tracés génériques et des lignes de foot.
const WIDE: f32 = 0.1;
/// Largeur des lignes des terrains de salle (basket, tennis, volley, hand).
const THIN: f32 = 0.05;

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    const LINES: Slot = Slot::plain("Lignes", WHITE);
    match kind {
        PropKind::SoccerPitch5 | PropKind::SoccerPitch => {
            const { &[LINES, Slot::textured("Revêtement", "Gazon synthétique")] }
        }
        PropKind::BasketCourt | PropKind::VolleyCourt => {
            const { &[LINES, Slot::textured("Revêtement", "Parquet de sport")] }
        }
        PropKind::TennisCourt => const { &[LINES, Slot::textured("Revêtement", "Terre battue")] },
        PropKind::HandballCourt => const { &[LINES, Slot::textured("Revêtement", "Résine bleue")] },
        _ => const { &[LINES] },
    }
}

/// `height` est la longueur de `FieldLine` et le rayon des arcs et du cercle.
pub(super) fn shapes(kind: PropKind, height: f32) -> Vec<Piece> {
    match kind {
        PropKind::FieldLine => vec![bx(0.0, 0.0, height, WIDE, LINE.0, LINE.1, 0)],
        PropKind::FieldArc => curve(0.0, 0.0, height, (PI, TAU), WIDE),
        PropKind::FieldQuarterArc => curve(0.0, 0.0, height, (1.5 * PI, TAU), WIDE),
        PropKind::FieldCircle => curve(0.0, 0.0, height, (0.0, TAU), WIDE),
        PropKind::FieldDot => vec![dot(0.0, 0.0)],
        PropKind::SoccerPitch5 => soccer5(),
        PropKind::SoccerPitch => soccer(),
        PropKind::BasketCourt => basket(),
        PropKind::TennisCourt => tennis(),
        PropKind::VolleyCourt => volley(),
        PropKind::HandballCourt => handball(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------- briques

/// Ligne droite de largeur `w` de `a` à `b` = [x, z], parallèle à x ou à z, bouts compris.
fn line([x0, z0]: [f32; 2], [x1, z1]: [f32; 2], w: f32) -> Piece {
    bx(f32::midpoint(x0, x1), f32::midpoint(z0, z1), (x1 - x0).abs() + w, (z1 - z0).abs() + w, LINE.0, LINE.1, 0)
}

/// Arc de tracé de centre (x, z) et de rayon `r` (angles comme `arc`), large de `w`.
fn curve(x: f32, z: f32, r: f32, angles: (f32, f32), w: f32) -> Vec<Piece> {
    arc(Vec3::new(x, f32::midpoint(LINE.0, LINE.1), z), r, angles, (w, LINE.1 - LINE.0), 0)
}

/// Point de 22 cm (penalty, centre).
fn dot(x: f32, z: f32) -> Piece {
    cyl(x, z, 0.22, LINE.0, LINE.1, 0)
}

/// Terrain symétrique de `len × wid` : revêtement, contour, et la moitié ouest `half` (lignes de
/// largeur `w`, but en x = -len / 2) recopiée en miroir à l'est.
fn court(len: f32, wid: f32, w: f32, half: Vec<Piece>) -> Vec<Piece> {
    let (x, z) = (len / 2.0, wid / 2.0);
    let mut west = vec![line([-x, -z], [0.0, -z], w), line([-x, z], [0.0, z], w), line([-x, -z], [-x, z], w)];
    west.extend(half);
    let mut v = vec![bx(0.0, 0.0, len + w, wid + w, 0.0, SURF, 1)];
    // symétrique par rapport à x = 0 : une pièce tournée de a autour de la verticale le devient de -a
    v.extend(west.iter().map(|p| Piece { at: p.at * Vec3::new(-1.0, 1.0, 1.0), rot: p.rot.inverse(), ..*p }));
    v.extend(west);
    v
}

/// Surface rectangulaire devant le but ouest (ligne de but en x = `gx`), profonde de `d`, large de `2 hz`.
fn area(gx: f32, d: f32, hz: f32, w: f32) -> [Piece; 3] {
    [line([gx, -hz], [gx + d, -hz], w), line([gx, hz], [gx + d, hz], w), line([gx + d, -hz], [gx + d, hz], w)]
}

/// Arcs de corner de rayon `r` aux coins ouest (gx, ±hz).
fn corners(gx: f32, hz: f32, r: f32, w: f32) -> Vec<Piece> {
    let mut v = curve(gx, -hz, r, (1.5 * PI, TAU), w);
    v.extend(curve(gx, hz, r, (0.0, FRAC_PI_2), w));
    v
}

/// Zone en D devant le but ouest (ligne de but en x = `gx`, poteaux en z = ±`post`) : quarts de
/// cercle de rayon `r` centrés sur les poteaux, coupés aux lignes de côté (z = ±`hz`) et reliés
/// par une droite ; un morceau sur deux seulement si `dashed`.
fn d_zone(gx: f32, hz: f32, post: f32, r: f32, w: f32, dashed: bool) -> Vec<Piece> {
    let a = ((hz - post) / r).min(1.0).asin();
    let n = (4.0 * post).round().max(1.0) as usize;
    let seg = 2.0 * post / n as f32;
    let straight = (0..n).map(|i| bx(gx + r, -post + (i as f32 + 0.5) * seg, w, seg, LINE.0, LINE.1, 0));
    // dans l'ordre du tracé, pour que les pointillés alternent tout du long
    let path = curve(gx, -post, r, (0.0, a), w)
        .into_iter()
        .rev()
        .chain(straight)
        .chain(curve(gx, post, r, (TAU - a, TAU), w).into_iter().rev());
    path.step_by(if dashed { 2 } else { 1 }).collect()
}

// ---------------------------------------------------------------- terrains

/// Foot à 11, 105 × 68 m.
fn soccer() -> Vec<Piece> {
    let (w, gx) = (WIDE, -52.5);
    let a = (5.5_f32 / 9.15).acos();
    let mut half = Vec::from(area(gx, 16.5, 20.16, w));
    half.extend(area(gx, 5.5, 9.16, w));
    half.push(dot(gx + 11.0, 0.0));
    half.extend(curve(gx + 11.0, 0.0, 9.15, (-a, a), w));
    half.extend(corners(gx, 34.0, 1.0, w));
    let mut v = court(105.0, 68.0, w, half);
    v.push(line([0.0, -34.0], [0.0, 34.0], w));
    v.extend(curve(0.0, 0.0, 9.15, (0.0, TAU), w));
    v.push(dot(0.0, 0.0));
    v
}

/// Foot à 5 (tracé de futsal), 40 × 20 m : surfaces en arcs de 6 m, points à 6 et 10 m.
fn soccer5() -> Vec<Piece> {
    let (w, gx) = (0.08, -20.0);
    let mut half = d_zone(gx, 10.0, 1.58, 6.0, w, false);
    half.extend([dot(gx + 6.0, 0.0), dot(gx + 10.0, 0.0)]);
    half.extend(corners(gx, 10.0, 0.25, w));
    let mut v = court(40.0, 20.0, w, half);
    v.push(line([0.0, -10.0], [0.0, 10.0], w));
    v.extend(curve(0.0, 0.0, 3.0, (0.0, TAU), w));
    v.push(dot(0.0, 0.0));
    v
}

/// Basket FIBA, 28 × 15 m : raquette de 5,8 × 4,9 m, cercle des lancers francs de 1,8 m, arc à
/// 3 points de 6,75 m autour du panier (1,575 m de la ligne de fond) prolongé par des droites à
/// 0,9 m des lignes de côté, demi-cercle de non-charge de 1,25 m.
fn basket() -> Vec<Piece> {
    let (w, gx) = (THIN, -14.0);
    let (hoop, ft, side) = (gx + 1.575, gx + 5.8, 7.5_f32 - 0.9);
    let a = (side / 6.75).asin();
    let reach = hoop + 6.75 * a.cos();
    let mut half = Vec::from(area(gx, 5.8, 2.45, w));
    half.extend([line([gx, -side], [reach, -side], w), line([gx, side], [reach, side], w)]);
    half.extend(curve(hoop, 0.0, 6.75, (-a, a), w));
    half.extend(curve(ft, 0.0, 1.8, (-FRAC_PI_2, FRAC_PI_2), w));
    half.extend(curve(hoop, 0.0, 1.25, (-FRAC_PI_2, FRAC_PI_2), w));
    let mut v = court(28.0, 15.0, w, half);
    v.push(line([0.0, -7.5], [0.0, 7.5], w));
    v.extend(curve(0.0, 0.0, 1.8, (0.0, TAU), w));
    v
}

/// Tennis, 23,77 × 10,97 m : couloirs (simple à ±4,115 m), lignes de service à 6,4 m du filet.
fn tennis() -> Vec<Piece> {
    let (w, gx, single, service) = (THIN, -11.885, 4.115, -6.4);
    let half = vec![
        line([gx, -single], [0.0, -single], w),
        line([gx, single], [0.0, single], w),
        line([service, -single], [service, single], w),
        line([service, 0.0], [0.0, 0.0], w),
        line([gx, 0.0], [gx + 0.1, 0.0], w),
    ];
    court(23.77, 10.97, w, half)
}

/// Volley, 18 × 9 m : ligne centrale, lignes d'attaque à 3 m.
fn volley() -> Vec<Piece> {
    let mut v = court(18.0, 9.0, THIN, vec![line([-3.0, -4.5], [-3.0, 4.5], THIN)]);
    v.push(line([0.0, -4.5], [0.0, 4.5], THIN));
    v
}

/// Hand, 40 × 20 m : zone de but à 6 m, ligne de jet franc à 9 m en pointillés, marques des 7 m
/// et de la limite du gardien (4 m).
fn handball() -> Vec<Piece> {
    let (w, gx) = (THIN, -20.0);
    let mut half = d_zone(gx, 10.0, 1.5, 6.0, w, false);
    half.extend(d_zone(gx, 10.0, 1.5, 9.0, w, true));
    half.push(line([gx + 7.0, -0.5], [gx + 7.0, 0.5], w));
    half.push(line([gx + 4.0, -0.075], [gx + 4.0, 0.075], w));
    let mut v = court(40.0, 20.0, w, half);
    v.push(line([0.0, -10.0], [0.0, 10.0], w));
    v
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::{Category, Prop};

    /// Encombrement [x, z, hauteur] de l'objet pour une dimension réglable `height`.
    fn size(kind: PropKind, height: f32) -> Vec3 {
        let (lo, hi) = Prop { height, ..Prop::new(kind, Vec2::ZERO, 0, 0.0) }.local_box();
        let s = hi - lo;
        Vec3::new(s.x, s.z, s.y)
    }

    #[test]
    fn markings_are_flat_outdoor_and_lie_on_the_ground() {
        for k in Category::Markings.kinds() {
            assert!(k.flat() && k.outdoor(), "{k:?}");
            assert!(shapes(k, 3.0).iter().all(|p| p.at.y > 0.0 && p.at.y < 0.02), "{k:?}");
        }
    }
    #[test]
    fn length_and_radius_follow_height() {
        for h in [0.5, 9.15, 40.0] {
            assert!((size(PropKind::FieldLine, h) - Vec3::new(h, WIDE, 0.01)).abs().max_element() < 1e-3);
            let circle = size(PropKind::FieldCircle, h);
            assert!((circle.x - 2.0 * h - WIDE).abs() < 0.02 && (circle.y - 2.0 * h - WIDE).abs() < 0.02, "{circle}");
            let half = size(PropKind::FieldArc, h);
            assert!((half.x - 2.0 * h - WIDE).abs() < 0.02 && (half.y - h - WIDE / 2.0).abs() < 0.05, "{half}");
            let quarter = size(PropKind::FieldQuarterArc, h);
            assert!((quarter.x - h).abs() < 0.1 && (quarter.y - h).abs() < 0.1, "{quarter}");
        }
        assert_eq!(PropKind::FieldCircle.dim(), Some("rayon"));
        assert_eq!(PropKind::FieldLine.dim(), Some("longueur"));
    }
    #[test]
    fn full_courts_have_official_dimensions() {
        for (k, len, wid) in [
            (PropKind::SoccerPitch5, 40.0, 20.0),
            (PropKind::SoccerPitch, 105.0, 68.0),
            (PropKind::BasketCourt, 28.0, 15.0),
            (PropKind::TennisCourt, 23.77, 10.97),
            (PropKind::VolleyCourt, 18.0, 9.0),
            (PropKind::HandballCourt, 40.0, 20.0),
        ] {
            let s = k.size();
            assert!((s.x - len).abs() < 0.11 && (s.y - wid).abs() < 0.11, "{k:?} : {s}");
            assert_eq!(k.slots()[1].name, "Revêtement");
        }
    }
    #[test]
    fn courts_are_mirrored_and_dashes_skip_every_other_piece() {
        let pieces = soccer();
        // hors du rond central, dont le nombre de segments peut être impair
        let east = pieces.iter().filter(|p| p.at.x > 10.0).count();
        assert_eq!(east, pieces.iter().filter(|p| p.at.x < -10.0).count());
        let solid = d_zone(-20.0, 10.0, 1.5, 9.0, THIN, false);
        let dashed = d_zone(-20.0, 10.0, 1.5, 9.0, THIN, true);
        assert_eq!(dashed.len(), solid.len().div_ceil(2));
        assert!(solid.iter().all(|p| p.at.z.abs() < 10.0), "la ligne des 9 m s'arrête aux lignes de côté");
    }
}
