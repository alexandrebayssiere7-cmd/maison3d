//! Données de la maison (sérialisables) et leur conversion en pièces géométriques.

use bevy::math::{Quat, Vec2, Vec3};
use bevy::transform::components::Transform;
use serde::{Deserialize, Serialize};

use crate::terrain::Terrain;

/// Hauteur d'un étage (dalle comprise).
pub const LEVEL_H: f32 = 3.0;
pub type Rgb = [f32; 3];

pub fn elev(level: i32) -> f32 {
    level as f32 * LEVEL_H
}

const WHITE: Rgb = [0.95, 0.95, 0.93];
const DARK: Rgb = [0.15, 0.15, 0.17];
const METAL: Rgb = [0.7, 0.72, 0.75];
/// Blanc pur : une texture s'affiche avec ses vraies couleurs.
pub const WHITE_TEX: Rgb = [1.0, 1.0, 1.0];
/// Teinte des troncs (reçoivent automatiquement la texture d'écorce).
const BARK: Rgb = [0.95, 0.92, 0.9];
const BIRCH_BARK: Rgb = [0.96, 0.95, 0.94];
const LEAF: Rgb = [0.25, 0.5, 0.2];

// ---------------------------------------------------------------- éléments

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum OpeningKind {
    Door,
    Window,
    Bay,
    FrenchDoor,
    Fixed,
    Grid,
}

