//! Meubles, escaliers, végétaux et voitures posés dans la maison ou le jardin.

use bevy::math::{Quat, Vec2, Vec3};
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::shapes::{prop_shapes, prop_slots};
use super::{ElemRef, Finish, LEVEL_H, Part, PropKind, Rgb, Shape, Slot, WHITE, finish_at, finish_at_mut};

impl PropKind {
    /// Parties colorables (`prop_slots`) ; la partie 0 reçoit `color` et `texture` de l'objet.
    pub fn slots(self) -> &'static [Slot] {
        prop_slots(self)
    }
    pub fn default_color(self) -> Rgb {
        self.slots().first().map_or(WHITE, |s| s.color)
    }
    /// Posé à plat au sol (moins de 5 cm : tapis, tracé, plaque) : les autres objets vont dessus.
    pub fn flat(self) -> bool {
        self.size().z < 0.05
    }
    /// S'ouvre pendant la visite (armoire, placards, portes) : sa géométrie bouge avec `open`.
    pub fn openable(self) -> bool {
        prop_shapes(self, 0.0, LEVEL_H) != prop_shapes(self, 1.0, LEVEL_H)
    }
    pub fn default_texture(self) -> usize {
        self.slots().first().map_or(0, |s| s.finish().texture)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Objet posé : meuble, escalier, végétal ou voiture.
pub struct Prop {
    pub kind: PropKind,
    /// Centre de l'objet `[x, z]`.
    #[schemars(with = "[f32; 2]")]
    pub pos: Vec2,
    /// Étage (-1 à 9). Au rez-de-chaussée, les objets d'extérieur se posent sur le relief.
    pub level: i32,
    /// Rotation en degrés autour de la verticale : l'avant pointe vers `[sin rot, cos rot]`
    /// (0 = sud/+z, 90 = est/+x, 180 = nord/-z, 270 = ouest/-x).
    pub rot: f32,
    /// Teinte `[r, g, b]` des parties principales (feuillage, tissu, bois).
    pub color: Rgb,
    /// Matière (`maison3d textures`), 0 = aucune.
    #[serde(default)]
    pub texture: usize,
    /// Dimension réglable en mètres (3 par défaut) : hauteur montée par un escalier, hauteur d'un
    /// pilier, longueur d'une ligne, rayon d'un cercle (`maison3d catalog` : `dim`) ; ignorée
    /// pour les autres objets.
    #[serde(default = "level_h")]
    pub height: f32,
    /// Finitions des parties suivantes (`maison3d catalog` : `parts`), à partir de la partie 1 ;
    /// une partie absente garde sa finition d'origine. Plus simple : `paint` avec `part`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finishes: Vec<Finish>,
}

impl Prop {
    /// Objet aux finitions d'origine de sa sorte, de hauteur 3 m.
    pub fn new(kind: PropKind, pos: Vec2, level: i32, rot: f32) -> Self {
        let (color, texture) = (kind.default_color(), kind.default_texture());
        Prop { kind, pos, level, rot, color, texture, height: LEVEL_H, finishes: vec![] }
    }
    /// Finition de la partie `slot` (0 = `color` et `texture`).
    pub fn finish(&self, slot: usize) -> Finish {
        finish_at(Finish { color: self.color, texture: self.texture }, &self.finishes, self.kind.slots(), slot)
    }
    pub fn finish_mut(&mut self, slot: usize) -> Option<(&mut Rgb, &mut usize)> {
        finish_at_mut((&mut self.color, &mut self.texture), &mut self.finishes, self.kind.slots(), slot)
    }
    /// Boîte englobante dans le repère de l'objet (x = largeur, y = hauteur, z = profondeur,
    /// avant vers +z), tirée de sa géométrie réelle.
    pub fn local_box(&self) -> (Vec3, Vec3) {
        let local = Prop { pos: Vec2::ZERO, rot: 0.0, ..self.clone() };
        prop_parts(&local, ElemRef::Prop(0), 0.0, 0.0)
            .into_iter()
            .filter(|part| part.shape != Shape::Light)
            .flat_map(|part| {
                let tf = part.bounds();
                [-0.5, 0.5].into_iter().flat_map(move |x| {
                    [-0.5, 0.5]
                        .into_iter()
                        .flat_map(move |y| [-0.5, 0.5].map(|z| tf.transform_point(Vec3::new(x, y, z))))
                })
            })
            .fold((Vec3::MAX, Vec3::MIN), |(lo, hi), c| (lo.min(c), hi.max(c)))
    }
    /// Coins au sol `[x, z]` de l'emprise, dans le monde.
    pub fn corners(&self) -> [Vec2; 4] {
        let (lo, hi) = self.local_box();
        let (s, c) = self.rot.to_radians().sin_cos();
        [(lo.x, lo.z), (hi.x, lo.z), (hi.x, hi.z), (lo.x, hi.z)]
            .map(|(x, z)| self.pos + Vec2::new(x * c + z * s, -x * s + z * c))
    }
}

impl PropKind {
    /// Encombrement `[largeur x, profondeur z, hauteur]` à `rot` 0 et hauteur par défaut.
    pub fn size(self) -> Vec3 {
        let (lo, hi) = Prop::new(self, Vec2::ZERO, 0, 0.0).local_box();
        let s = hi - lo;
        Vec3::new(s.x, s.z, s.y)
    }
}

fn level_h() -> f32 {
    LEVEL_H
}

/// Pièces d'un objet posé à l'altitude `y` (voir `House::prop_y`).
pub fn prop_parts(p: &Prop, owner: ElemRef, open: f32, y: f32) -> Vec<Part> {
    let base = Transform::from_xyz(p.pos.x, y, p.pos.y).with_rotation(Quat::from_rotation_y(p.rot.to_radians()));
    prop_shapes(p.kind, open, p.height)
        .into_iter()
        .map(|pc| {
            let Finish { color, texture } = p.finish(pc.slot);
            let local = Transform::from_translation(pc.at).with_rotation(pc.rot).with_scale(pc.size);
            Part { tf: base.mul_transform(local), shape: pc.shape, color, texture, owner, slot: pc.slot }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn local_frame_and_corners_agree() {
        let p = Prop::new(PropKind::Bed, Vec2::new(5.0, 2.0), 0, 90.0);
        let (lo, hi) = p.corners().iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), c| (lo.min(*c), hi.max(*c)));
        assert!((hi - lo).abs_diff_eq(Vec2::new(2.0, 1.6), 0.05), "lit tourné : {lo} {hi}");
        let lamp = PropKind::StreetLamp.size();
        assert!(lamp.z > 4.0 && PropKind::StreetLamp.outdoor() && PropKind::Shelf.size().y < 0.5);
    }
    #[test]
    fn each_part_keeps_its_own_finish() {
        let mut p = Prop::new(PropKind::Sofa, Vec2::ZERO, 0, 0.0);
        assert_eq!(p.finish(1), PropKind::Sofa.slots()[1].finish(), "pieds d'origine");
        assert!(p.finish_mut(2).is_none(), "le canapé n'a que deux parties");
        *p.finish_mut(1).unwrap().0 = [1.0, 0.0, 0.0];
        assert_eq!(p.finishes.len(), 1);
        let parts = prop_parts(&p, ElemRef::Prop(0), 0.0, 0.0);
        for q in &parts {
            assert_eq!(q.color, if q.slot == 1 { [1.0, 0.0, 0.0] } else { p.color });
        }
        // une partie lointaine complète les finitions intermédiaires avec celles d'origine
        let mut car = Prop::new(PropKind::Car, Vec2::ZERO, 0, 0.0);
        *car.finish_mut(6).unwrap().1 = 3;
        assert_eq!(car.finishes.len(), 6);
        assert_eq!(car.finish(3), PropKind::Car.slots()[3].finish());
        assert_eq!(car.finish(6).texture, 3);
        assert_eq!(car.finish(9), car.finish(0), "partie inconnue : la principale");
        let trunk = Prop::new(PropKind::Tree, Vec2::ZERO, 0, 0.0).finish(1);
        assert_eq!(trunk.texture, crate::textures::find("Écorce"));
    }
    #[test]
    fn stairs_reach_their_height() {
        for kind in [PropKind::Stairs, PropKind::SpiralStairs, PropKind::LStairs] {
            for height in [1.0, 3.0, 4.5] {
                let p = Prop { height, ..Prop::new(kind, Vec2::ZERO, 0, 0.0) };
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
    fn every_kind_has_a_distinct_name() {
        fn distinct(names: Vec<&str>) {
            let mut sorted = names.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), names.len(), "{names:?}");
            assert!(names.iter().all(|n| !n.is_empty()));
        }
        distinct(PropKind::ALL.map(PropKind::name).to_vec());
        distinct(RoofKind::ALL.map(RoofKind::name).to_vec());
        distinct(OpeningKind::ALL.map(OpeningKind::name).to_vec());
        distinct(crate::terrain::Brush::ALL.map(crate::terrain::Brush::name).to_vec());
        for k in OpeningKind::ALL {
            let (w, h, sill) = k.default_dims();
            assert!(w > 0.0 && h > 0.0 && sill >= 0.0 && sill + h <= LEVEL_H, "{k:?}");
        }
        assert!(PropKind::Pine.outdoor() && !PropKind::Sofa.outdoor());
        assert!(PropKind::Pillar.dim().is_some() && PropKind::Bed.dim().is_none());
    }
}
