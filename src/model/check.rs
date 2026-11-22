//! Contrôles de cohérence d'un monde : ce que la validation d'une commande seule ne peut pas voir
//! (objets qui se chevauchent, flottent, traversent un mur, escaliers sans trémie…).

use bevy::math::{Vec2, Vec3};

use super::{House, Prop, PropKind, Slab, Wall};

/// Chevauchement toléré entre deux emprises (contact, approximations de géométrie).
const SLACK: f32 = 0.03;

impl House {
    /// Avertissements lisibles, un par problème, avec les indices des éléments en cause.
    pub fn warnings(&self) -> Vec<String> {
        let mut w = Vec::new();
        self.prop_warnings(&mut w);
        self.stairs_warnings(&mut w);
        self.wall_warnings(&mut w);
        self.outdoor_warnings(&mut w);
        self.name_warnings(&mut w);
        w
    }

    fn prop_warnings(&self, out: &mut Vec<String>) {
        let indoor: Vec<(usize, &Prop, Solid)> =
            self.props.iter().enumerate().filter_map(|(i, p)| Solid::of(p).map(|s| (i, p, s))).collect();
        for (k, (i, p, a)) in indoor.iter().enumerate() {
            for (j, q, b) in &indoor[k + 1..] {
                if a.clash(b) {
                    out.push(format!("Prop {i} ({:?}) et Prop {j} ({:?}) se chevauchent", p.kind, q.kind));
                }
            }
            for (wi, wall) in self.walls.iter().enumerate().filter(|(_, w)| w.level == p.level) {
                if overlap(&a.corners, &wall_box(wall)) {
                    out.push(format!("Prop {i} ({:?}) traverse le mur {wi}", p.kind));
                }
            }
        }
        for (i, p) in self.props.iter().enumerate() {
            if p.level != 0 && !self.on_floor(p.level, p.pos) {
                out.push(format!(
                    "Prop {i} ({:?}) flotte : aucun sol au level {} sous [{}, {}]",
                    p.kind, p.level, p.pos.x, p.pos.y
                ));
            }
        }
    }

    fn stairs_warnings(&self, out: &mut Vec<String>) {
        let stairs = self.props.iter().enumerate().filter(|(_, p)| p.kind.climbs());
        for (i, p) in stairs {
            let climb = (p.height / 3.0).round().max(1.0) as i32;
            for l in p.level + 1..=p.level + climb {
                if let Some(s) = self.slab_at(l, p.pos) {
                    out.push(format!(
                        "Prop {i} ({:?}) monte dans le sol {s} du level {l} : laisser une trémie",
                        p.kind
                    ));
                }
            }
            if p.kind == PropKind::Stairs {
                let (lo, hi) = p.local_box();
                let front = p.front();
                let top = p.pos + front * (hi.z - f32::midpoint(lo.z, hi.z) + 0.4);
                if !self.on_floor(p.level + climb, top) {
                    out.push(format!(
                        "Prop {i} (Stairs) arrive dans le vide : aucun sol au level {} en [{:.2}, {:.2}]",
                        p.level + climb,
                        top.x,
                        top.y
                    ));
                }
            }
        }
    }

    fn wall_warnings(&self, out: &mut Vec<String>) {
        for (i, w) in self.walls.iter().enumerate() {
            if let Some(j) = self.walls[..i].iter().position(|v| same_wall(v, w)) {
                out.push(format!("Wall {i} est en double avec le mur {j}"));
            }
            if w.level == 0 && [w.a, w.b, (w.a + w.b) / 2.0].iter().any(|&p| self.terrain.height(p) > 0.3) {
                out.push(format!("Wall {i} est enterré : le relief dépasse 30 cm à son pied"));
            }
            for (j, o) in w.openings.iter().enumerate() {
                if o.is_door() && w.level != 0 {
                    let at = w.a + w.dir() * o.t;
                    let n = w.dir().perp() * (w.thick / 2.0 + 0.4);
                    if !(self.on_floor(w.level, at + n) && self.on_floor(w.level, at - n)) {
                        out.push(format!("Opening [{i}, {j}] ({:?}) donne sur le vide : pas de sol d'un côté", o.kind));
                    }
                }
            }
        }
        for (i, r) in self.roofs.iter().enumerate() {
            let inside = |p: Vec2| p.cmpge(r.min - 0.2).all() && p.cmple(r.max + 0.2).all();
            if let Some(wi) = self.walls.iter().position(|w| w.level == r.level && inside(w.a) && inside(w.b)) {
                out.push(format!(
                    "Roof {i} est posé au pied du mur {wi} (même level {}) : un toit se pose au level au-dessus des murs",
                    r.level
                ));
            }
        }
    }