impl OpeningKind {
    pub const ALL: [OpeningKind; 6] = [Self::Door, Self::Window, Self::Bay, Self::FrenchDoor, Self::Fixed, Self::Grid];
    pub fn name(self) -> &'static str {
        match self {
            Self::Door => "Porte",
            Self::Window => "Fenêtre",
            Self::Bay => "Baie vitrée",
            Self::FrenchDoor => "Porte-fenêtre",
            Self::Fixed => "Vitre fixe",
            Self::Grid => "Fenêtre à carreaux",
        }
    }
    /// (largeur, hauteur, allège)
    pub fn default_dims(self) -> (f32, f32, f32) {
        match self {
            Self::Door => (0.9, 2.1, 0.0),
            Self::Window => (1.2, 1.2, 0.9),
            Self::Bay => (2.4, 2.2, 0.0),
            Self::FrenchDoor => (1.4, 2.15, 0.0),
            Self::Fixed => (2.0, 2.4, 0.3),
            Self::Grid => (1.0, 1.3, 0.9),
        }
    }
    pub fn default_color(self) -> Rgb {
        match self {
            Self::Door => [0.5, 0.33, 0.2],
            _ => WHITE,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Opening {
    pub kind: OpeningKind,
    /// Position du centre le long du mur, depuis `a`.
    pub t: f32,
    pub width: f32,
    pub height: f32,
    pub sill: f32,
    pub color: Rgb,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Wall {
    pub a: Vec2,
    pub b: Vec2,
    pub level: i32,
    pub height: f32,
    pub thick: f32,
    pub color: Rgb,
    pub openings: Vec<Opening>,
    #[serde(default)]
    pub texture: usize,
}

impl Wall {
    pub fn len(&self) -> f32 {
        self.a.distance(self.b)
    }
    pub fn dir(&self) -> Vec2 {
        (self.b - self.a).normalize_or_zero()
    }
    /// Repère local : origine en `a` au sol de l'étage, X le long du mur, Y vers le haut.
    pub fn frame(&self) -> Transform {
        let d = self.b - self.a;
        Transform::from_xyz(self.a.x, elev(self.level), self.a.y)
            .with_rotation(Quat::from_rotation_y(-d.y.atan2(d.x)))
    }
    /// Une ouverture de largeur `w` centrée en `t` tient-elle sans chevaucher les autres ?
    pub fn fits(&self, t: f32, w: f32, ignore: Option<usize>) -> bool {
        let gap = 0.05;
        t - w / 2.0 >= gap
            && t + w / 2.0 <= self.len() - gap
            && self.openings.iter().enumerate().all(|(i, o)| {
                Some(i) == ignore || (t - o.t).abs() >= (w + o.width) / 2.0 + gap
            })
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Slab {
    pub min: Vec2,
    pub max: Vec2,
    pub level: i32,
    pub color: Rgb,
    #[serde(default)]
    pub texture: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RoofKind {
    Gable,
    Hip,
    Flat,
    Shed,
    Pyramid,
    Mansard,
}

impl RoofKind {
    pub const ALL: [RoofKind; 6] = [Self::Gable, Self::Hip, Self::Flat, Self::Shed, Self::Pyramid, Self::Mansard];
    pub fn name(self) -> &'static str {
        match self {
            Self::Gable => "Deux pans",
            Self::Hip => "Quatre pans",
            Self::Flat => "Toit plat",
            Self::Shed => "Monopente",
            Self::Pyramid => "Pyramide",
            Self::Mansard => "Mansarde",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Roof {
    pub min: Vec2,
    pub max: Vec2,
    pub level: i32,
    pub kind: RoofKind,
    pub height: f32,
    pub overhang: f32,
    pub color: Rgb,
    #[serde(default)]
    pub texture: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PropKind {
    Table,
    Chair,
    Sofa,
    Bed,
    Wardrobe,
    Kitchen,
    Fridge,
    Bathtub,
    Toilet,
    Stairs,
    Pillar,
    Lamp,
    Plant,
    Tree,
    Bush,
    Pine,
    Hedge,
    Fence,
    SpiralStairs,
    LStairs,
    Birch,
    Palm,
    Cypress,
    AppleTree,
    Flowers,
    Rosebush,
    Sunflower,
}

impl PropKind {
    pub const ALL: [PropKind; 27] = [
        Self::Table,
        Self::Chair,
        Self::Sofa,
        Self::Bed,
        Self::Wardrobe,
        Self::Kitchen,
        Self::Fridge,
        Self::Bathtub,
        Self::Toilet,
        Self::Stairs,
        Self::Pillar,
        Self::Lamp,
        Self::Plant,
        Self::Tree,
        Self::Bush,
        Self::Pine,
        Self::Hedge,
        Self::Fence,
        Self::SpiralStairs,
        Self::LStairs,
        Self::Birch,
        Self::Palm,
        Self::Cypress,
        Self::AppleTree,
        Self::Flowers,
        Self::Rosebush,
        Self::Sunflower,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Table => "Table",
            Self::Chair => "Chaise",
            Self::Sofa => "Canapé",
            Self::Bed => "Lit",
            Self::Wardrobe => "Armoire",
            Self::Kitchen => "Cuisine",
            Self::Fridge => "Frigo",
            Self::Bathtub => "Baignoire",
            Self::Toilet => "WC",
            Self::Stairs => "Escalier",
            Self::Pillar => "Pilier",
            Self::Lamp => "Lampadaire",
            Self::Plant => "Plante",
            Self::Tree => "Arbre",
            Self::Bush => "Buisson",
            Self::Pine => "Sapin",
            Self::Hedge => "Haie",
            Self::Fence => "Clôture",
            Self::SpiralStairs => "Colimaçon",
            Self::LStairs => "Quart tournant",
            Self::Birch => "Bouleau",
            Self::Palm => "Palmier",
            Self::Cypress => "Cyprès",
            Self::AppleTree => "Pommier",
            Self::Flowers => "Fleurs",
            Self::Rosebush => "Rosier",
            Self::Sunflower => "Tournesols",
        }
    }
    /// Pousse dehors : posé sur le relief du terrain plutôt qu'à plat.
    pub fn outdoor(self) -> bool {
        use PropKind::*;
        matches!(self, Tree | Bush | Pine | Hedge | Fence | Birch | Palm | Cypress | AppleTree | Flowers | Rosebush | Sunflower)
    }
    /// Escaliers et piliers ont une hauteur réglable.
    pub fn has_height(self) -> bool {
        matches!(self, Self::Stairs | Self::SpiralStairs | Self::LStairs | Self::Pillar)
    }
    pub fn default_color(self) -> Rgb {
        match self {
            Self::Table | Self::Chair => [0.55, 0.37, 0.2],
            Self::Sofa => [0.3, 0.42, 0.62],
            Self::Bed => [0.45, 0.3, 0.2],
            Self::Wardrobe => [0.72, 0.56, 0.4],
            Self::Kitchen => [0.88, 0.88, 0.85],
            Self::Fridge | Self::Bathtub | Self::Toilet => WHITE,
            Self::Stairs | Self::LStairs | Self::Fence => [0.6, 0.45, 0.3],
            Self::SpiralStairs => [0.35, 0.36, 0.38],
            Self::Flowers => [0.85, 0.15, 0.25],
            Self::Rosebush => [0.95, 0.45, 0.6],
            Self::Sunflower => [0.98, 0.8, 0.1],
            Self::Pillar => [0.85, 0.85, 0.8],
            Self::Lamp => [1.0, 0.9, 0.7],
            // le feuillage est coloré par sa texture : teinte blanche
            Self::Plant | Self::Tree | Self::Bush | Self::Pine | Self::Hedge | Self::Birch | Self::Palm | Self::Cypress | Self::AppleTree => WHITE_TEX,
        }
    }
    pub fn default_texture(self) -> usize {
        use crate::textures::find;
        match self {
            Self::Tree | Self::Hedge | Self::AppleTree | Self::Palm => find("Feuillage"),
            Self::Bush | Self::Plant | Self::Birch => find("Feuillage clair"),
            Self::Pine | Self::Cypress => find("Aiguilles de pin"),
            _ => 0,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Prop {
    pub kind: PropKind,
    pub pos: Vec2,
    pub level: i32,
    /// Degrés autour de l'axe vertical.
    pub rot: f32,
    pub color: Rgb,
    #[serde(default)]
    pub texture: usize,
    /// Hauteur des escaliers et piliers.
    #[serde(default = "level_h")]
    pub height: f32,
}

fn level_h() -> f32 {
    LEVEL_H
}

/// Piscine creusée dans le terrain ; coins alignés sur la grille du relief (1 m).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pool {
    pub min: Vec2,
    pub max: Vec2,
    pub depth: f32,
    pub color: Rgb,
    pub texture: usize,
}

/// Tronçon de route posé sur le terrain (qui est terrassé dessous à sa création).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Road {
    pub a: Vec2,
    pub b: Vec2,
    pub width: f32,
    pub color: Rgb,
    pub texture: usize,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct House {
    pub walls: Vec<Wall>,
    pub slabs: Vec<Slab>,
    pub roofs: Vec<Roof>,
    pub props: Vec<Prop>,
    #[serde(default)]
    pub pools: Vec<Pool>,
    #[serde(default)]
    pub roads: Vec<Road>,
    #[serde(default)]
    pub terrain: Terrain,
}

/// Référence vers un élément de la maison (indices dans les `Vec`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ElemRef {
    Wall(usize),
    Opening(usize, usize),
    Slab(usize),
    Roof(usize),
    Prop(usize),
    Pool(usize),
    Road(usize),
}

// ---------------------------------------------------------------- pièces géométriques

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    Cube,
    Glass,
    Cylinder,
    Sphere,
    Cone,
    /// Toit en pente : `size` = (longueur, hauteur, largeur), base en y = 0.
    Roof { kind: RoofKind, size: Vec3 },
    Light,
}

/// Une primitive à afficher : cube/cylindre/sphère unitaire mis à l'échelle par `tf`.
#[derive(Clone, Debug)]
pub struct Part {
    pub tf: Transform,
    pub shape: Shape,
    pub color: Rgb,
    pub texture: usize,
    pub owner: ElemRef,
}

impl Part {
    /// Boîte englobante sous forme de cube unitaire transformé (sélection, aperçus).
    pub fn bounds(&self) -> Transform {
        match self.shape {
            Shape::Roof { size, .. } => self
                .tf
                .mul_transform(Transform::from_xyz(0.0, size.y / 2.0, 0.0).with_scale(size)),
            _ => self.tf,
        }
    }
}

fn cube(tf: Transform, color: Rgb, texture: usize, owner: ElemRef) -> Part {
    Part { tf, shape: Shape::Cube, color, texture, owner }
}

/// Ouverture (0 = fermé, 1 = ouvert) des portes et placards, pour l'animation de la visite.
pub type OpenFn<'a> = &'a dyn Fn(ElemRef) -> f32;

/// Boîte locale définie par ses bornes x, y et sa profondeur z centrée.
fn local_box(x0: f32, x1: f32, y0: f32, y1: f32, depth: f32) -> Option<Transform> {
    (x1 - x0 > 1e-3 && y1 - y0 > 1e-3).then(|| {
        Transform::from_xyz((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0)
            .with_scale(Vec3::new(x1 - x0, y1 - y0, depth))
    })
}

impl House {
    /// Toutes les pièces visibles ; `max_level` masque les étages supérieurs.
    pub fn parts(&self, max_level: Option<i32>, open: OpenFn) -> Vec<Part> {
        let shown = |l: i32| max_level.is_none_or(|m| l <= m);
        let mut out = Vec::new();
        for (i, w) in self.walls.iter().enumerate().filter(|(_, w)| shown(w.level)) {
            wall_parts(w, i, open, &mut out);
        }
        for (i, s) in self.slabs.iter().enumerate().filter(|(_, s)| shown(s.level)) {
            let c = (s.min + s.max) / 2.0;
            let size = s.max - s.min;
            let tf = Transform::from_xyz(c.x, elev(s.level) - 0.09, c.y)
                .with_scale(Vec3::new(size.x, 0.22, size.y));
            out.push(cube(tf, s.color, s.texture, ElemRef::Slab(i)));
        }
        for (i, r) in self.roofs.iter().enumerate().filter(|(_, r)| shown(r.level)) {
            out.push(roof_part(r, ElemRef::Roof(i)));
        }
        for (i, p) in self.props.iter().enumerate().filter(|(_, p)| shown(p.level)) {
            out.extend(prop_parts(p, ElemRef::Prop(i), open(ElemRef::Prop(i)), self.prop_y(p)));
        }
        if shown(0) {
            for (i, p) in self.pools.iter().enumerate() {
                pool_parts(p, self.terrain.height(p.min), ElemRef::Pool(i), &mut out);
            }
            for (i, r) in self.roads.iter().enumerate() {
                out.extend(road_part(r, &self.terrain, ElemRef::Road(i)));
            }
        }
        out
    }

    /// Altitude de base d'un objet : les végétaux du rez-de-chaussée suivent le relief.
    pub fn prop_y(&self, p: &Prop) -> f32 {
        if p.level == 0 && p.kind.outdoor() { self.terrain.height(p.pos) } else { elev(p.level) }
    }

    /// Altitude du sol nu ; dans une piscine, très bas : c'est son fond qui porte.
    pub fn ground(&self, p: Vec2) -> f32 {
        if self.pools.iter().any(|q| p.cmpgt(q.min).all() && p.cmplt(q.max).all()) { -50.0 } else { self.terrain.height(p) }
    }

    /// Le fond de chaque piscine reste plat, à l'altitude du terrain à son coin.
    fn settle_pools(&mut self) {
        for p in &self.pools {
            self.terrain.flatten_rect(p.min, p.max, self.terrain.height(p.min));
        }
    }

    pub fn add_pool(&mut self, p: Pool) {
        self.terrain.flatten_rect(p.min, p.max, self.terrain.height((p.min + p.max) / 2.0));
        self.pools.push(p);
    }

    pub fn add_road(&mut self, r: Road) {
        self.terrain.grade(r.a, r.b, r.width);
        self.roads.push(r);
    }

    pub fn sculpt(&mut self, mode: crate::terrain::Brush, c: Vec2, r: f32, amount: f32, target: f32, seed: u32) {
        self.terrain.brush(mode, c, r, amount, target, seed);
        self.settle_pools();
    }

    /// Relief aléatoire autour de la maison, qui reste à plat ; routes et piscines sont reterrassées.
    pub fn generate_terrain(&mut self, seed: u32, amp: f32, scale: f32, mountains: bool) {
        let pts = self.walls.iter().flat_map(|w| [w.a, w.b]).chain(self.slabs.iter().flat_map(|s| [s.min, s.max]));
        let keep = pts.fold(None, |acc: Option<(Vec2, Vec2)>, p| Some(acc.map_or((p, p), |(lo, hi)| (lo.min(p), hi.max(p)))));
        self.terrain.generate(seed, amp, scale, mountains, keep);
        for r in &self.roads {
            self.terrain.grade(r.a, r.b, r.width);
        }
        self.settle_pools();
    }

    /// Insère un étage au-dessus de `l` : copie ses murs (sans les portes) et ses sols intérieurs,
    /// et remonte d'un cran tout ce qui était plus haut, toits compris.
    pub fn add_storey(&mut self, l: i32) {
        let up = |lv: &mut i32| {
            if *lv > l {
                *lv += 1;
            }
        };
        self.walls.iter_mut().for_each(|w| up(&mut w.level));
        self.slabs.iter_mut().for_each(|s| up(&mut s.level));
        self.roofs.iter_mut().for_each(|r| up(&mut r.level));
        self.props.iter_mut().for_each(|p| up(&mut p.level));
        let walls: Vec<Wall> = self
            .walls
            .iter()
            .filter(|w| w.level == l)
            .map(|w| {
                let mut w = w.clone();
                w.level += 1;
                w.openings.retain(|o| o.kind != OpeningKind::Door);
                w
            })
            .collect();
        let (lo, hi) = walls.iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), w| (lo.min(w.a.min(w.b)), hi.max(w.a.max(w.b))));
        let slabs: Vec<Slab> = self
            .slabs
            .iter()
            .filter(|s| s.level == l && s.min.cmpge(lo - 0.2).all() && s.max.cmple(hi + 0.2).all())
            .map(|s| Slab { level: l + 1, ..s.clone() })
            .collect();
        self.walls.extend(walls);
        self.slabs.extend(slabs);
    }

    /// Pièces avec tout fermé (aperçus, éditeur).
    pub fn parts_closed(&self, max_level: Option<i32>) -> Vec<Part> {
        self.parts(max_level, &|_| 0.0)
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
        self.slabs
            .iter()
            .filter(|s| s.level == level)
            .map(|s| (s.max - s.min).abs().element_product())
            .sum()
    }
}

/// Les quatre murs d'une pièce rectangulaire.
pub fn room_walls(min: Vec2, max: Vec2, level: i32, height: f32, thick: f32, color: Rgb) -> Vec<Wall> {
    let c = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];
    (0..4)
        .map(|i| Wall {
            a: c[i],
            b: c[(i + 1) % 4],
            level,
            height,
            thick,
            color,
            openings: vec![],
            texture: 0,
        })
        .collect()
}

fn wall_parts(w: &Wall, wi: usize, open: OpenFn, out: &mut Vec<Part>) {
    let len = w.len();
    if len < 1e-3 {
        return;
    }
    let frame = w.frame();
    let (h, th) = (w.height, w.thick);
    let owner = ElemRef::Wall(wi);
    let mut push = |b: Option<Transform>, shape: Shape, color: Rgb, owner: ElemRef| {
        if let Some(b) = b {
            let texture = if owner == ElemRef::Wall(wi) { w.texture } else { 0 };
            out.push(Part { tf: frame.mul_transform(b), shape, color, texture, owner });
        }
    };
    let mut order: Vec<usize> = (0..w.openings.len()).collect();
    order.sort_by(|&i, &j| w.openings[i].t.total_cmp(&w.openings[j].t));
    // les murs dépassent de th/2 à chaque bout pour que les angles se referment
    let mut x = -th / 2.0;
    for oi in order {
        let o = &w.openings[oi];
        let (x0, x1) = (o.t - o.width / 2.0, o.t + o.width / 2.0);
        let top = (o.sill + o.height).min(h);
        push(local_box(x, x0, 0.0, h, th), Shape::Cube, w.color, owner);
        push(local_box(x0, x1, 0.0, o.sill, th), Shape::Cube, w.color, owner);
        push(local_box(x0, x1, top, h, th), Shape::Cube, w.color, owner);
        x = x1;

        let own = ElemRef::Opening(wi, oi);
        let f = 0.07;
        let fd = th + 0.04;
        let frame_color = if o.kind == OpeningKind::Door { WHITE } else { o.color };
        push(local_box(x0, x0 + f, o.sill, top, fd), Shape::Cube, frame_color, own);
        push(local_box(x1 - f, x1, o.sill, top, fd), Shape::Cube, frame_color, own);
        push(local_box(x0, x1, top - f, top, fd), Shape::Cube, frame_color, own);
        match o.kind {
            OpeningKind::FrenchDoor => {
                // deux battants vitrés qui s'ouvrent vers l'intérieur
                let pw = (x1 - x0 - 2.0 * f) / 2.0;
                for (hx, side) in [(x0 + f, 1.0), (x1 - f, -1.0)] {
                    let hinge = Transform::from_xyz(hx, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-side * open(own) * 1.6));
                    let mut leaf = |a: f32, b: f32, y0: f32, y1: f32, d: f32, shape: Shape| {
                        let (a, b) = if side > 0.0 { (a, b) } else { (-b, -a) };
                        push(local_box(a, b, y0, y1, d).map(|b| hinge.mul_transform(b)), shape, o.color, own);
                    };
                    leaf(0.0, f, 0.0, top - f, 0.06, Shape::Cube);
                    leaf(pw - f, pw, 0.0, top - f, 0.06, Shape::Cube);
                    leaf(0.0, pw, top - 2.0 * f, top - f, 0.06, Shape::Cube);
                    leaf(0.0, pw, 0.0, 0.15, 0.06, Shape::Cube);
                    leaf(f, pw - f, 0.15, top - 2.0 * f, 0.02, Shape::Glass);
                }
            }
            OpeningKind::Door => {
                // le battant pivote autour de sa charnière (montant gauche)
                let hinge = Transform::from_xyz(x0 + f, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-open(own) * 1.6));
                let pw = x1 - x0 - 2.0 * f;
                push(local_box(0.0, pw, 0.0, top - f, 0.05).map(|b| hinge.mul_transform(b)), Shape::Cube, o.color, own);
                let knob = local_box(pw - 0.13, pw - 0.06, 0.98, 1.04, 0.14).map(|b| hinge.mul_transform(b));
                push(knob, Shape::Cube, METAL, own);
            }
            OpeningKind::Window | OpeningKind::Bay | OpeningKind::Fixed | OpeningKind::Grid => {
                push(local_box(x0, x1, o.sill, o.sill + f, fd), Shape::Cube, frame_color, own);
                if o.sill > 0.0 {
                    let sill = local_box(x0 - 0.05, x1 + 0.05, o.sill - 0.04, o.sill, th + 0.12);
                    push(sill, Shape::Cube, WHITE, own);
                }
                if o.kind == OpeningKind::Bay {
                    let m = o.t;
                    push(local_box(m - f / 2.0, m + f / 2.0, o.sill, top, fd), Shape::Cube, frame_color, own);
                }
                if o.kind == OpeningKind::Grid {
                    // petits bois : un montant et deux traverses
                    let (m, b) = (o.t, 0.015);
                    push(local_box(m - b, m + b, o.sill, top, 0.06), Shape::Cube, frame_color, own);
                    for k in [1.0, 2.0] {
                        let y = o.sill + f + (top - o.sill - 2.0 * f) * k / 3.0;
                        push(local_box(x0, x1, y - b, y + b, 0.06), Shape::Cube, frame_color, own);
                    }
                }
                push(local_box(x0 + f, x1 - f, o.sill + f, top - f, 0.02), Shape::Glass, o.color, own);
            }
        }
    }
    push(local_box(x, len + th / 2.0, 0.0, h, th), Shape::Cube, w.color, owner);
}

fn roof_part(r: &Roof, owner: ElemRef) -> Part {
    let min = r.min.min(r.max) - Vec2::splat(r.overhang);
    let max = r.min.max(r.max) + Vec2::splat(r.overhang);
    let (c, size) = ((min + max) / 2.0, max - min);
    let base = elev(r.level);
    if r.kind == RoofKind::Flat {
        let tf = Transform::from_xyz(c.x, base + 0.15, c.y).with_scale(Vec3::new(size.x, 0.3, size.y));
        return cube(tf, r.color, r.texture, owner);
    }
    // le faîtage suit toujours le grand côté
    let (long, short, rot) = if size.x >= size.y {
        (size.x, size.y, 0.0)
    } else {
        (size.y, size.x, std::f32::consts::FRAC_PI_2)
    };
    Part {
        tf: Transform::from_xyz(c.x, base, c.y).with_rotation(Quat::from_rotation_y(rot)),
        shape: Shape::Roof { kind: r.kind, size: Vec3::new(long, r.height, short) },
        color: r.color,
        texture: r.texture,
        owner,
    }
}

/// Boîte entre deux coins au sol et deux altitudes.
fn boxed(a: Vec2, b: Vec2, y0: f32, y1: f32) -> Transform {
    let (c, s) = ((a + b) / 2.0, b - a);
    Transform::from_xyz(c.x, (y0 + y1) / 2.0, c.y).with_scale(Vec3::new(s.x, y1 - y0, s.y))
}

/// Bassin (fond, parois, marches), eau et margelle ; `top` = altitude du terrain au bord.
fn pool_parts(p: &Pool, top: f32, owner: ElemRef, out: &mut Vec<Part>) {
    let (min, max) = (p.min, p.max);
    let bottom = top - p.depth;
    let (t, cop) = (0.2, 0.35);
    out.push(cube(boxed(min, max, bottom - 0.2, bottom), p.color, p.texture, owner));
    for (a, b) in [
        (min, Vec2::new(max.x, min.y + t)),
        (Vec2::new(min.x, max.y - t), max),
        (min, Vec2::new(min.x + t, max.y)),
        (Vec2::new(max.x - t, min.y), max),
    ] {
        out.push(cube(boxed(a, b, bottom, top), p.color, p.texture, owner));
    }
    let stone = [0.9, 0.87, 0.8];
    for (a, b) in [
        (min - cop, Vec2::new(max.x + cop, min.y)),
        (Vec2::new(min.x - cop, max.y), max + cop),
        (Vec2::new(min.x - cop, min.y), Vec2::new(min.x, max.y)),
        (Vec2::new(max.x, min.y), Vec2::new(max.x + cop, max.y)),
    ] {
        out.push(cube(boxed(a, b, top - 0.02, top + 0.06), stone, 0, owner));
    }
    // marches pour sortir du bassin, côté x min
    let n = (((max.x - min.x - 2.0 * t) / 0.35) as usize).min(4);
    let w = (max.y - min.y - 2.0 * t).min(1.4);
    for i in 0..n {
        let x = min.x + t + i as f32 * 0.35;
        let y = bottom + p.depth * (n - i) as f32 / (n + 1) as f32;
        out.push(cube(boxed(Vec2::new(x, min.y + t), Vec2::new(x + 0.35, min.y + t + w), bottom, y), WHITE, 0, owner));
    }
    let water = boxed(min + t, max - t, top - 0.17, top - 0.15);
    out.push(Part { tf: water, shape: Shape::Glass, color: p.color, texture: 0, owner });
}

/// Ruban incliné d'un bout à l'autre, au ras du terrain.
fn road_part(r: &Road, terrain: &Terrain, owner: ElemRef) -> Option<Part> {
    let d = r.b - r.a;
    let len = d.length();
    if len < 1e-3 {
        return None;
    }
    let (ya, yb) = (terrain.height(r.a), terrain.height(r.b));
    let c = (r.a + r.b) / 2.0;
    let rot = Quat::from_rotation_y(-d.y.atan2(d.x)) * Quat::from_rotation_z((yb - ya).atan2(len));
    // déborde d'une demi-largeur à chaque bout : les virages se referment
    let long = Vec2::new(len, yb - ya).length() + r.width;
    let tf = Transform::from_xyz(c.x, (ya + yb) / 2.0 + 0.02, c.y).with_rotation(rot).with_scale(Vec3::new(long, 0.08, r.width));
    Some(cube(tf, r.color, r.texture, owner))
}

/// Primitive locale d'un meuble : forme, centre, taille, couleur fixe (`None` = couleur du meuble),
/// rotation autour de la verticale (radians).
type Piece = (Shape, Vec3, Vec3, Option<Rgb>, f32);

/// Primitives d'un meuble, la face avant vers +z ; `open` (0..1) ouvre ses portes.
/// Boules de feuillage (centre, taille) à la couleur/texture du végétal.
fn clumps(c: &[([f32; 3], [f32; 3])]) -> Vec<Piece> {
    c.iter().map(|&(p, s)| (Shape::Sphere, Vec3::from(p), Vec3::from(s), None, 0.0)).collect()
}

fn prop_shapes(kind: PropKind, open: f32, height: f32) -> Vec<Piece> {
    // boîte posée entre y0 et y1
    let bx = |x: f32, z: f32, sx: f32, sz: f32, y0: f32, y1: f32, c: Option<Rgb>| -> Piece {
        (Shape::Cube, Vec3::new(x, (y0 + y1) / 2.0, z), Vec3::new(sx, y1 - y0, sz), c, 0.0)
    };
    // porte de largeur w accrochée en (hx, hz) ; side = 1 si elle s'étend vers +x depuis la charnière
    let door = |hx: f32, hz: f32, w: f32, y0: f32, y1: f32, side: f32| -> [Piece; 2] {
        let rot = -side * open * 1.7;
        let q = Quat::from_rotation_y(rot);
        let at = |x: f32, z: f32| Vec3::new(hx, (y0 + y1) / 2.0, hz) + q * Vec3::new(side * x, 0.0, z);
        [
            (Shape::Cube, at(w / 2.0, 0.02), Vec3::new(w - 0.01, y1 - y0, 0.03), None, rot),
            (Shape::Cube, at(w - 0.07, 0.06), Vec3::new(0.025, 0.2_f32.min(y1 - y0 - 0.1), 0.04), Some(METAL), rot),
        ]
    };
    // caisson ouvert à l'avant (pour voir l'intérieur quand les portes s'ouvrent)
    let carcass = |w: f32, d: f32, h: f32, inside: Option<Rgb>| -> [Piece; 5] {
        let t = 0.025;
        [
            bx(0.0, -d / 2.0 + t / 2.0, w, t, 0.0, h, None),
            bx(-w / 2.0 + t / 2.0, 0.0, t, d, 0.0, h, None),
            bx(w / 2.0 - t / 2.0, 0.0, t, d, 0.0, h, None),
            bx(0.0, 0.0, w, d, h - t, h, None),
            bx(0.0, 0.0, w - 2.0 * t, d - t, 0.0, 0.08, inside),
        ]
    };
    let legs = |hx: f32, hz: f32, h: f32| {
        [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sz)| bx(sx * hx, sz * hz, 0.05, 0.05, 0.0, h, None))
    };
    // marches d'environ 20 cm
    let n = ((height / 0.2).round() as usize).max(1);
    let rise = height / n as f32;
    let mut v = Vec::new();
    match kind {
        PropKind::Table => {
            v.push(bx(0.0, 0.0, 1.6, 0.9, 0.72, 0.77, None));
            v.extend(legs(0.72, 0.38, 0.72));
        }
        PropKind::Chair => {
            v.push(bx(0.0, 0.0, 0.45, 0.45, 0.43, 0.48, None));
            v.push(bx(0.0, -0.2, 0.45, 0.05, 0.48, 0.95, None));
            v.extend(legs(0.19, 0.19, 0.43));
        }
        PropKind::Sofa => {
            v.push(bx(0.0, 0.0, 2.0, 0.9, 0.1, 0.45, None));
            v.push(bx(0.0, -0.35, 2.0, 0.2, 0.45, 0.85, None));
            v.push(bx(-0.9, 0.0, 0.2, 0.9, 0.45, 0.65, None));
            v.push(bx(0.9, 0.0, 0.2, 0.9, 0.45, 0.65, None));
            v.extend(legs(0.9, 0.38, 0.1).map(|(s, c, z, _, r)| (s, c, z, Some(DARK), r)));
        }
        PropKind::Bed => {
            v.push(bx(0.0, 0.0, 1.6, 2.0, 0.1, 0.35, None));
            v.push(bx(0.0, 0.05, 1.5, 1.85, 0.35, 0.55, Some(WHITE)));
            v.push(bx(0.0, -0.98, 1.6, 0.06, 0.1, 1.0, None));
            v.push(bx(-0.38, -0.75, 0.6, 0.3, 0.55, 0.65, Some(WHITE)));
            v.push(bx(0.38, -0.75, 0.6, 0.3, 0.55, 0.65, Some(WHITE)));
        }
        PropKind::Wardrobe => {
            v.extend(carcass(1.2, 0.6, 2.0, None));
            v.push(bx(0.0, 0.0, 1.15, 0.55, 1.5, 1.53, None));
            v.push(bx(0.0, -0.05, 1.15, 0.03, 1.38, 1.4, Some(METAL)));
            // linge plié et vêtements suspendus
            v.push(bx(-0.3, 0.0, 0.35, 0.35, 1.53, 1.68, Some([0.7, 0.3, 0.3])));
            v.push(bx(0.25, 0.0, 0.35, 0.35, 1.53, 1.65, Some([0.3, 0.4, 0.7])));
            for (i, c) in [[0.2, 0.25, 0.4], [0.85, 0.85, 0.8], [0.4, 0.5, 0.3], [0.6, 0.2, 0.25]].into_iter().enumerate() {
                v.push(bx(-0.35 + i as f32 * 0.22, -0.05, 0.05, 0.45, 0.6, 1.38, Some(c)));
            }
            v.extend(door(-0.6, 0.3, 0.6, 0.02, 1.98, 1.0));
            v.extend(door(0.6, 0.3, 0.6, 0.02, 1.98, -1.0));
        }
        PropKind::Kitchen => {
            v.extend(carcass(2.4, 0.6, 0.86, None));
            v.push(bx(0.0, 0.0, 2.35, 0.55, 0.42, 0.44, None));
            v.push(bx(0.0, 0.0, 2.42, 0.62, 0.86, 0.9, Some(DARK)));
            v.push(bx(0.5, 0.0, 0.6, 0.4, 0.9, 0.91, Some(METAL)));
            v.push(bx(-0.6, 0.0, 0.6, 0.5, 0.9, 0.91, Some([0.05, 0.05, 0.05])));
            // vaisselle
            v.push(bx(-0.8, 0.0, 0.3, 0.3, 0.44, 0.56, Some(WHITE)));
            v.push(bx(0.3, -0.05, 0.4, 0.25, 0.08, 0.3, Some([0.6, 0.3, 0.2])));
            for (hx, side) in [(-1.2, 1.0), (0.0, -1.0), (0.0, 1.0), (1.2, -1.0)] {
                v.extend(door(hx, 0.3, 0.6, 0.06, 0.84, side));
            }
        }
        PropKind::Fridge => {
            v.extend(carcass(0.7, 0.7, 1.85, Some(WHITE)));
            for y in [0.5, 0.9, 1.3] {
                v.push(bx(0.0, 0.0, 0.64, 0.6, y, y + 0.02, Some([0.8, 0.9, 0.95])));
            }
            v.push(bx(-0.15, 0.0, 0.15, 0.15, 0.92, 1.1, Some([0.9, 0.5, 0.1])));
            v.push(bx(0.12, 0.05, 0.25, 0.12, 1.32, 1.4, Some([0.95, 0.9, 0.6])));
            v.push(bx(0.0, 0.0, 0.2, 0.15, 0.52, 0.75, Some([0.3, 0.6, 0.3])));
            v.extend(door(-0.35, 0.35, 0.7, 0.02, 1.84, 1.0));
        }
        PropKind::Bathtub => {
            v.push(bx(0.0, 0.0, 0.75, 1.7, 0.0, 0.55, None));
            v.push(bx(0.0, 0.0, 0.6, 1.5, 0.5, 0.56, Some([0.55, 0.75, 0.9])));
        }
        PropKind::Toilet => {
            v.push(bx(0.0, 0.05, 0.4, 0.5, 0.0, 0.42, None));
            v.push(bx(0.0, -0.22, 0.4, 0.18, 0.42, 0.8, None));
        }
        PropKind::Stairs => {
            let run = 0.28;
            for i in 0..n {
                let z = -(n as f32) * run / 2.0 + (i as f32 + 0.5) * run;
                v.push(bx(0.0, z, 1.0, run, 0.0, (i + 1) as f32 * rise, None));
            }
        }
        PropKind::SpiralStairs => {
            v.push((Shape::Cylinder, Vec3::new(0.0, (height + 1.0) / 2.0, 0.0), Vec3::new(0.16, height + 1.0, 0.16), None, 0.0));
            for i in 0..n {
                let a = i as f32 * 0.5;
                let top = (i + 1) as f32 * rise;
                let dir = Vec3::new(a.cos(), 0.0, -a.sin());
                v.push((Shape::Cube, dir * 0.55 + Vec3::Y * (top - 0.03), Vec3::new(0.9, 0.06, 0.42), None, a));
            }
        }
        PropKind::LStairs => {
            // première volée vers le palier (centré sur l'origine), seconde volée vers +x
            let run = 0.28;
            let n1 = n / 2;
            for i in 0..n1 {
                v.push(bx(0.0, -0.5 - (n1 - i) as f32 * run + run / 2.0, 1.0, run, 0.0, (i + 1) as f32 * rise, None));
            }
            v.push(bx(0.0, 0.0, 1.0, 1.0, 0.0, (n1 + 1) as f32 * rise, None));
            for j in 0..n.saturating_sub(n1 + 1) {
                v.push(bx(0.5 + (j as f32 + 0.5) * run, 0.0, run, 1.0, 0.0, (n1 + 2 + j) as f32 * rise, None));
            }
        }
        PropKind::Pillar => v.push(bx(0.0, 0.0, 0.3, 0.3, 0.0, height, None)),
        PropKind::Lamp => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.3, 0.04, 0.3), Some(DARK), 0.0));
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.04, 1.6, 0.04), Some(DARK), 0.0));
            v.push((Shape::Sphere, Vec3::new(0.0, 1.7, 0.0), Vec3::new(0.4, 0.3, 0.4), None, 0.0));
            v.push((Shape::Light, Vec3::new(0.0, 1.5, 0.0), Vec3::ONE, None, 0.0));
        }
        PropKind::Plant => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.2, 0.0), Vec3::new(0.35, 0.4, 0.35), Some([0.7, 0.35, 0.2]), 0.0));
            v.extend(clumps(&[([0.0, 0.62, 0.0], [0.5, 0.45, 0.5]), ([0.08, 0.8, 0.05], [0.35, 0.35, 0.35]), ([-0.1, 0.72, -0.06], [0.32, 0.3, 0.32])]));
        }
        PropKind::Tree => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 1.3, 0.0), Vec3::new(0.32, 2.6, 0.32), Some(BARK), 0.0));
            // couronne en grappes de boules de tailles variées, pour une silhouette irrégulière
            v.extend(clumps(&[
                ([0.0, 3.3, 0.0], [2.6, 2.3, 2.6]),
                ([0.9, 2.9, 0.3], [1.8, 1.6, 1.8]),
                ([-0.8, 3.0, -0.4], [1.9, 1.7, 1.9]),
                ([0.2, 4.1, -0.5], [1.7, 1.5, 1.7]),
                ([-0.3, 4.2, 0.6], [1.6, 1.4, 1.6]),
                ([0.5, 2.6, -0.9], [1.4, 1.2, 1.4]),
                ([-0.6, 2.7, 0.9], [1.3, 1.2, 1.3]),
                ([0.0, 4.8, 0.0], [1.1, 1.0, 1.1]),
            ]));
        }
        PropKind::Bush => v.extend(clumps(&[
            ([0.0, 0.45, 0.0], [1.1, 0.9, 1.1]),
            ([0.4, 0.4, 0.3], [0.8, 0.75, 0.8]),
            ([-0.35, 0.38, -0.25], [0.85, 0.7, 0.8]),
            ([-0.2, 0.42, 0.4], [0.7, 0.65, 0.7]),
            ([0.1, 0.72, -0.1], [0.7, 0.6, 0.7]),
        ])),
        PropKind::Pine => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.7, 0.0), Vec3::new(0.3, 1.4, 0.3), Some(BARK), 0.0));
            for (y, r, h) in [(1.7, 2.8, 1.9), (2.7, 2.2, 1.8), (3.6, 1.6, 1.6), (4.4, 1.0, 1.3), (5.0, 0.5, 0.9)] {
                v.push((Shape::Cone, Vec3::new(0.0, y, 0.0), Vec3::new(r, h, r), None, 0.0));
            }
        }
        PropKind::Hedge => {
            v.push(bx(0.0, 0.0, 2.0, 0.7, 0.0, 1.2, None));
            v.extend(clumps(&[([-0.6, 1.2, 0.0], [0.9, 0.45, 0.75]), ([0.0, 1.25, 0.0], [0.95, 0.45, 0.75]), ([0.6, 1.2, 0.0], [0.9, 0.45, 0.75])]));
        }
        PropKind::Fence => {
            for x in [-1.0, 0.0, 1.0] {
                v.push(bx(x, 0.0, 0.08, 0.08, 0.0, 1.0, None));
            }
            v.push(bx(0.0, 0.0, 2.0, 0.04, 0.35, 0.45, None));
            v.push(bx(0.0, 0.0, 2.0, 0.04, 0.75, 0.85, None));
        }
        PropKind::Birch => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.22, 4.0, 0.22), Some(BIRCH_BARK), 0.0));
            v.extend(clumps(&[
                ([0.0, 4.2, 0.0], [1.6, 2.6, 1.6]),
                ([0.45, 3.6, 0.2], [1.2, 1.8, 1.2]),
                ([-0.4, 3.8, -0.25], [1.2, 2.0, 1.2]),
                ([0.1, 5.3, 0.1], [1.0, 1.6, 1.0]),
            ]));
        }
        PropKind::Palm => {
            // tronc en tronçons, légèrement penché, et palmes en étoile
            for i in 0..6 {
                let k = i as f32;
                v.push((Shape::Cylinder, Vec3::new(k * 0.06, 0.4 + k * 0.8, 0.0), Vec3::new(0.34 - k * 0.02, 0.85, 0.34 - k * 0.02), Some(BARK), 0.0));
            }
            let top = Vec3::new(0.36, 4.9, 0.0);
            for k in 0..7 {
                let a = k as f32 * std::f32::consts::TAU / 7.0;
                v.push((Shape::Sphere, top + Vec3::new(a.cos(), -0.15, -a.sin()) * 1.1, Vec3::new(2.4, 0.12, 0.55), None, a));
            }
            v.push((Shape::Sphere, top, Vec3::splat(0.5), Some([0.45, 0.3, 0.15]), 0.0));
        }
        PropKind::Cypress => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.25, 0.6, 0.25), Some(BARK), 0.0));
            v.extend(clumps(&[([0.0, 3.2, 0.0], [1.3, 5.6, 1.3]), ([0.0, 5.7, 0.0], [0.7, 1.6, 0.7]), ([0.1, 1.8, 0.1], [1.1, 2.6, 1.1])]));
        }
        PropKind::AppleTree => {
            v.push((Shape::Cylinder, Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.26, 1.8, 0.26), Some(BARK), 0.0));
            v.extend(clumps(&[
                ([0.0, 2.6, 0.0], [2.6, 1.9, 2.6]),
                ([0.8, 2.3, 0.3], [1.5, 1.3, 1.5]),
                ([-0.7, 2.4, -0.4], [1.6, 1.4, 1.6]),
                ([0.1, 3.2, -0.2], [1.6, 1.2, 1.6]),
            ]));
            // pommes à moitié dans la couronne
            for k in 0..10 {
                let (a, e) = (k as f32 * 2.4, [-0.4_f32, 0.1, 0.5][k % 3]);
                let p = Vec3::new(1.3 * e.cos() * a.cos(), 2.6 + 0.95 * e.sin(), -1.3 * e.cos() * a.sin());
                v.push((Shape::Sphere, p, Vec3::splat(0.16), Some([0.8, 0.1, 0.08]), 0.0));
            }
        }
        PropKind::Flowers => {
            v.push(bx(0.0, 0.0, 1.3, 0.9, 0.0, 0.1, Some([0.35, 0.24, 0.15])));
            for k in 0..15 {
                let jitter = ((k * 37) % 7) as f32 * 0.02;
                let (x, z) = ((k % 5) as f32 * 0.24 - 0.48 + jitter, (k / 5) as f32 * 0.26 - 0.26 - jitter);
                let h = 0.35 + jitter * 2.0;
                v.push((Shape::Cylinder, Vec3::new(x, h / 2.0, z), Vec3::new(0.03, h, 0.03), Some(LEAF), 0.0));
                let head = match k % 4 {
                    3 => Some([0.98, 0.85, 0.2]),
                    1 => Some([0.97, 0.97, 0.95]),
                    _ => None,
                };
                v.push((Shape::Sphere, Vec3::new(x, h, z), Vec3::new(0.14, 0.08, 0.14), head, 0.0));
            }
            v.push((Shape::Sphere, Vec3::new(0.0, 0.12, 0.0), Vec3::new(1.2, 0.25, 0.8), Some(LEAF), 0.0));
        }
        PropKind::Rosebush => {
            for (p, s) in [([0.0, 0.45, 0.0], [1.0, 0.9, 1.0]), ([0.3, 0.6, 0.2], [0.7, 0.7, 0.7]), ([-0.3, 0.55, -0.2], [0.7, 0.7, 0.7])] {
                v.push((Shape::Sphere, Vec3::from(p), Vec3::from(s), Some(LEAF), 0.0));
            }
            for k in 0..12 {
                let (a, e) = (k as f32 * 2.4, [-0.2_f32, 0.3, 0.7][k % 3]);
                let p = Vec3::new(0.5 * e.cos() * a.cos(), 0.45 + 0.45 * e.sin(), -0.5 * e.cos() * a.sin());
                v.push((Shape::Sphere, p, Vec3::splat(0.15), None, 0.0));
            }
        }
        PropKind::Sunflower => {
            for (x, z, h) in [(0.0, 0.0, 1.7), (0.35, 0.2, 1.4), (-0.3, 0.25, 1.55), (0.1, -0.3, 1.25)] {
                v.push((Shape::Cylinder, Vec3::new(x, h / 2.0, z), Vec3::new(0.04, h, 0.04), Some(LEAF), 0.0));
                v.push((Shape::Sphere, Vec3::new(x, h * 0.55, z + 0.1), Vec3::new(0.28, 0.04, 0.16), Some(LEAF), 0.0));
                v.push((Shape::Cylinder, Vec3::new(x, h, z), Vec3::new(0.45, 0.04, 0.45), None, 0.0));
                v.push((Shape::Cylinder, Vec3::new(x, h + 0.02, z), Vec3::new(0.22, 0.04, 0.22), Some([0.35, 0.2, 0.08]), 0.0));
            }
        }
    }
    v
}

