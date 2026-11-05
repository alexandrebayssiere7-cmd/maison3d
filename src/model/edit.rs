//! Modifications de la maison : étages, couleurs, suppression, déplacement, duplication.

use bevy::math::Vec2;

use super::{ElemRef, House, OpeningKind, PropKind, Rgb, Slab, Wall};

impl House {
    /// Insère un étage au-dessus de `l` : copie ses murs (sans les portes) et ses sols intérieurs,
    /// et remonte d'un cran tout ce qui était plus haut, toits compris. Avec `area`, seuls les
    /// éléments contenus dans ce rectangle (à 20 cm près) sont concernés : un seul bâtiment.
    pub fn add_storey(&mut self, l: i32, area: Option<(Vec2, Vec2)>) {
        let inside = |p: Vec2| area.is_none_or(|(lo, hi)| p.cmpge(lo - 0.2).all() && p.cmple(hi + 0.2).all());
        let up = |lv: &mut i32, pts: [Vec2; 2]| {
            if *lv > l && pts.into_iter().all(inside) {
                *lv += 1;
            }
        };
        self.walls.iter_mut().for_each(|w| up(&mut w.level, [w.a, w.b]));
        self.slabs.iter_mut().for_each(|s| up(&mut s.level, [s.min, s.max]));
        self.roofs.iter_mut().for_each(|r| up(&mut r.level, [r.min, r.max]));
        self.props.iter_mut().for_each(|p| up(&mut p.level, [p.pos, p.pos]));
        let walls: Vec<Wall> = self
            .walls
            .iter()
            .filter(|w| w.level == l && inside(w.a) && inside(w.b))
            .map(|w| {
                let mut w = w.clone();
                w.level += 1;
                w.openings.retain(|o| o.kind != OpeningKind::Door);
                w
            })
            .collect();
        let (lo, hi) =
            walls.iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), w| (lo.min(w.a.min(w.b)), hi.max(w.a.max(w.b))));
        let slabs: Vec<Slab> = self
            .slabs
            .iter()
            .filter(|s| s.level == l && s.min.cmpge(lo - 0.2).all() && s.max.cmple(hi + 0.2).all())
            .map(|s| Slab { level: l + 1, ..s.clone() })
            .collect();
        self.walls.extend(walls);
        self.slabs.extend(slabs);
    }

    pub fn texture_mut(&mut self, e: ElemRef) -> Option<&mut usize> {
        Some(match e {
            ElemRef::Wall(i) => &mut self.walls.get_mut(i)?.texture,
            ElemRef::Slab(i) => &mut self.slabs.get_mut(i)?.texture,
            ElemRef::Roof(i) => &mut self.roofs.get_mut(i)?.texture,
            ElemRef::Prop(i) => &mut self.props.get_mut(i)?.texture,
            ElemRef::Pool(i) => &mut self.pools.get_mut(i)?.texture,
            ElemRef::Road(i) => &mut self.roads.get_mut(i)?.texture,
            ElemRef::Opening(..) => return None,
        })
    }

    /// Peut-on l'ouvrir pendant la visite (porte, armoire, frigo, placards) ?
    pub fn openable(&self, e: ElemRef) -> bool {
        match e {
            ElemRef::Opening(w, o) => self
                .walls
                .get(w)
                .and_then(|w| w.openings.get(o))
                .is_some_and(|o| matches!(o.kind, OpeningKind::Door | OpeningKind::FrenchDoor)),
            ElemRef::Prop(i) => self
                .props
                .get(i)
                .is_some_and(|p| matches!(p.kind, PropKind::Wardrobe | PropKind::Fridge | PropKind::Kitchen)),
            _ => false,
        }
    }

    pub fn color_mut(&mut self, e: ElemRef) -> Option<&mut Rgb> {
        Some(match e {
            ElemRef::Wall(i) => &mut self.walls.get_mut(i)?.color,
            ElemRef::Opening(w, o) => &mut self.walls.get_mut(w)?.openings.get_mut(o)?.color,
            ElemRef::Slab(i) => &mut self.slabs.get_mut(i)?.color,
            ElemRef::Roof(i) => &mut self.roofs.get_mut(i)?.color,
            ElemRef::Prop(i) => &mut self.props.get_mut(i)?.color,
            ElemRef::Pool(i) => &mut self.pools.get_mut(i)?.color,
            ElemRef::Road(i) => &mut self.roads.get_mut(i)?.color,
        })
    }

    pub fn remove(&mut self, e: ElemRef) {
        match e {
            ElemRef::Wall(i) if i < self.walls.len() => drop(self.walls.remove(i)),
            ElemRef::Opening(w, o) => {
                if let Some(w) = self.walls.get_mut(w).filter(|w| o < w.openings.len()) {
                    w.openings.remove(o);
                }
            }
            ElemRef::Slab(i) if i < self.slabs.len() => drop(self.slabs.remove(i)),
            ElemRef::Roof(i) if i < self.roofs.len() => drop(self.roofs.remove(i)),
            ElemRef::Prop(i) if i < self.props.len() => drop(self.props.remove(i)),
            ElemRef::Pool(i) if i < self.pools.len() => drop(self.pools.remove(i)),
            ElemRef::Road(i) if i < self.roads.len() => drop(self.roads.remove(i)),
            _ => {}
        }
    }

    pub fn translate(&mut self, e: ElemRef, d: Vec2) {
        match e {
            ElemRef::Wall(i) => {
                if let Some(w) = self.walls.get_mut(i) {
                    w.a += d;
                    w.b += d;
                }
            }
            ElemRef::Opening(wi, oi) => {
                let Some(w) = self.walls.get_mut(wi) else { return };
                let Some(o) = w.openings.get(oi) else { return };
                let t = o.t + d.dot(w.dir());
                if w.fits(t, o.width, Some(oi)) {
                    w.openings[oi].t = t;
                }
            }
            ElemRef::Slab(i) => {
                if let Some(s) = self.slabs.get_mut(i) {
                    s.min += d;
                    s.max += d;
                }
            }
            ElemRef::Roof(i) => {
                if let Some(r) = self.roofs.get_mut(i) {
                    r.min += d;
                    r.max += d;
                }
            }
            ElemRef::Prop(i) => {
                if let Some(p) = self.props.get_mut(i) {
                    p.pos += d;
                }
            }
            ElemRef::Pool(i) => {
                if let Some(p) = self.pools.get_mut(i) {
                    // reste aligné sur la grille du relief pour que le trou tombe juste
                    p.min += d.round();
                    p.max += d.round();
                    self.settle_pools();
                }
            }
            ElemRef::Road(i) => {
                if let Some(r) = self.roads.get_mut(i) {
                    r.a += d;
                    r.b += d;
                }
            }
        }
    }

    /// Copie l'élément décalé de `d` ; renvoie la référence de la copie.
    pub fn duplicate(&mut self, e: ElemRef, d: Vec2) -> Option<ElemRef> {
        let new = match e {
            ElemRef::Wall(i) => {
                self.walls.push(self.walls.get(i)?.clone());
                ElemRef::Wall(self.walls.len() - 1)
            }
            ElemRef::Slab(i) => {
                self.slabs.push(self.slabs.get(i)?.clone());
                ElemRef::Slab(self.slabs.len() - 1)
            }
            ElemRef::Roof(i) => {
                self.roofs.push(self.roofs.get(i)?.clone());
                ElemRef::Roof(self.roofs.len() - 1)
            }
            ElemRef::Prop(i) => {
                self.props.push(self.props.get(i)?.clone());
                ElemRef::Prop(self.props.len() - 1)
            }
            ElemRef::Pool(i) => {
                self.pools.push(self.pools.get(i)?.clone());
                ElemRef::Pool(self.pools.len() - 1)
            }
            ElemRef::Road(i) => {
                self.roads.push(self.roads.get(i)?.clone());
                ElemRef::Road(self.roads.len() - 1)
            }
            ElemRef::Opening(..) => return None,
        };
        self.translate(new, d);
        Some(new)
    }

    pub fn floor_area(&self, level: i32) -> f32 {
        self.slabs.iter().filter(|s| s.level == level).map(|s| (s.max - s.min).abs().element_product()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    fn wall(len: f32) -> Wall {
        Wall {
            a: Vec2::ZERO,
            b: Vec2::new(len, 0.0),
            level: 0,
            height: 3.0,
            thick: 0.2,
            color: WHITE,
            openings: vec![],
            texture: 0,
        }
    }
    #[test]
    fn add_storey_copies_walls_and_lifts_roof() {
        let mut h = demo_house();
        let walls0 = h.walls.iter().filter(|w| w.level == 0).count();
        h.add_storey(0, None);
        assert_eq!(h.walls.iter().filter(|w| w.level == 1).count(), walls0);
        assert!(
            h.walls.iter().filter(|w| w.level == 1).all(|w| w.openings.iter().all(|o| o.kind != OpeningKind::Door))
        );
        assert_eq!(h.roofs[0].level, 2);
        // le parquet est copié, pas la terrasse
        assert_eq!(h.slabs.iter().filter(|s| s.level == 1).count(), 1);
    }
    #[test]
    fn add_storey_with_area_leaves_other_buildings_alone() {
        let mut h = House {
            walls: super::super::room_walls(Vec2::ZERO, Vec2::splat(4.0), 0, 3.0, 0.2, [1.0; 3]),
            ..House::default()
        };
        h.walls.extend(super::super::room_walls(Vec2::splat(10.0), Vec2::splat(14.0), 0, 3.0, 0.2, [1.0; 3]));
        h.add_storey(0, Some((Vec2::ZERO, Vec2::splat(4.0))));
        assert_eq!(h.walls.len(), 12);
        assert!(h.walls[8..].iter().all(|w| w.level == 1 && w.a.max_element() <= 4.0));
    }

    #[test]
    fn hide_upper_levels_and_edit_ops() {
        let mut h = demo_house();
        assert!(h.parts_closed(Some(0)).iter().all(|p| p.owner != ElemRef::Roof(0)));
        assert!(h.parts_closed(None).iter().any(|p| p.owner == ElemRef::Roof(0)));
        let n = h.props.len();
        let copy = h.duplicate(ElemRef::Prop(0), Vec2::X).unwrap();
        assert_eq!(copy, ElemRef::Prop(n));
        assert_eq!(h.props[n].pos, h.props[0].pos + Vec2::X);
        h.remove(ElemRef::Opening(0, 0));
        assert_eq!(h.walls[0].openings.len(), 2);
        // maison 10 × 8 + terrasse 14 × 4,87
        assert!((h.floor_area(0) - 148.18).abs() < 0.01);
    }
    #[test]
    fn remove_every_kind_and_ignore_bad_indices() {
        let mut h = demo_house();
        let before = h.clone();
        for e in
            [ElemRef::Wall(99), ElemRef::Opening(0, 99), ElemRef::Opening(99, 0), ElemRef::Slab(99), ElemRef::Pool(99)]
        {
            h.remove(e);
            h.translate(e, Vec2::X);
            assert!(h.color_mut(e).is_none());
        }
        assert_eq!(h, before, "un indice invalide ne touche à rien");
        let counts =
            |h: &House| (h.walls.len(), h.slabs.len(), h.roofs.len(), h.props.len(), h.pools.len(), h.roads.len());
        let (w, s, r, p, po, ro) = counts(&h);
        for e in
            [ElemRef::Wall(0), ElemRef::Slab(0), ElemRef::Roof(0), ElemRef::Prop(0), ElemRef::Pool(0), ElemRef::Road(0)]
        {
            h.remove(e);
        }
        assert_eq!(counts(&h), (w - 1, s - 1, r - 1, p - 1, po - 1, ro - 1));
    }
    #[test]
    fn openings_have_color_but_no_texture() {
        let mut h = demo_house();
        assert!(h.texture_mut(ElemRef::Opening(0, 0)).is_none());
        assert!(h.color_mut(ElemRef::Opening(0, 0)).is_some());
        for e in
            [ElemRef::Wall(0), ElemRef::Slab(0), ElemRef::Roof(0), ElemRef::Prop(0), ElemRef::Pool(0), ElemRef::Road(0)]
        {
            *h.texture_mut(e).unwrap() = 7;
        }
        assert_eq!((h.walls[0].texture, h.roads[0].texture), (7, 7));
        assert!(!h.openable(ElemRef::Wall(0)));
    }
    #[test]
    fn opening_moves_only_where_it_fits() {
        let mut h = House::default();
        h.walls.push(wall(4.0));
        h.walls[0].openings.push(Opening {
            kind: OpeningKind::Door,
            t: 2.0,
            width: 1.0,
            height: 2.1,
            sill: 0.0,
            color: WHITE,
        });
        h.translate(ElemRef::Opening(0, 0), Vec2::new(0.5, 3.0));
        assert_eq!(h.walls[0].openings[0].t, 2.5, "seule la composante le long du mur compte");
        h.translate(ElemRef::Opening(0, 0), Vec2::new(5.0, 0.0));
        assert_eq!(h.walls[0].openings[0].t, 2.5, "sortirait du mur : refusé");
    }
    #[test]
    fn pools_move_on_the_terrain_grid() {
        let mut h = demo_house();
        let min = h.pools[0].min;
        h.translate(ElemRef::Pool(0), Vec2::new(1.4, -0.6));
        assert_eq!(h.pools[0].min, min + Vec2::new(1.0, -1.0));
        let copy = h.duplicate(ElemRef::Pool(0), Vec2::new(0.0, 8.0)).unwrap();
        assert_eq!(h.pools[1].min, h.pools[0].min + Vec2::new(0.0, 8.0));
        assert_eq!(copy, ElemRef::Pool(1));
        assert!(h.duplicate(ElemRef::Opening(0, 0), Vec2::X).is_none());
        assert!(h.duplicate(ElemRef::Road(99), Vec2::X).is_none());
    }
}
