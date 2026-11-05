//! Données de la maison (sérialisables) et leur conversion en pièces géométriques.

use bevy::math::{Quat, Vec2, Vec3};
// shortcut: Transform (bevy_transform) n'est pas un type maths, à remplacer si le cœur doit se passer de Bevy
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod annotations;
mod catalog;
mod check;
mod demo;
mod edit;
mod geometry;
mod openings;
mod props;
mod roofs;
mod shapes;

pub use catalog::{Category, PropKind};
pub use demo::demo_house;
pub use geometry::{pick, pick_part, ray_obb};
use geometry::{pool_parts, road_part, roof_part, wall_parts};
pub use openings::OpeningKind;
pub use props::{Prop, prop_parts};
pub use roofs::{RoofKind, roof_tris};

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

/// Teinte et matière d'une partie d'un objet ou d'une ouverture.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Finish {
    /// Teinte `[r, g, b]`, multipliée par la texture.
    pub color: Rgb,
    /// Matière (`maison3d textures`), 0 = aucune.
    #[serde(default)]
    pub texture: usize,
}

/// Partie colorable d'une sorte d'objet ou d'ouverture : son nom et sa finition d'origine.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Slot {
    pub name: &'static str,
    pub color: Rgb,
    /// Nom de la texture d'origine (`maison3d textures`), vide pour aucune.
    pub texture: &'static str,
}

impl Slot {
    pub const fn new(name: &'static str, color: Rgb, texture: &'static str) -> Self {
        Self { name, color, texture }
    }
    /// Couleur unie.
    pub const fn plain(name: &'static str, color: Rgb) -> Self {
        Self::new(name, color, "")
    }
    /// Texture montrée telle quelle (teinte blanche).
    pub const fn textured(name: &'static str, texture: &'static str) -> Self {
        Self::new(name, WHITE_TEX, texture)
    }
    pub fn finish(&self) -> Finish {
        Finish { color: self.color, texture: crate::textures::find(self.texture) }
    }
}

/// Finition de la partie `slot` d'un élément dont la partie 0 est `main` et les suivantes
/// `finishes` (complétées par les finitions d'origine de `slots`).
fn finish_at(main: Finish, finishes: &[Finish], slots: &[Slot], slot: usize) -> Finish {
    match slot.checked_sub(1) {
        None => main,
        Some(i) => finishes.get(i).copied().or_else(|| slots.get(slot).map(Slot::finish)).unwrap_or(main),
    }
}

/// Couleur et matière modifiables de la partie `slot` ; complète `finishes` jusqu'à elle avec
/// les finitions d'origine. `None` si l'élément n'a pas cette partie.
fn finish_at_mut<'a>(
    (color, texture): (&'a mut Rgb, &'a mut usize),
    finishes: &'a mut Vec<Finish>,
    slots: &[Slot],
    slot: usize,
) -> Option<(&'a mut Rgb, &'a mut usize)> {
    if slot >= slots.len().max(1) {
        return None;
    }
    let Some(i) = slot.checked_sub(1) else { return Some((color, texture)) };
    while finishes.len() <= i {
        finishes.push(slots[finishes.len() + 1].finish());
    }
    let f = &mut finishes[i];
    Some((&mut f.color, &mut f.texture))
}

// ---------------------------------------------------------------- éléments

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Porte ou fenêtre percée dans un mur.
pub struct Opening {
    pub kind: OpeningKind,
    /// Position du centre le long du mur, en mètres depuis son extrémité `a`.
    pub t: f32,
    /// Largeur en mètres.
    pub width: f32,
    /// Hauteur en mètres ; le haut est coupé à la hauteur du mur.
    pub height: f32,
    /// Allège : hauteur du bas de l'ouverture au-dessus du sol de l'étage (0 pour une porte).
    pub sill: f32,
    /// Teinte `[r, g, b]` de la partie principale (battant d'une porte, cadre d'une fenêtre).
    pub color: Rgb,
    /// Matière de la partie principale (`maison3d textures`), 0 = aucune.
    #[serde(default)]
    pub texture: usize,
    /// Finitions des parties suivantes (`maison3d catalog` : `parts`), à partir de la partie 1 ;
    /// une partie absente garde sa finition d'origine.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finishes: Vec<Finish>,
}