/// Pièces d'un objet posé à l'altitude `y` (voir `House::prop_y`).
pub fn prop_parts(p: &Prop, owner: ElemRef, open: f32, y: f32) -> Vec<Part> {
    let base = Transform::from_xyz(p.pos.x, y, p.pos.y)
        .with_rotation(Quat::from_rotation_y(p.rot.to_radians()));
    prop_shapes(p.kind, open, p.height)
        .into_iter()
        .map(|(shape, c, s, color, rot)| Part {
            tf: base.mul_transform(Transform::from_translation(c).with_rotation(Quat::from_rotation_y(rot)).with_scale(s)),
            shape,
            color: color.unwrap_or(p.color),
            // seules les pièces à la couleur du meuble reçoivent sa texture ; les troncs ont leur écorce
            texture: match color {
                None => p.texture,
                Some(BARK) => crate::textures::find("Écorce"),
                Some(BIRCH_BARK) => crate::textures::find("Écorce de bouleau"),
                Some(_) => 0,
            },
            owner,
        })
        .collect()
}

/// Triangles d'un toit en pente, faîtage le long de X. `size` = (longueur, hauteur, largeur).
pub fn roof_tris(kind: RoofKind, size: Vec3) -> Vec<[Vec3; 3]> {
    let (a, h, b) = (size.x / 2.0, size.y, size.z / 2.0);
    let rect = |a: f32, b: f32, y: f32| [Vec3::new(-a, y, -b), Vec3::new(a, y, -b), Vec3::new(a, y, b), Vec3::new(-a, y, b)];
    let c = rect(a, b, 0.0);
    let mut t = vec![[c[0], c[2], c[1]], [c[0], c[3], c[2]]]; // dessous
    // pans montant de la base `c` jusqu'à un faîtage le long de X, raccourci de `inset` à chaque bout
    let ridge = |t: &mut Vec<[Vec3; 3]>, c: [Vec3; 4], top: f32, inset: f32| {
        let a = c[1].x;
        let r = [Vec3::new(-a + inset, top, 0.0), Vec3::new(a - inset, top, 0.0)];
        t.extend([
            [c[0], c[1], r[1]], [c[0], r[1], r[0]], // pan avant
            [c[2], c[3], r[0]], [c[2], r[0], r[1]], // pan arrière
            [c[3], c[0], r[0]], [c[1], c[2], r[1]], // pignons / croupes
        ]);
    };
    match kind {
        RoofKind::Gable => ridge(&mut t, c, h, 0.0),
        RoofKind::Hip => ridge(&mut t, c, h, b.min(a)),
        RoofKind::Pyramid => ridge(&mut t, c, h, a),
        RoofKind::Mansard => {
            // brisis raide jusqu'aux deux tiers, puis terrasson à faible pente
            let k = 0.2 * a.min(b);
            let top = rect(a - k, b - k, 0.65 * h);
            for i in 0..4 {
                let j = (i + 1) % 4;
                t.extend([[c[i], c[j], top[j]], [c[i], top[j], top[i]]]);
            }
            ridge(&mut t, top, h, (b - k).min(a - k));
        }
        RoofKind::Shed => {
            let (t0, t1) = (Vec3::new(-a, h, b), Vec3::new(a, h, b));
            t.extend([[c[0], c[1], t1], [c[0], t1, t0], [c[3], c[2], t1], [c[3], t1, t0], [c[0], t0, c[3]], [c[1], c[2], t1]]);
        }
        RoofKind::Flat => {}
    }
    // les faîtages réduits à un point laissent des triangles plats
    t.retain(|[p, q, r]| (*q - *p).cross(*r - *p).length_squared() > 1e-8);
    t
}

