//! Bornes des valeurs acceptées par les commandes. Les mêmes bornes figurent dans la doc du schéma.

use bevy::math::Vec2;

use super::EditError;
use crate::model::{
    Category, ElemRef, Finish, House, Opening, OpeningKind, Pool, Prop, Rgb, Road, Roof, Slab, Slot, Wall,
};
use crate::terrain::HALF;

pub(super) fn level(l: i32) -> Result<(), EditError> {
    if (-1..=9).contains(&l) { Ok(()) } else { Err(EditError::LevelOutOfRange(l)) }
}

pub(super) fn rect(min: Vec2, max: Vec2) -> Result<(), EditError> {
    point(min)?;
    point(max)?;
    if min.cmplt(max).all() { Ok(()) } else { Err(EditError::EmptyRect { min, max }) }
}

/// Dans le terrain ; refuse aussi l'infini et NaN, qui deviendraient `null` dans le fichier.
pub(super) fn point(p: Vec2) -> Result<(), EditError> {
    if p.x.abs() <= HALF && p.y.abs() <= HALF { Ok(()) } else { Err(EditError::OutOfWorld(p)) }
}

pub(super) fn range(field: &'static str, value: f32, min: f32, max: f32) -> Result<(), EditError> {
    if (min..=max).contains(&value) { Ok(()) } else { Err(EditError::OutOfRange { field, value, min, max }) }
}

pub(super) fn color(c: Rgb) -> Result<(), EditError> {
    if c.iter().all(|v| (0.0..=1.0).contains(v)) { Ok(()) } else { Err(EditError::BadColor(c)) }
}

pub(super) fn texture(t: usize) -> Result<(), EditError> {
    let max = crate::textures::count();
    if t <= max { Ok(()) } else { Err(EditError::UnknownTexture { texture: t, max }) }
}

fn look(c: Rgb, t: usize) -> Result<(), EditError> {
    color(c)?;
    texture(t)
}

/// Finitions des parties après la principale : une au plus par partie, chacune valide.
fn finishes(list: &[Finish], slots: &[Slot]) -> Result<(), EditError> {
    let max = slots.len().saturating_sub(1);
    if list.len() > max {
        return Err(EditError::TooManyFinishes { count: list.len(), max });
    }
    list.iter().try_for_each(|f| look(f.color, f.texture))
}

pub(super) fn wall(w: &Wall) -> Result<(), EditError> {
    level(w.level)?;
    point(w.a)?;
    point(w.b)?;
    if !(0.05..=f32::MAX).contains(&w.len()) {
        return Err(EditError::WallTooShort);
    }
    range("height", w.height, 0.1, 30.0)?;
    range("thick", w.thick, 0.02, 3.0)?;
    look(w.color, w.texture)?;
    w.openings.iter().enumerate().try_for_each(|(i, o)| opening(w, o, Some(i)))
}

/// Une ouverture `o` dans le mur `w` ; `me` est son propre indice si elle y est déjà.
pub(super) fn opening(w: &Wall, o: &Opening, me: Option<usize>) -> Result<(), EditError> {
    range("width", o.width, 0.2, 10.0)?;
    range("height", o.height, 0.2, 10.0)?;
    range("sill", o.sill, 0.0, 10.0)?;
    look(o.color, o.texture)?;
    finishes(&o.finishes, o.kind.slots())?;
    if o.sill + o.height > w.height + 1e-3 {
        return Err(EditError::OpeningTooTall { top: o.sill + o.height, wall_height: w.height });
    }
    place(w, o.kind, o.t, o.width, me)
}

/// L'ouverture tient-elle à `t` sur ce mur ? Sinon, pourquoi.
pub(super) fn place(w: &Wall, kind: OpeningKind, t: f32, width: f32, me: Option<usize>) -> Result<(), EditError> {
    let (t_min, t_max) = w.t_range(width);
    if !(t_min..=t_max).contains(&t) {
        return Err(EditError::OpeningOutsideWall { kind, t, t_min, t_max, wall_len: w.len() });
    }
    match w.overlap(t, width, me) {
        Some(other) => Err(EditError::OpeningOverlaps { kind, t, other }),
        None => Ok(()),
    }
}

pub(super) fn slab(s: &Slab) -> Result<(), EditError> {
    level(s.level)?;
    rect(s.min, s.max)?;
    look(s.color, s.texture)
}

pub(super) fn roof(r: &Roof) -> Result<(), EditError> {
    level(r.level)?;
    rect(r.min, r.max)?;
    range("height", r.height, 0.0, 30.0)?;
    range("overhang", r.overhang, 0.0, 5.0)?;
    look(r.color, r.texture)
}

pub(super) fn prop(p: &Prop) -> Result<(), EditError> {
    level(p.level)?;
    point(p.pos)?;
    range("rot", p.rot, -36_000.0, 36_000.0)?;
    // une ligne de terrain peut traverser tout le monde
    let max = if p.kind.category() == Category::Markings { 2.0 * HALF } else { 30.0 };
    range("height", p.height, 0.1, max)?;
    look(p.color, p.texture)?;
    finishes(&p.finishes, p.kind.slots())
}

pub(super) fn pool(p: &Pool) -> Result<(), EditError> {
    rect(p.min, p.max)?;
    if p.min != p.min.round() || p.max != p.max.round() {
        return Err(EditError::PoolOffGrid);
    }
    range("depth", p.depth, 0.2, 10.0)?;
    look(p.color, p.texture)
}

pub(super) fn road(r: &Road) -> Result<(), EditError> {
    point(r.a)?;
    point(r.b)?;
    if !(0.05..=f32::MAX).contains(&r.a.distance(r.b)) {
        return Err(EditError::RoadTooShort);
    }
    range("width", r.width, 0.5, 30.0)?;
    look(r.color, r.texture)
}

/// Décalage d'un `move` ou d'un `duplicate`.
pub(super) fn offset(by: Vec2) -> Result<(), EditError> {
    range("by[0]", by.x, -2.0 * HALF, 2.0 * HALF)?;
    range("by[1]", by.y, -2.0 * HALF, 2.0 * HALF)
}

impl House {
    /// Nombre d'éléments de la liste visée par `e` (pour les messages d'erreur).
    pub(super) fn count(&self, e: ElemRef) -> usize {
        match e {
            ElemRef::Wall(_) => self.walls.len(),
            ElemRef::Opening(w, _) => self.walls.get(w).map_or(0, |w| w.openings.len()),
            ElemRef::Slab(_) => self.slabs.len(),
            ElemRef::Roof(_) => self.roofs.len(),
            ElemRef::Prop(_) => self.props.len(),
            ElemRef::Pool(_) => self.pools.len(),
            ElemRef::Road(_) => self.roads.len(),
        }
    }

    /// Revalide un élément existant après un déplacement ou un `set`.
    pub(super) fn check_elem(&self, e: ElemRef) -> Result<(), EditError> {
        match e {
            ElemRef::Wall(i) => wall(&self.walls[i]),
            ElemRef::Opening(w, o) => opening(&self.walls[w], &self.walls[w].openings[o], Some(o)),
            ElemRef::Slab(i) => slab(&self.slabs[i]),
            ElemRef::Roof(i) => roof(&self.roofs[i]),
            ElemRef::Prop(i) => prop(&self.props[i]),
            ElemRef::Pool(i) => pool(&self.pools[i]),
            ElemRef::Road(i) => road(&self.roads[i]),
        }
    }
}