impl Opening {
    /// Ouverture aux dimensions et aux finitions d'origine de son type, centrée en `t`.
    pub fn new(kind: OpeningKind, t: f32) -> Self {
        let (width, height, sill) = kind.default_dims();
        let Finish { color, texture } = kind.slots()[0].finish();
        Self { kind, t, width, height, sill, color, texture, finishes: vec![] }
    }
    pub fn finish(&self, slot: usize) -> Finish {
        finish_at(Finish { color: self.color, texture: self.texture }, &self.finishes, self.kind.slots(), slot)
    }
    pub fn finish_mut(&mut self, slot: usize) -> Option<(&mut Rgb, &mut usize)> {
        finish_at_mut((&mut self.color, &mut self.texture), &mut self.finishes, self.kind.slots(), slot)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Mur droit de `a` à `b`, d'épaisseur centrée sur ce segment.
pub struct Wall {
    /// Extrémité de départ `[x, z]` ; les positions `t` des ouvertures se mesurent depuis elle.
    #[schemars(with = "[f32; 2]")]
    pub a: Vec2,
    /// Extrémité d'arrivée `[x, z]` ; au moins 5 cm de `a`.
    #[schemars(with = "[f32; 2]")]
    pub b: Vec2,
    /// Étage (-1 à 9) : le mur part du sol de cet étage.
    pub level: i32,
    /// Hauteur en mètres (3 = un étage plein).
    pub height: f32,
    /// Épaisseur en mètres (0,2 pour un mur extérieur, 0,1 pour une cloison).
    pub thick: f32,
    /// Teinte `[r, g, b]`, multipliée par la texture.
    pub color: Rgb,
    /// Portes et fenêtres ; `[]` puis `place_opening` est plus simple.
    pub openings: Vec<Opening>,
    /// Matière (`maison3d textures`), 0 = aucune.
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
        Transform::from_xyz(self.a.x, elev(self.level), self.a.y).with_rotation(Quat::from_rotation_y(-d.y.atan2(d.x)))
    }
    /// Plage des centres `t` possibles pour une ouverture de largeur `w` : 5 cm de marge plus
    /// la demi-épaisseur du mur perpendiculaire (supposé de même épaisseur) à chaque bout.
    pub fn t_range(&self, w: f32) -> (f32, f32) {
        let margin = 0.05 + self.thick / 2.0;
        (w / 2.0 + margin, self.len() - w / 2.0 - margin)
    }
    /// Première ouverture (hors `ignore`) que chevaucherait une ouverture de largeur `w` en `t`.
    pub fn overlap(&self, t: f32, w: f32, ignore: Option<usize>) -> Option<usize> {
        self.openings
            .iter()
            .enumerate()
            .position(|(i, o)| Some(i) != ignore && (t - o.t).abs() < f32::midpoint(w, o.width) + 0.05)
    }
    /// Une ouverture de largeur `w` centrée en `t` tient-elle sans chevaucher les autres ?
    pub fn fits(&self, t: f32, w: f32, ignore: Option<usize>) -> bool {
        let (lo, hi) = self.t_range(w);
        (lo..=hi).contains(&t) && self.overlap(t, w, ignore).is_none()
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Sol rectangulaire (plancher, dalle, terrasse) de 22 cm d'épaisseur, dessus au niveau de
/// l'étage. Un sol à `level` 1 sert aussi de plafond au rez-de-chaussée.
pub struct Slab {
    /// Coin nord-ouest `[x, z]`.
    #[schemars(with = "[f32; 2]")]
    pub min: Vec2,
    /// Coin sud-est `[x, z]`, strictement plus grand que `min`.
    #[schemars(with = "[f32; 2]")]
    pub max: Vec2,
    /// Étage (-1 à 9).
    pub level: i32,
    /// Teinte `[r, g, b]`, multipliée par la texture.
    pub color: Rgb,
    /// Matière (`maison3d textures`), 0 = aucune.
    #[serde(default)]
    pub texture: usize,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Toit posé à la base de `level` sur l'emprise `min`–`max`, débord ajouté autour.
pub struct Roof {
    /// Coin nord-ouest de l'emprise des murs couverts.
    #[schemars(with = "[f32; 2]")]
    pub min: Vec2,
    /// Coin sud-est de l'emprise des murs couverts.
    #[schemars(with = "[f32; 2]")]
    pub max: Vec2,
    /// Étage où repose le toit : 1 pour couvrir des murs du rez-de-chaussée.
    pub level: i32,
    pub kind: RoofKind,
    /// Hauteur du faîtage au-dessus de la base, en mètres (2,5 par défaut).
    pub height: f32,
    /// Débord en mètres ajouté de chaque côté de l'emprise (0,5 par défaut).
    pub overhang: f32,
    /// Teinte `[r, g, b]`, multipliée par la texture.
    pub color: Rgb,
    /// Matière (`maison3d textures`), 0 = aucune.
    #[serde(default)]
    pub texture: usize,
}

/// Piscine creusée dans le terrain ; coins alignés sur la grille du relief (1 m).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    /// Coin nord-ouest `[x, z]`, en mètres entiers.
    #[schemars(with = "[f32; 2]")]
    pub min: Vec2,
    /// Coin sud-est `[x, z]`, en mètres entiers, strictement plus grand que `min`.
    #[schemars(with = "[f32; 2]")]
    pub max: Vec2,
    /// Profondeur d'eau en mètres (1,5 par défaut dans l'éditeur).
    pub depth: f32,
    /// Teinte du bassin `[r, g, b]`.
    pub color: Rgb,
    /// Matière du bassin (`maison3d textures`, « Mosaïque bleue » dans l'éditeur).
    pub texture: usize,
}

/// Tronçon de route posé sur le terrain (qui est terrassé dessous à sa création).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Road {
    /// Début `[x, z]`.
    #[schemars(with = "[f32; 2]")]
    pub a: Vec2,
    /// Fin `[x, z]`, au moins 5 cm de `a`.
    #[schemars(with = "[f32; 2]")]
    pub b: Vec2,
    /// Largeur en mètres (4 par défaut dans l'éditeur, 3 pour une allée).
    pub width: f32,
    /// Teinte `[r, g, b]`.
    pub color: Rgb,
    /// Matière (`maison3d textures`, « Béton brut » dans l'éditeur).
    pub texture: usize,
}

/// Version du format de sauvegarde ; à incrémenter (avec migration dans `House::load`) à chaque changement incompatible.
pub const VERSION: u32 = 1;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct House {
    /// Absent des fichiers d'avant le versionnage : 0.
    #[serde(default)]
    pub version: u32,
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

impl Default for House {
    fn default() -> Self {
        Self {
            version: VERSION,
            walls: vec![],
            slabs: vec![],
            roofs: vec![],
            props: vec![],
            pools: vec![],
            roads: vec![],
            terrain: Terrain::default(),
        }
    }
}

/// Échec de lecture ou d'écriture d'un monde.
#[derive(Debug)]
pub enum WorldError {
    Io(std::io::Error),
    Json(serde_json::Error),
    BadTerrain { n: usize, full: usize },
    TooNew(u32),
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::BadTerrain { n, full } => write!(f, "relief invalide : {n} altitudes au lieu de {full}"),
            Self::TooNew(v) => write!(f, "format {v} plus récent que ce programme (format {VERSION})"),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<std::io::Error> for WorldError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for WorldError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Référence vers un élément par son indice dans sa liste, comme l'affiche `maison3d describe`.
/// En JSON : `{"Wall": 0}`, `{"Opening": [mur, n]}`, `{"Prop": 3}`…
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
pub enum ElemRef {
    /// Mur n.
    Wall(usize),
    /// Ouverture : `[indice du mur, indice de l'ouverture dans ce mur]`.
    Opening(usize, usize),
    /// Sol n.
    Slab(usize),
    /// Toit n.
    Roof(usize),
    /// Objet n (meuble, escalier, végétal, voiture).
    Prop(usize),
    /// Piscine n.
    Pool(usize),
    /// Tronçon de route n.
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
    /// Roue : axe le long de X ; `tf.scale` = (largeur, diamètre, diamètre).
    Wheel,
    /// Toit en pente : `size` = (longueur, hauteur, largeur), base en y = 0.
    Roof {
        kind: RoofKind,
        size: Vec3,
    },
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
    /// Partie colorable de l'élément à laquelle appartient la pièce (0 hors objets et ouvertures).
    pub slot: usize,
}

impl Part {
    /// Boîte englobante sous forme de cube unitaire transformé (sélection, aperçus).
    pub fn bounds(&self) -> Transform {
        match self.shape {
            Shape::Roof { size, .. } => {
                self.tf.mul_transform(Transform::from_xyz(0.0, size.y / 2.0, 0.0).with_scale(size))
            }
            _ => self.tf,
        }
    }
}

fn cube(tf: Transform, color: Rgb, texture: usize, owner: ElemRef) -> Part {
    Part { tf, shape: Shape::Cube, color, texture, owner, slot: 0 }
}

/// Ouverture (0 = fermé, 1 = ouvert) des portes et placards, pour l'animation de la visite.
pub type OpenFn<'a> = &'a dyn Fn(ElemRef) -> f32;

/// Boîte locale définie par ses bornes x, y et sa profondeur z centrée.
fn local_box(x0: f32, x1: f32, y0: f32, y1: f32, depth: f32) -> Option<Transform> {
    (x1 - x0 > 1e-3 && y1 - y0 > 1e-3).then(|| {
        Transform::from_xyz(f32::midpoint(x0, x1), f32::midpoint(y0, y1), 0.0).with_scale(Vec3::new(
            x1 - x0,
            y1 - y0,
            depth,
        ))
    })
}

impl House {
    /// Lit un monde enregistré ; refuse un relief dont la grille n'a pas la bonne taille.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<House, WorldError> {
        let mut h: House = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        if h.version > VERSION {
            return Err(WorldError::TooNew(h.version));
        }
        // 0 → 1 : seul le champ `version` apparaît
        h.version = VERSION;
        let (n, full) = (h.terrain.h.len(), crate::terrain::SIDE * crate::terrain::SIDE);
        if n != 0 && n != full {
            return Err(WorldError::BadTerrain { n, full });
        }
        Ok(h)
    }

    /// Écrit à côté puis renomme : un arrêt en pleine écriture ne corrompt pas le monde.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), WorldError> {
        let path = path.as_ref();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

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
            let tf = Transform::from_xyz(c.x, elev(s.level) - 0.09, c.y).with_scale(Vec3::new(size.x, 0.22, size.y));
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

    /// Altitude de base d'un objet : au rez-de-chaussée, les objets d'extérieur suivent le relief,
    /// ou le dessus de la route (marquages, voitures).
    pub fn prop_y(&self, p: &Prop) -> f32 {
        if p.level == 0 && p.kind.outdoor() {
            self.road_top(p.pos).unwrap_or_else(|| self.terrain.height(p.pos))
        } else {
            elev(p.level)
        }
    }

    /// Dessus de la plus haute route qui passe en `p` (voir `road_part` : 6 cm au-dessus de la
    /// droite qui relie ses deux bouts sur le relief).
    fn road_top(&self, p: Vec2) -> Option<f32> {
        self.roads
            .iter()
            .filter(|r| r.a.distance_squared(r.b) > 1e-6)
            .filter_map(|r| {
                let d = r.b - r.a;
                let t = ((p - r.a).dot(d) / d.length_squared()).clamp(0.0, 1.0);
                let (ya, yb) = (self.terrain.height(r.a), self.terrain.height(r.b));
                (p.distance(r.a + d * t) <= r.width / 2.0).then_some(ya + (yb - ya) * t + 0.06)
            })
            .reduce(f32::max)
    }

    /// Altitude du sol nu ; dans une piscine, très bas : c'est son fond qui porte.
    pub fn ground(&self, p: Vec2) -> f32 {
        if self.pools.iter().any(|q| p.cmpgt(q.min).all() && p.cmplt(q.max).all()) {
            -50.0
        } else {
            self.terrain.height(p)
        }
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
        let keep =
            pts.fold(None, |acc: Option<(Vec2, Vec2)>, p| Some(acc.map_or((p, p), |(lo, hi)| (lo.min(p), hi.max(p)))));
        self.terrain.generate(seed, amp, scale, mountains, keep);
        self.regrade();
    }

    /// Reterrasse routes et piscines après un changement du relief.
    pub fn regrade(&mut self) {
        for r in &self.roads {
            self.terrain.grade(r.a, r.b, r.width);
        }
        self.settle_pools();
    }

    /// Pièces avec tout fermé (aperçus, éditeur).
    pub fn parts_closed(&self, max_level: Option<i32>) -> Vec<Part> {
        self.parts(max_level, &|_| 0.0)
    }
}

/// Les quatre murs d'une pièce rectangulaire.
pub fn room_walls(min: Vec2, max: Vec2, level: i32, height: f32, thick: f32, color: Rgb) -> Vec<Wall> {
    let c = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];
    (0..4)
        .map(|i| Wall { a: c[i], b: c[(i + 1) % 4], level, height, thick, color, openings: vec![], texture: 0 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn outdoor_props_sit_on_roads() {
        let mut h = House::default();
        h.roads.push(Road { a: Vec2::new(-10.0, 0.0), b: Vec2::new(10.0, 0.0), width: 6.0, color: WHITE, texture: 0 });
        h.roads.push(Road { a: Vec2::ZERO, b: Vec2::ZERO, width: 6.0, color: WHITE, texture: 0 });
        let at = |kind, z: f32| h.prop_y(&Prop::new(kind, Vec2::new(0.0, z), 0, 0.0));
        assert!((at(PropKind::Crosswalk, 2.9) - 0.06).abs() < 1e-5, "sur la chaussée");
        assert_eq!(at(PropKind::Crosswalk, 3.5), 0.0, "à côté de la route");
        assert_eq!(at(PropKind::Sofa, 0.0), 0.0, "un objet d'intérieur reste au niveau de l'étage");
    }
    #[test]
    fn opening_fit_rules() {
        let mut w = wall(4.0);
        assert!(w.fits(2.0, 1.0, None));
        assert!(!w.fits(0.3, 1.0, None), "dépasse du mur");
        w.openings.push(Opening {
            kind: OpeningKind::Window,
            t: 2.0,
            width: 1.0,
            height: 1.0,
            sill: 1.0,
            color: WHITE,
            texture: 0,
            finishes: vec![],
        });
        assert!(!w.fits(2.5, 1.0, None), "chevauche");
        assert!(w.fits(2.5, 1.0, Some(0)));
        assert!(w.fits(3.3, 0.6, None));
    }
    #[test]
    fn save_format_roundtrips() {
        let h = demo_house();
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(serde_json::from_str::<House>(&json).unwrap(), h);
    }
    #[test]
    fn load_rejects_truncated_terrain() {
        let mut h = House::default();
        h.terrain.h = vec![0.0; 10];
        let path = std::env::temp_dir().join("maison3d_relief_tronque.json");
        std::fs::write(&path, serde_json::to_string(&h).unwrap()).unwrap();
        assert!(matches!(House::load(&path), Err(WorldError::BadTerrain { n: 10, .. })));
        h.terrain.h.clear();
        std::fs::write(&path, serde_json::to_string(&h).unwrap()).unwrap();
        assert_eq!(House::load(&path).unwrap(), h);
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn load_reports_missing_file_and_bad_json() {
        assert!(matches!(House::load("/nulle/part/maison.json"), Err(WorldError::Io(_))));
        let path = std::env::temp_dir().join("maison3d_json_casse.json");
        std::fs::write(&path, "{ pas du json").unwrap();
        assert!(matches!(House::load(&path), Err(WorldError::Json(_))));
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn save_writes_atomically_and_reports_errors() {
        let dir = std::env::temp_dir().join("maison3d_house_save");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("sous/maison.json");
        let h = demo_house();
        h.save(&path).unwrap();
        assert_eq!(House::load(&path).unwrap(), h);
        assert!(!dir.join("sous/maison.json.tmp").exists());
        // le dossier parent est un fichier
        assert!(matches!(h.save(path.join("x.json")), Err(WorldError::Io(_))));
        assert!(matches!(h.save(&dir), Err(WorldError::Io(_))), "un dossier n'est pas un fichier");
        let _ = std::fs::remove_dir_all(dir);
    }
    #[test]
    fn loads_files_from_before_versioning() {
        let h = House::load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/monde_v0.json")).unwrap();
        assert_eq!(h.version, VERSION);
        assert_eq!((h.walls.len(), h.slabs.len(), h.props.len()), (4, 1, 1));
    }
    #[test]
    fn refuses_files_from_a_newer_format() {
        let path = std::env::temp_dir().join("maison3d_futur.json");
        House { version: VERSION + 1, ..House::default() }.save(&path).unwrap();
        assert!(matches!(House::load(&path), Err(WorldError::TooNew(v)) if v == VERSION + 1));
        let _ = std::fs::remove_file(path);
    }
}