// ---------------------------------------------------------------- sélection par rayon

/// Distance d'intersection entre un rayon et un cube unitaire transformé.
pub fn ray_obb(origin: Vec3, dir: Vec3, tf: &Transform) -> Option<f32> {
    let inv = tf.compute_affine().inverse();
    let (o, d) = (inv.transform_point3(origin), inv.transform_vector3(dir));
    let (mut t0, mut t1) = (f32::NEG_INFINITY, f32::INFINITY);
    for i in 0..3 {
        if d[i].abs() < 1e-8 {
            if o[i].abs() > 0.5 {
                return None;
            }
            continue;
        }
        let (a, b) = ((-0.5 - o[i]) / d[i], (0.5 - o[i]) / d[i]);
        t0 = t0.max(a.min(b));
        t1 = t1.min(a.max(b));
    }
    (t0 <= t1 && t1 > 0.0).then_some(t0.max(0.0))
}

/// Élément le plus proche touché par le rayon, avec la distance.
pub fn pick(parts: &[Part], origin: Vec3, dir: Vec3) -> Option<(ElemRef, f32)> {
    parts
        .iter()
        .filter(|p| p.shape != Shape::Light)
        .filter_map(|p| Some((p.owner, ray_obb(origin, dir, &p.bounds())?)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

// ---------------------------------------------------------------- maison d'exemple

pub fn demo_house() -> House {
    use crate::textures::find;
    let mut h = House::default();
    h.walls = room_walls(Vec2::ZERO, Vec2::new(10.0, 8.0), 0, LEVEL_H, 0.25, WHITE_TEX);
    for w in &mut h.walls {
        w.texture = find("Crépi beige");
    }
    h.walls.push(Wall {
        a: Vec2::new(6.0, 0.0),
        b: Vec2::new(6.0, 8.0),
        level: 0,
        height: LEVEL_H,
        thick: 0.12,
        color: [0.85, 0.9, 0.95],
        openings: vec![],
        texture: find("Plâtre"),
    });
    let op = |kind: OpeningKind, t: f32| {
        let (width, height, sill) = kind.default_dims();
        Opening { kind, t, width, height, sill, color: kind.default_color() }
    };
    h.walls[0].openings = vec![op(OpeningKind::Door, 3.0), op(OpeningKind::Window, 1.2), op(OpeningKind::Window, 8.0)];
    h.walls[1].openings = vec![op(OpeningKind::Window, 4.0)];
    h.walls[2].openings = vec![op(OpeningKind::Bay, 6.5), op(OpeningKind::Window, 2.0)];
    h.walls[3].openings = vec![op(OpeningKind::Window, 4.0)];
    h.walls[4].openings = vec![op(OpeningKind::Door, 5.0)];
    h.slabs.push(Slab { min: Vec2::ZERO, max: Vec2::new(10.0, 8.0), level: 0, color: WHITE_TEX, texture: find("Parquet chêne") });
    h.slabs.push(Slab {
        min: Vec2::new(-2.0, -5.0),
        max: Vec2::new(12.0, -0.13),
        level: 0,
        color: WHITE_TEX,
        texture: find("Dalles de jardin"),
    });
    h.roofs.push(Roof {
        min: Vec2::ZERO,
        max: Vec2::new(10.0, 8.0),
        level: 1,
        kind: RoofKind::Gable,
        height: 2.5,
        overhang: 0.5,
        color: WHITE_TEX,
        texture: find("Tuiles terre cuite"),
    });
    let mut prop = |kind: PropKind, x: f32, z: f32, rot: f32| {
        h.props.push(Prop { kind, pos: Vec2::new(x, z), level: 0, rot, color: kind.default_color(), texture: kind.default_texture(), height: LEVEL_H })
    };
    prop(PropKind::Sofa, 2.5, 6.8, 180.0);
    prop(PropKind::Table, 3.0, 3.5, 0.0);
    prop(PropKind::Chair, 2.5, 2.8, 0.0);
    prop(PropKind::Chair, 3.5, 2.8, 0.0);
    prop(PropKind::Chair, 2.5, 4.2, 180.0);
    prop(PropKind::Chair, 3.5, 4.2, 180.0);
    prop(PropKind::Kitchen, 1.6, 0.45, 0.0);
    prop(PropKind::Fridge, 5.3, 0.5, 0.0);
    prop(PropKind::Plant, 5.4, 7.4, 0.0);
    prop(PropKind::Bed, 8.0, 6.8, 180.0);
    prop(PropKind::Wardrobe, 9.4, 2.0, 270.0);
    prop(PropKind::Lamp, 0.6, 7.4, 0.0);
    prop(PropKind::Tree, -4.0, -3.0, 0.0);
    prop(PropKind::Pine, 14.0, 10.0, 0.0);
    prop(PropKind::Tree, -5.0, 9.0, 0.0);
    prop(PropKind::Pine, 16.0, -2.0, 0.0);
    for i in 0..5 {
        prop(PropKind::Hedge, 11.5, 1.0 + 2.0 * i as f32, 90.0);
    }
    prop(PropKind::Bush, 1.0, -1.2, 0.0);
    prop(PropKind::Bush, 5.0, -1.2, 0.0);
    for i in 0..6 {
        prop(PropKind::Fence, -2.0 + 2.0 * i as f32 + 1.0, -4.0, 0.0);
    }
    prop(PropKind::Flowers, 8.5, -1.2, 0.0);
    prop(PropKind::Rosebush, -1.2, -1.2, 0.0);
    prop(PropKind::Birch, -12.0, 4.0, 0.0);
    prop(PropKind::AppleTree, 15.0, 4.0, 0.0);
    prop(PropKind::Palm, -7.0, -3.0, 0.0);
    h.generate_terrain(7, 6.0, 45.0, false);
    h.add_pool(Pool { min: Vec2::new(-10.0, 1.0), max: Vec2::new(-4.0, 5.0), depth: 1.5, color: WHITE_TEX, texture: find("Mosaïque bleue") });
    h.add_road(Road { a: Vec2::new(14.0, -6.0), b: Vec2::new(14.0, -60.0), width: 4.0, color: [0.55, 0.55, 0.57], texture: find("Béton brut") });
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall(len: f32) -> Wall {
        Wall { a: Vec2::ZERO, b: Vec2::new(len, 0.0), level: 0, height: 3.0, thick: 0.2, color: WHITE, openings: vec![], texture: 0 }
    }

    #[test]
    fn new_roofs_have_no_flat_triangles() {
        for k in RoofKind::ALL.into_iter().filter(|k| *k != RoofKind::Flat) {
            let tris = roof_tris(k, Vec3::new(10.0, 3.0, 6.0));
            assert!(tris.len() >= 6, "{k:?}");
            let top = tris.iter().flatten().map(|v| v.y).fold(0.0, f32::max);
            assert!((top - 3.0).abs() < 1e-4, "{k:?} culmine à {top}");
        }
    }

    #[test]
    fn add_storey_copies_walls_and_lifts_roof() {
        let mut h = demo_house();
        let walls0 = h.walls.iter().filter(|w| w.level == 0).count();
        h.add_storey(0);
        assert_eq!(h.walls.iter().filter(|w| w.level == 1).count(), walls0);
        assert!(h.walls.iter().filter(|w| w.level == 1).all(|w| w.openings.iter().all(|o| o.kind != OpeningKind::Door)));
        assert_eq!(h.roofs[0].level, 2);
        // le parquet est copié, pas la terrasse
        assert_eq!(h.slabs.iter().filter(|s| s.level == 1).count(), 1);
    }

    #[test]
    fn pools_are_dug_and_flat_and_trees_follow_terrain() {
        let mut h = demo_house();
        let pool = h.pools[0].clone();
        let c = (pool.min + pool.max) / 2.0;
        assert!(h.ground(c) < -10.0, "dans la piscine on tombe sur le fond");
        let top = h.terrain.height(pool.min);
        assert!((h.terrain.height(pool.max) - top).abs() < 1e-4);
        h.sculpt(crate::terrain::Brush::Raise, pool.max, 3.0, 1.0, 0.0, 0);
        assert!((h.terrain.height(pool.max) - h.terrain.height(pool.min)).abs() < 1e-4, "le bassin reste plat");
        let tree = h.props.iter().find(|p| p.kind == PropKind::AppleTree).unwrap();
        assert_eq!(h.prop_y(tree), h.terrain.height(tree.pos));
        let sofa = h.props.iter().find(|p| p.kind == PropKind::Sofa).unwrap();
        assert_eq!(h.prop_y(sofa), 0.0);
    }

    #[test]
    fn stairs_reach_their_height() {
        for kind in [PropKind::Stairs, PropKind::SpiralStairs, PropKind::LStairs] {
            for height in [1.0, 3.0, 4.5] {
                let p = Prop { kind, pos: Vec2::ZERO, level: 0, rot: 0.0, color: WHITE, texture: 0, height };
                let top = prop_parts(&p, ElemRef::Prop(0), 0.0, 0.0)
                    .iter()
                    .filter(|p| p.shape == Shape::Cube)
                    .map(|p| p.tf.translation.y + p.tf.scale.y / 2.0)
                    .fold(0.0, f32::max);
                assert!((top - height).abs() < 1e-3, "{kind:?} {height} → {top}");
            }
        }
    }

    #[test]
    fn door_leaves_a_hole_in_the_wall() {
        let mut w = wall(4.0);
        w.openings.push(Opening { kind: OpeningKind::Door, t: 2.0, width: 1.0, height: 2.1, sill: 0.0, color: WHITE });
        let mut parts = vec![];
        wall_parts(&w, 0, &|_| 0.0, &mut parts);
        // un point au milieu de la porte, à 1 m de haut, ne doit toucher aucun morceau de mur
        let probe = Vec3::new(2.0, 1.0, 0.0);
        for p in parts.iter().filter(|p| p.owner == ElemRef::Wall(0)) {
            let local = p.tf.compute_affine().inverse().transform_point3(probe);
            assert!(local.abs().max_element() > 0.5, "mur dans la porte : {:?}", p.tf);
        }
        assert_eq!(parts.iter().filter(|p| p.owner == ElemRef::Wall(0)).count(), 3);
        assert!(parts.iter().any(|p| p.owner == ElemRef::Opening(0, 0)));
    }

    #[test]
    fn opening_fit_rules() {
        let mut w = wall(4.0);
        assert!(w.fits(2.0, 1.0, None));
        assert!(!w.fits(0.3, 1.0, None), "dépasse du mur");
        w.openings.push(Opening { kind: OpeningKind::Window, t: 2.0, width: 1.0, height: 1.0, sill: 1.0, color: WHITE });
        assert!(!w.fits(2.5, 1.0, None), "chevauche");
        assert!(w.fits(2.5, 1.0, Some(0)));
        assert!(w.fits(3.3, 0.6, None));
    }

    #[test]
    fn ray_picks_nearest_element() {
        let h = demo_house();
        let parts = h.parts_closed(Some(0));
        // depuis le sud, vers le nord, à 2 m de haut sur x = 5 : le mur sud (mur 0) est le premier touché
        let hit = pick(&parts, Vec3::new(5.0, 2.0, -20.0), Vec3::Z);
        assert_eq!(hit.map(|h| h.0), Some(ElemRef::Wall(0)));
        assert!(pick(&parts, Vec3::new(500.0, 2.0, 0.0), Vec3::Y).is_none());
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
    fn doors_swing_open() {
        let h = demo_house();
        let door = ElemRef::Opening(0, 0);
        assert!(h.openable(door) && !h.openable(ElemRef::Opening(0, 1)), "porte oui, fenêtre non");
        let panel = |open: f32| {
            let parts = h.parts(Some(0), &|e| if e == door { open } else { 0.0 });
            // le battant : haut et fin (5 cm)
            parts.into_iter().find(|p| p.owner == door && p.tf.scale.y > 1.5 && (p.tf.scale.z - 0.05).abs() < 1e-4).unwrap().tf
        };
        let (closed, open) = (panel(0.0), panel(1.0));
        assert!((closed.translation.z - 0.0).abs() < 0.01, "fermée : dans le plan du mur");
        assert!(open.translation.z > 0.3, "ouverte : sortie du plan du mur");
        let wardrobe = h.props.iter().position(|p| p.kind == PropKind::Wardrobe).unwrap();
        assert!(h.openable(ElemRef::Prop(wardrobe)));
    }

    #[test]
    fn save_format_roundtrips() {
        let h = demo_house();
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(serde_json::from_str::<House>(&json).unwrap(), h);
    }
}