    fn outdoor_warnings(&self, out: &mut Vec<String>) {
        for (i, p) in self.pools.iter().enumerate() {
            if let Some(s) = self.slabs.iter().position(|s| s.level == 0 && rects_overlap(p.min, p.max, s.min, s.max)) {
                out.push(format!("Pool {i} est sous le sol {s}"));
            }
        }
        for (i, r) in self.roads.iter().enumerate() {
            if let Some(wi) = self.walls.iter().position(|w| w.level == 0 && segments_cross(r.a, r.b, w.a, w.b)) {
                out.push(format!("Road {i} traverse le mur {wi}"));
            }
        }
    }

    /// Un sol du level `l` contient-il le point ? Au rez-de-chaussée, le terrain suffit.
    fn on_floor(&self, l: i32, p: Vec2) -> bool {
        l == 0 || self.slab_at(l, p).is_some()
    }

    fn slab_at(&self, l: i32, p: Vec2) -> Option<usize> {
        self.slabs.iter().position(|s: &Slab| s.level == l && p.cmpgt(s.min).all() && p.cmplt(s.max).all())
    }
}

/// Objet d'intérieur vu comme un volume : emprise au sol et hauteurs occupées. Tapis, tracés
/// et objets d'extérieur n'en sont pas : ils passent sous les meubles ou suivent le relief.
pub(super) struct Solid {
    level: i32,
    corners: [Vec2; 4],
    ys: (f32, f32),
}

impl Solid {
    pub(super) fn of(p: &Prop) -> Option<Self> {
        Self::with(p, p.local_box())
    }

    /// Comme `of`, d'après la `local_box` de `p` déjà calculée.
    pub(super) fn with(p: &Prop, (lo, hi): (Vec3, Vec3)) -> Option<Self> {
        (!p.kind.outdoor() && !p.kind.flat()).then(|| Self {
            level: p.level,
            corners: p.corners_in((lo, hi)),
            ys: (lo.y, hi.y),
        })
    }

    /// Même étage, emprises qui se chevauchent, et l'un ne passe pas au-dessus de l'autre (le
    /// meuble haut de cuisine au-dessus du meuble bas).
    pub(super) fn clash(&self, o: &Self) -> bool {
        let stacked = self.ys.1 <= o.ys.0 + SLACK || o.ys.1 <= self.ys.0 + SLACK;
        self.level == o.level && !stacked && overlap(&self.corners, &o.corners)
    }
}

/// Emprise au sol d'un mur : rectangle de son épaisseur autour du segment.
pub(super) fn wall_box(w: &Wall) -> [Vec2; 4] {
    let n = w.dir().perp() * (w.thick / 2.0);
    [w.a + n, w.b + n, w.b - n, w.a - n]
}

/// Deux rectangles orientés (coins dans l'ordre) se chevauchent-ils de plus de `SLACK` ?
/// Théorème de l'axe séparateur : on projette sur les normales des côtés des deux rectangles.
pub(super) fn overlap(a: &[Vec2; 4], b: &[Vec2; 4]) -> bool {
    let axes = [a[1] - a[0], a[3] - a[0], b[1] - b[0], b[3] - b[0]];
    axes.iter().filter(|ax| ax.length_squared() > 1e-8).all(|ax| {
        let ax = ax.normalize();
        let span = |r: &[Vec2; 4]| {
            r.iter().map(|p| p.dot(ax)).fold((f32::MAX, f32::MIN), |(lo, hi), d| (lo.min(d), hi.max(d)))
        };
        let ((a0, a1), (b0, b1)) = (span(a), span(b));
        a1.min(b1) - a0.max(b0) > SLACK
    })
}

