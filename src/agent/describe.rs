//! Résumé texte d'un monde, avec les références utilisables dans les commandes.

use std::fmt::Write;

use bevy::math::Vec2;

use crate::model::{House, RoofKind, elev};
use crate::ui::level_name;

/// Coins de l'emprise au sol de tous les éléments, `None` pour un monde vide.
pub(super) fn bounds(h: &House) -> Option<(Vec2, Vec2)> {
    let pts = (h.walls.iter().flat_map(|w| [w.a, w.b]))
        .chain(h.slabs.iter().flat_map(|s| [s.min, s.max]))
        .chain(h.roofs.iter().flat_map(|r| [r.min, r.max]))
        .chain(h.props.iter().map(|p| p.pos))
        .chain(h.pools.iter().flat_map(|p| [p.min, p.max]))
        .chain(h.roads.iter().flat_map(|r| [r.a, r.b]));
    pts.fold(None, |acc: Option<(Vec2, Vec2)>, p| Some(acc.map_or((p, p), |(lo, hi)| (lo.min(p), hi.max(p)))))
}

/// Emprise des seuls bâtiments (murs, sols, toits), sinon de tout le monde.
pub(super) fn building_bounds(h: &House) -> Option<(Vec2, Vec2)> {
    let pts = (h.walls.iter().flat_map(|w| [w.a, w.b]))
        .chain(h.slabs.iter().flat_map(|s| [s.min, s.max]))
        .chain(h.roofs.iter().flat_map(|r| [r.min, r.max]));
    pts.fold(None, |acc: Option<(Vec2, Vec2)>, p| Some(acc.map_or((p, p), |(lo, hi)| (lo.min(p), hi.max(p)))))
        .or_else(|| bounds(h))
}

pub(super) fn pt(p: Vec2) -> String {
    let r = |v: f32| (v * 100.0).round() / 100.0;
    format!("[{}, {}]", r(p.x), r(p.y))
}

/// Résumé du monde par étage. Les objets portent leur type JSON (`Sofa`) et leur emprise au sol.
pub fn describe(h: &House) -> String {
    // ici les `let _ = write!` sont sans danger : écrire dans une String n'échoue jamais
    let mut s = String::new();
    let Some((lo, hi)) = bounds(h) else {
        return "Monde vide.\n".into();
    };
    let _ = writeln!(s, "Emprise : {} → {} ({:.1} × {:.1} m)", pt(lo), pt(hi), hi.x - lo.x, hi.y - lo.y);
    let mut levels: Vec<i32> = (h.walls.iter().map(|w| w.level))
        .chain(h.slabs.iter().map(|s| s.level))
        .chain(h.roofs.iter().map(|r| r.level))
        .chain(h.props.iter().map(|p| p.level))
        .collect();
    levels.sort_unstable();
    levels.dedup();
    for l in levels {
        level_section(&mut s, h, l);
    }
    if !h.pools.is_empty() || !h.roads.is_empty() {
        s.push_str("\nExtérieur\n");
    }
    for (i, p) in h.pools.iter().enumerate() {
        let _ = writeln!(s, "  Pool {i} : {} → {}, prof {} m", pt(p.min), pt(p.max), p.depth);
    }
    for (i, r) in h.roads.iter().enumerate() {
        let _ = writeln!(s, "  Road {i} : {} → {}, larg {} m", pt(r.a), pt(r.b), r.width);
    }
    let (min, max) = h.terrain.h.iter().fold((0.0f32, 0.0f32), |(a, b), &y| (a.min(y), b.max(y)));
    let _ = writeln!(s, "\nRelief : altitude de {min:.1} à {max:.1} m");
    s
}

fn level_section(s: &mut String, h: &House, l: i32) {
    let _ = writeln!(
        s,
        "\n{} (level {l}, sol à {} m) — surface de sol {:.1} m²",
        level_name(l),
        elev(l),
        h.floor_area(l).abs()
    );
    for (i, w) in h.walls.iter().enumerate().filter(|(_, w)| w.level == l) {
        let _ = write!(
            s,
            "  Wall {i} : {} → {}, {:.2} m, haut {} m, ép {} m, texture {}",
            pt(w.a),
            pt(w.b),
            w.len(),
            w.height,
            w.thick,
            w.texture
        );
        for (j, o) in w.openings.iter().enumerate() {
            let _ = write!(
                s,
                "\n    Opening [{i}, {j}] : {:?} ({}) à t={} (larg {} m, haut {} m, allège {} m)",
                o.kind,
                o.kind.name(),
                o.t,
                o.width,
                o.height,
                o.sill
            );
        }
        s.push('\n');
    }
    for (i, sl) in h.slabs.iter().enumerate().filter(|(_, s)| s.level == l) {
        let _ = writeln!(s, "  Slab {i} : {} → {}, texture {}", pt(sl.min), pt(sl.max), sl.texture);
    }
    for (i, r) in h.roofs.iter().enumerate().filter(|(_, r)| r.level == l) {
        let height = if r.kind == RoofKind::Flat { String::new() } else { format!(", haut {} m", r.height) };
        let _ = writeln!(s, "  Roof {i} : {:?} ({}) {} → {}{height}", r.kind, r.kind.name(), pt(r.min), pt(r.max));
    }
    for (i, p) in h.props.iter().enumerate().filter(|(_, p)| p.level == l) {
        let (lo, hi) = p.corners().iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), c| (lo.min(*c), hi.max(*c)));
        let height = p.kind.dim().map_or(String::new(), |d| format!(", {d} {} m", p.height));
        let _ = writeln!(
            s,
            "  Prop {i} : {:?} ({}) en {}, rot {}°{height}, emprise {} → {}",
            p.kind,
            p.kind.name(),
            pt(p.pos),
            p.rot,
            pt(lo),
            pt(hi)
        );
    }
}
