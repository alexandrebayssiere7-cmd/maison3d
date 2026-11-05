//! Meubles, escaliers, végétaux et voitures posés dans la maison ou le jardin.

use bevy::math::{Quat, Vec2, Vec3};
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::shapes::prop_shapes;
use super::{BARK, BIRCH_BARK, ElemRef, LEVEL_H, Part, Rgb, Shape, WHITE, WHITE_TEX};

/// Type d'objet posé. À `rot` 0, l'avant de l'objet regarde +z (le sud). Encombrement, hauteur
/// et extérieur ou non : `maison3d catalog`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub enum PropKind {
    /// Table de repas rectangulaire.
    Table,
    /// Chaise ; l'avant est le côté où l'on s'assoit (dossier à l'arrière). À tourner vers la table.
    Chair,
    /// Canapé trois places ; l'avant est l'assise, à tourner vers la pièce ou la télé, dossier contre un mur.
    Sofa,
    /// Lit double ; la tête est à l'arrière (à coller au mur), les pieds vers l'avant.
    Bed,
    /// Armoire, s'ouvre pendant la visite ; dos contre un mur, portes vers l'avant.
    Wardrobe,
    /// Bloc cuisine (meubles bas, évier, plaques), placards ouvrables ; dos contre un mur.
    Kitchen,
    /// Réfrigérateur, s'ouvre pendant la visite ; dos contre un mur.
    Fridge,
    /// Baignoire.
    Bathtub,
    /// WC ; réservoir à l'arrière, contre un mur.
    Toilet,
    /// Escalier droit qui monte vers l'avant sur `height` mètres (3 = un étage). Le palier d'arrivée doit tomber dans un sol de l'étage au-dessus.
    Stairs,
    /// Pilier carré de 30 cm, haut de `height` mètres (portique, auvent, véranda).
    Pillar,
    /// Lampe sur pied d'intérieur (1,85 m) qui éclaire réellement la pièce. Pour l'éclairage public : `StreetLamp`.
    Lamp,
    /// Plante d'intérieur en pot.
    Plant,
    /// Arbre feuillu.
    Tree,
    /// Buisson rond.
    Bush,
    /// Sapin.
    Pine,
    /// Module de haie de 2 m de long (axe gauche-droite de l'objet). Aligner des modules tous les 2 m pour une haie continue ; `rot` 90 pour une haie nord-sud.
    Hedge,
    /// Module de clôture de 2 m de long, à aligner comme la haie.
    Fence,
    /// Berline de 4,2 m que l'on peut conduire pendant la visite ; à poser sur une route ou une allée.
    Car,
    /// Escalier en colimaçon qui monte de `height` mètres, encombrement réduit.
    SpiralStairs,
    /// Escalier quart tournant : une volée vers l'avant jusqu'à un palier, puis une volée vers le +x local de l'objet (l'est à rot 0), sur `height` mètres.
    LStairs,
    /// Bouleau.
    Birch,
    /// Palmier.
    Palm,
    /// Cyprès, haut et étroit (alignements, bords de terrain).
    Cypress,
    /// Pommier.
    AppleTree,
    /// Massif de fleurs.
    Flowers,
    /// Rosier.
    Rosebush,
    /// Tournesols.
    Sunflower,
    /// Lampadaire de rue de 4,5 m qui éclaire réellement ; la lanterne avance vers l'avant de l'objet
    /// (à tourner vers la chaussée). Un tous les 10 à 15 m sur un trottoir.
    StreetLamp,
    /// Étagère ouverte de magasin ou de bibliothèque (1 × 0,4 × 1,8 m), dos contre un mur ou dos à
    /// dos avec une autre pour former un rayon.
    Shelf,
}