fn rects_overlap(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> bool {
    (a1.min(b1) - a0.max(b0)).cmpgt(Vec2::splat(0.1)).all()
}

fn same_wall(v: &Wall, w: &Wall) -> bool {
    let close = |p: Vec2, q: Vec2| p.distance(q) < 0.01;
    v.level == w.level && ((close(v.a, w.a) && close(v.b, w.b)) || (close(v.a, w.b) && close(v.b, w.a)))
}

/// Les segments se coupent-ils franchement (pas seulement du bout) ?
fn segments_cross(p: Vec2, p2: Vec2, q: Vec2, q2: Vec2) -> bool {
    let (r, s) = (p2 - p, q2 - q);
    let den = r.perp_dot(s);
    if den.abs() < 1e-6 {
        return false;
    }
    let t = (q - p).perp_dot(s) / den;
    let u = (q - p).perp_dot(r) / den;
    (0.01..0.99).contains(&t) && (0.01..0.99).contains(&u)
}

#[cfg(test)]
mod tests {
    use super::super::{LEVEL_H, Opening, OpeningKind, Pool, Road, Roof, RoofKind, WHITE, room_walls};
    use super::*;

    fn prop(kind: PropKind, pos: Vec2, level: i32, rot: f32) -> Prop {
        Prop { color: WHITE, texture: 0, ..Prop::new(kind, pos, level, rot) }
    }

    fn room(h: &mut House, lo: Vec2, hi: Vec2, level: i32) {
        h.walls.extend(room_walls(lo, hi, level, LEVEL_H, 0.2, WHITE));
        h.slabs.push(Slab { min: lo, max: hi, level, color: WHITE, texture: 0, tag: None });
    }

    fn has(h: &House, needle: &str) -> bool {
        let w = h.warnings();
        w.iter().any(|m| m.contains(needle)) || panic!("« {needle} » absent de {w:?}")
    }

    #[test]
    fn a_tidy_house_has_no_warning() {
        let mut h = House::default();
        room(&mut h, Vec2::ZERO, Vec2::new(8.0, 6.0), 0);
        h.props.push(prop(PropKind::Sofa, Vec2::new(4.0, 0.55), 0, 0.0));
        h.props.push(prop(PropKind::Bed, Vec2::new(2.0, 3.5), 0, 0.0));
        h.roofs.push(Roof {
            min: Vec2::ZERO,
            max: Vec2::new(8.0, 6.0),
            level: 1,
            kind: RoofKind::Gable,
            height: 2.5,
            overhang: 0.5,
            color: WHITE,
            texture: 0,
            tag: None,
        });
        assert_eq!(h.warnings(), Vec::<String>::new());
    }

    #[test]
    fn overlapping_floating_and_wall_crossing_props() {
        let mut h = House::default();
        room(&mut h, Vec2::ZERO, Vec2::new(8.0, 6.0), 0);
        h.props.push(prop(PropKind::Table, Vec2::new(3.0, 3.0), 0, 0.0));
        h.props.push(prop(PropKind::Bed, Vec2::new(3.5, 3.2), 0, 90.0));
        h.props.push(prop(PropKind::Wardrobe, Vec2::new(7.9, 2.0), 0, 270.0));
        h.props.push(prop(PropKind::Sofa, Vec2::new(4.0, 2.0), 1, 0.0));
        assert!(has(&h, "Prop 0 (Table) et Prop 1 (Bed) se chevauchent"));
        assert!(has(&h, "Prop 2 (Wardrobe) traverse le mur 1"));
        assert!(has(&h, "Prop 3 (Sofa) flotte"));
    }

    #[test]
    fn wall_units_above_furniture_do_not_overlap_it() {
        let mut h = House::default();
        room(&mut h, Vec2::ZERO, Vec2::new(8.0, 6.0), 0);
        h.props.push(prop(PropKind::BaseCabinet, Vec2::new(3.0, 0.4), 0, 0.0));
        h.props.push(prop(PropKind::WallCabinet, Vec2::new(3.0, 0.3), 0, 0.0));
        assert_eq!(h.warnings(), Vec::<String>::new());
        h.props.push(prop(PropKind::Fridge, Vec2::new(3.0, 0.5), 0, 0.0));
        assert!(has(&h, "Prop 0 (BaseCabinet) et Prop 2 (Fridge) se chevauchent"));
        assert!(
            has(&h, "Prop 1 (WallCabinet) et Prop 2 (Fridge) se chevauchent"),
            "le frigo monte jusqu'au meuble haut"
        );
    }

    #[test]
    fn stairs_need_a_hole_and_a_landing() {
        let mut h = House::default();
        room(&mut h, Vec2::ZERO, Vec2::new(8.0, 6.0), 0);
        h.slabs.push(Slab { min: Vec2::ZERO, max: Vec2::new(8.0, 6.0), level: 1, color: WHITE, texture: 0, tag: None });
        h.props.push(prop(PropKind::Stairs, Vec2::new(1.0, 3.0), 0, 180.0));
        assert!(has(&h, "monte dans le sol 1 du level 1"));
        // trémie au-dessus de l'escalier, palier au nord
        h.slabs[1] =
            Slab { min: Vec2::new(1.6, 0.0), max: Vec2::new(8.0, 6.0), level: 1, color: WHITE, texture: 0, tag: None };
        assert!(has(&h, "arrive dans le vide"));
        h.slabs.push(Slab { min: Vec2::ZERO, max: Vec2::new(1.6, 0.9), level: 1, color: WHITE, texture: 0, tag: None });
        assert!(!h.warnings().iter().any(|m| m.contains("Stairs")), "{:?}", h.warnings());
    }

    #[test]
    fn walls_roofs_doors_pools_and_roads() {
        let mut h = House::default();
        room(&mut h, Vec2::ZERO, Vec2::new(8.0, 6.0), 0);
        h.walls.push(h.walls[0].clone());
        h.roofs.push(Roof {
            min: Vec2::ZERO,
            max: Vec2::new(8.0, 6.0),
            level: 0,
            kind: RoofKind::Flat,
            height: 0.3,
            overhang: 0.5,
            color: WHITE,
            texture: 0,
            tag: None,
        });
        let mut up = h.walls[1].clone();
        up.level = 1;
        up.openings.push(Opening::new(OpeningKind::Door, 3.0));
        up.openings.push(Opening::new(OpeningKind::GarageDoor, 6.0));
        h.walls.push(up);
        h.pools.push(Pool {
            min: Vec2::new(2.0, 2.0),
            max: Vec2::new(4.0, 4.0),
            depth: 1.5,
            color: WHITE,
            texture: 0,
            tag: None,
        });
        h.roads.push(Road {
            a: Vec2::new(4.0, -5.0),
            b: Vec2::new(4.0, 10.0),
            width: 3.0,
            color: WHITE,
            texture: 0,
            tag: None,
        });
        assert!(has(&h, "Wall 4 est en double avec le mur 0"));
        assert!(has(&h, "Roof 0 est posé au pied du mur 0"));
        assert!(has(&h, "Opening [5, 0] (Door) donne sur le vide"));
        assert!(has(&h, "Opening [5, 1] (GarageDoor) donne sur le vide"), "une porte de garage aussi");
        assert!(has(&h, "Pool 0 est sous le sol 0"));
        assert!(has(&h, "Road 0 traverse le mur 0"));
    }

    #[test]
    fn buried_walls_are_reported() {
        let mut h = House::default();
        room(&mut h, Vec2::new(30.0, 30.0), Vec2::new(36.0, 34.0), 0);
        h.terrain.h = vec![2.0; crate::terrain::SIDE * crate::terrain::SIDE];
        assert!(has(&h, "est enterré"));
    }
}