impl PropKind {
    pub const ALL: [PropKind; 30] = [
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
        Self::Car,
        Self::SpiralStairs,
        Self::LStairs,
        Self::Birch,
        Self::Palm,
        Self::Cypress,
        Self::AppleTree,
        Self::Flowers,
        Self::Rosebush,
        Self::Sunflower,
        Self::StreetLamp,
        Self::Shelf,
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
            Self::Lamp => "Lampe sur pied",
            Self::Plant => "Plante",
            Self::Tree => "Arbre",
            Self::Bush => "Buisson",
            Self::Pine => "Sapin",
            Self::Hedge => "Haie",
            Self::Fence => "Clôture",
            Self::Car => "Voiture",
            Self::SpiralStairs => "Colimaçon",
            Self::LStairs => "Quart tournant",
            Self::Birch => "Bouleau",
            Self::Palm => "Palmier",
            Self::Cypress => "Cyprès",
            Self::AppleTree => "Pommier",
            Self::Flowers => "Fleurs",
            Self::Rosebush => "Rosier",
            Self::Sunflower => "Tournesols",
            Self::StreetLamp => "Lampadaire de rue",
            Self::Shelf => "Étagère",
        }
    }
    /// Pousse dehors : posé sur le relief du terrain plutôt qu'à plat.
    pub fn outdoor(self) -> bool {
        use PropKind::{
            AppleTree, Birch, Bush, Car, Cypress, Fence, Flowers, Hedge, Palm, Pine, Rosebush, StreetLamp, Sunflower,
            Tree,
        };
        matches!(
            self,
            Car | StreetLamp
                | Tree
                | Bush
                | Pine
                | Hedge
                | Fence
                | Birch
                | Palm
                | Cypress
                | AppleTree
                | Flowers
                | Rosebush
                | Sunflower
        )
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
            Self::Wardrobe | Self::Shelf => [0.72, 0.56, 0.4],
            Self::Kitchen => [0.88, 0.88, 0.85],
            Self::Fridge | Self::Bathtub | Self::Toilet => WHITE,
            Self::Stairs | Self::LStairs | Self::Fence => [0.6, 0.45, 0.3],
            Self::SpiralStairs => [0.35, 0.36, 0.38],
            Self::Flowers => [0.85, 0.15, 0.25],
            Self::Rosebush => [0.95, 0.45, 0.6],
            Self::Sunflower => [0.98, 0.8, 0.1],
            Self::Car => [0.72, 0.08, 0.1],
            Self::Pillar => [0.85, 0.85, 0.8],
            Self::Lamp => [1.0, 0.9, 0.7],
            Self::StreetLamp => [0.2, 0.22, 0.25],
            // le feuillage est coloré par sa texture : teinte blanche
            Self::Plant
            | Self::Tree
            | Self::Bush
            | Self::Pine
            | Self::Hedge
            | Self::Birch
            | Self::Palm
            | Self::Cypress
            | Self::AppleTree => WHITE_TEX,
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
    /// Hauteur montée par un escalier ou hauteur d'un pilier, en mètres (3 par défaut) ;
    /// ignorée pour les autres objets.
    #[serde(default = "level_h")]
    pub height: f32,
}

impl Prop {
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
        let p = Prop { kind: self, pos: Vec2::ZERO, level: 0, rot: 0.0, color: WHITE, texture: 0, height: LEVEL_H };
        let (lo, hi) = p.local_box();
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
        .map(|(shape, c, s, color, rot)| Part {
            tf: base
                .mul_transform(Transform::from_translation(c).with_rotation(Quat::from_rotation_y(rot)).with_scale(s)),
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

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn local_frame_and_corners_agree() {
        let p = Prop {
            kind: PropKind::Bed,
            pos: Vec2::new(5.0, 2.0),
            level: 0,
            rot: 90.0,
            color: WHITE,
            texture: 0,
            height: 3.0,
        };
        let (lo, hi) = p.corners().iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), c| (lo.min(*c), hi.max(*c)));
        assert!((hi - lo).abs_diff_eq(Vec2::new(2.0, 1.6), 0.05), "lit tourné : {lo} {hi}");
        let lamp = PropKind::StreetLamp.size();
        assert!(lamp.z > 4.0 && PropKind::StreetLamp.outdoor() && PropKind::Shelf.size().y < 0.5);
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
        assert!(PropKind::Pillar.has_height() && !PropKind::Bed.has_height());
    }
}
