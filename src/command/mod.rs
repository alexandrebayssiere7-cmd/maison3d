//! Commandes d'édition : le vocabulaire commun à l'éditeur (souris) et aux agents (JSON).
//!
//! Toute modification de la maison passe par `House::apply`, qui valide et renvoie
//! une erreur lisible plutôt que de laisser un monde incohérent.

use bevy::math::Vec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::{
    ElemRef, House, LEVEL_H, Opening, OpeningKind, Pool, Prop, PropKind, Rgb, Road, Roof, RoofKind, RoomName, Side,
    Slab, WHITE_TEX, Wall, room_walls, same_name,
};
use crate::textures::find;

mod error;
mod validate;

pub use error::EditError;

/// Une modification du monde. En JSON : `{"op": "add_wall", "a": [0, 0], "b": [4, 0], ...}`.
///
/// Conventions : mètres et degrés. Un point au sol est `[x, z]` (y est la verticale) ; la vue de
/// dessus de `maison3d shot` met -z (le nord) en haut. `level` va de -1 (sous-sol) à 9 et chaque
/// étage fait 3 m : le sol de `level` est à `3 × level` m. `color` est `[r, g, b]` entre 0 et 1
/// (`[1, 1, 1]` montre la texture telle quelle) ; `texture` vaut 0 (aucune) ou un indice de
/// `maison3d textures`. Tout point doit rester sur le terrain : x et z entre -80 et 80 m.
/// Un champ inconnu (faute de frappe) fait refuser la commande.
///
/// Les éléments sont désignés par leur indice dans leur liste (`{"Wall": 0}`,
/// `{"Opening": [mur, n]}`), affiché par `maison3d describe` ; `apply` affiche ceux que crée
/// chaque commande. Une suppression décale les indices suivants de la même liste.
///
/// Préférer `room` et les `place_*`, qui remplissent dimensions et matières par défaut
/// (`maison3d catalog` les liste) ; les `add_*` demandent tous les champs.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    /// Pièce rectangulaire : 4 murs (centrés sur le bord du rectangle) et un sol. Crée les murs
    /// dans l'ordre nord (z = min), est (x = max), sud (z = max), ouest (x = min), chacun de `a`
    /// vers `b` dans le sens horaire vu du dessus, puis le sol. Deux pièces accolées ont chacune
    /// leur mur sur la cloison commune : pour un logement à plusieurs pièces, une `room` pour
    /// l'enveloppe puis des `add_wall` pour les cloisons.
    Room {
        /// Coin nord-ouest `[x, z]` (plus petites coordonnées).
        #[schemars(with = "[f32; 2]")]
        min: Vec2,
        /// Coin sud-est `[x, z]`, strictement plus grand que `min` sur chaque axe.
        #[schemars(with = "[f32; 2]")]
        max: Vec2,
        /// Étage (-1 à 9).
        level: i32,
        /// Hauteur des murs en mètres, 0,1 à 30 (3 par défaut : un étage plein).
        #[serde(default)]
        height: Option<f32>,
        /// Épaisseur des murs en mètres, 0,02 à 3 (0,2 par défaut).
        #[serde(default)]
        thick: Option<f32>,
        /// Matière des murs (`maison3d textures`, « Crépi blanc » par défaut).
        #[serde(default)]
        wall_texture: Option<usize>,
        /// Matière du sol (« Parquet chêne clair » par défaut).
        #[serde(default)]
        floor_texture: Option<usize>,
        /// Nom de la pièce (« cuisine », « chambre 1 »), posé en son centre comme `name_room`.
        #[serde(default)]
        name: Option<String>,
    },
    /// Nomme la pièce qui contient le point `at` : l'espace fermé par les murs autour de lui,
    /// cloisons comprises. `maison3d query` la retrouve ensuite par ce nom. Redonner un nom déjà
    /// pris le déplace. Ne crée ni ne change aucun élément.
    NameRoom {
        /// Nom, 1 à 40 caractères (« cuisine », « chambre 1 »).
        name: String,
        /// Étage (-1 à 9).
        level: i32,
        /// Un point `[x, z]` à l'intérieur de la pièce, hors des murs.
        #[schemars(with = "[f32; 2]")]
        at: Vec2,
    },
    /// Retire un nom posé par `name_room` (ou par le champ `name` de `room`). Ne change aucun
    /// élément. Pour renommer : `unname_room`, puis `name_room` avec le nouveau nom.
    UnnameRoom {
        /// Nom à retirer, sans tenir compte de la casse ni des accents.
        name: String,
    },
    /// Porte ou fenêtre aux dimensions et à la couleur par défaut de son type (`maison3d catalog`).
    /// Refusée si elle chevauche une autre ouverture ou sort de la plage permise : 5 cm plus la
    /// demi-épaisseur du mur à chaque bout (le message donne la plage de `t` valide).
    PlaceOpening {
        /// Indice du mur (`Wall n` dans `describe`).
        wall: usize,
        kind: OpeningKind,
        /// Position du centre de l'ouverture, en mètres depuis l'extrémité `a` du mur.
        t: f32,
    },
    /// Meuble, escalier, végétal, lampadaire ou voiture, avec sa couleur et sa matière par défaut.
    /// Les éléments d'extérieur du rez-de-chaussée se posent sur le relief.
    PlaceProp {
        kind: PropKind,
        /// Centre de l'objet `[x, z]`.
        #[schemars(with = "[f32; 2]")]
        pos: Vec2,
        /// Étage (-1 à 9).
        level: i32,
        /// Rotation en degrés autour de la verticale (0 par défaut). À 0 l'avant de l'objet
        /// regarde +z (le sud) ; plus généralement l'avant pointe vers `[sin rot, cos rot]`.
        /// 90 = vers +x (l'est), 180 = vers -z (le nord), 270 = vers -x (l'ouest).
        #[serde(default)]
        rot: f32,
        /// Échelle uniforme, 0,2 à 5 (1 par défaut) : multiplie toutes les dimensions de l'objet.
        #[serde(default)]
        scale: Option<f32>,
    },
    /// Objet posé dos à un mur, avant tourné à l'opposé du mur, sur l'étage du mur : `pos` et
    /// `rot` sont calculés (arrière contre la face du mur). Refusé si l'objet dépasse du mur en
    /// longueur (le message donne la plage de `t` valide). Exemple : lit tête contre le mur nord.
    PlaceAgainst {
        kind: PropKind,
        /// Indice du mur (`Wall n` dans `describe`).
        wall: usize,
        /// Position du centre de l'objet le long du mur, en mètres depuis l'extrémité `a`.
        t: f32,
        /// Côté du mur où poser l'objet. Absent : le côté où un sol (`Slab`) du même étage borde
        /// le mur ; refusé si les deux côtés en ont un (cloison) ou aucun.
        #[serde(default)]
        side: Option<Side>,
    },
    /// Rangée d'objets identiques posés de `from` à `to` (extrémités comprises) tous les `step`
    /// mètres : haie, clôture, arbres d'alignement, lampadaires. Renvoie le dernier posé.
    PlaceRow {
        kind: PropKind,
        /// Centre du premier objet `[x, z]`.
        #[schemars(with = "[f32; 2]")]
        from: Vec2,
        /// Centre du dernier objet `[x, z]`.
        #[schemars(with = "[f32; 2]")]
        to: Vec2,
        /// Espacement en mètres, 0,1 ou plus (2 pour des modules de haie ou de clôture jointifs) ;
        /// 200 objets au plus.
        step: f32,
        /// Étage (-1 à 9).
        level: i32,
        /// Rotation en degrés. Absente : l'axe gauche-droite des objets suit la rangée (modules
        /// de haie et de clôture bout à bout ; l'avant d'un lampadaire regarde à droite de la
        /// direction `from` → `to`).
        #[serde(default)]
        rot: Option<f32>,
    },
    /// Toit de 2,5 m de haut (1,5 m pour des sheds, 1,2 m pour un papillon, 30 cm pour un toit
    /// plat) avec 50 cm de débord. Il se pose à la base de `level` : pour couvrir
    /// des murs du rez-de-chaussée, `level` = 1. Le faîtage suit le grand côté.
    PlaceRoof {
        /// Coin nord-ouest de l'emprise des murs ; le débord est ajouté autour.
        #[schemars(with = "[f32; 2]")]
        min: Vec2,
        /// Coin sud-est de l'emprise des murs.
        #[schemars(with = "[f32; 2]")]
        max: Vec2,
        /// Étage sur lequel repose le toit : celui juste au-dessus des murs couverts.
        level: i32,
        kind: RoofKind,
        /// Matière (« Tuiles terre cuite » par défaut, « Zinc » pour un toit plat).
        #[serde(default)]
        texture: Option<usize>,
    },
    /// Mur droit, tous champs explicites. Pour une cloison intérieure, mettre ses extrémités sur
    /// les murs existants. Au rez-de-chaussée, le relief est aplani autour.
    AddWall(Wall),
    /// Sol rectangulaire (plancher, dalle, terrasse) posé au niveau de `level`. Au
    /// rez-de-chaussée, le relief est aplani autour. Pour une trémie d'escalier, juxtaposer
    /// plusieurs sols autour du trou.
    AddSlab(Slab),
    /// Toit, tous champs explicites (voir `place_roof`).
    AddRoof(Roof),
    /// Objet, tous champs explicites (voir `place_prop`).
    AddProp(Prop),
    /// Piscine creusée au rez-de-chaussée. Coins en mètres entiers (grille du relief) ; le
    /// terrain est aplani dessous.
    AddPool(Pool),
    /// Tronçon de route droit au rez-de-chaussée ; le terrain est terrassé dessous. Enchaîner
    /// plusieurs tronçons qui partagent leurs extrémités pour un tracé courbe. Une allée qui
    /// rejoint une route s'arrête à son bord (à `width / 2` de son axe).
    AddRoad(Road),
    /// Ouverture aux dimensions choisies (voir `place_opening` pour les règles de place).
    AddOpening {
        /// Indice du mur.
        wall: usize,
        opening: Opening,
    },
    /// Insère un étage au-dessus de `level` : copie ses murs (fenêtres, baies et
    /// portes-fenêtres comprises, sans les portes `Door`) et ses sols intérieurs, pas les objets ;
    /// remonte de 3 m tout ce qui était plus haut, toits compris. Les indices existants ne
    /// changent pas ; les copies sont ajoutées à la fin, dans l'ordre des originaux. Dans un
    /// monde à plusieurs bâtiments, donner `min` et `max` pour n'agir que sur l'un d'eux.
    AddStorey {
        /// Étage à copier, de -1 à 8 ; refusé si un élément concerné est déjà à l'étage 9.
        level: i32,
        /// Coin nord-ouest de la zone concernée, à donner avec `max` (les deux absents : tout le monde).
        #[serde(default)]
        #[schemars(with = "Option<[f32; 2]>")]
        min: Option<Vec2>,
        /// Coin sud-est de la zone concernée, à donner avec `min`.
        #[serde(default)]
        #[schemars(with = "Option<[f32; 2]>")]
        max: Option<Vec2>,
    },
    /// Remplace le relief par un relief aléatoire. Le rectangle qui englobe tous les murs et sols
    /// reste plat, ainsi que 3 m autour, puis le relief ne remonte pas plus vite qu'un talus à
    /// 45° (1 m par mètre) ; routes et piscines sont terrassées à nouveau. Le relief retombe à 0
    /// sur les bords du terrain. Un bâtiment ajouté ou déplacé ensuite aplanit le relief autour
    /// de lui selon la même règle.
    GenerateTerrain {
        /// Graine : la même graine redonne le même relief.
        seed: u32,
        /// Amplitude en mètres, 0 à 60 (l'éditeur propose 8).
        amp: f32,
        /// Taille typique d'une colline en mètres, 5 à 200 (l'éditeur propose 40).
        scale: f32,
        /// `true` : montagnes à crêtes ; `false` : collines douces.
        mountains: bool,
    },
    /// Remet tout le terrain à plat, à l'altitude 0.
    FlattenTerrain,
    /// Supprime un élément (un mur emporte ses ouvertures). Décale de 1 les indices suivants
    /// de la même liste : dans un lot, supprimer du plus grand indice au plus petit.
    Remove { target: ElemRef },
    /// Déplace un élément. Une ouverture glisse le long de son mur (seule la composante de
    /// `by` dans l'axe du mur compte) et la commande est refusée si elle ne tient plus ; une
    /// piscine se déplace par mètres entiers (`by` arrondi). Le relief suit l'élément déplacé :
    /// terrassé sous une route ou une piscine, aplani autour d'un mur ou d'un sol du
    /// rez-de-chaussée.
    Move {
        target: ElemRef,
        /// Décalage `[dx, dz]` en mètres.
        #[schemars(with = "[f32; 2]")]
        by: Vec2,
    },
    /// Copie un élément (pas une ouverture) décalé de `by` ; renvoie la référence de la copie.
    /// Le relief suit la copie comme pour `move`.
    Duplicate {
        target: ElemRef,
        /// Décalage de la copie `[dx, dz]` en mètres.
        #[schemars(with = "[f32; 2]")]
        by: Vec2,
    },
    /// Fait tourner un objet (`Prop` uniquement) sur lui-même. Pour un mur, utiliser `set` sur
    /// `a` et `b`.
    Rotate {
        target: ElemRef,
        /// Degrés ajoutés à sa rotation actuelle (voir `place_prop.rot` pour le sens).
        deg: f32,
    },
    /// Change la couleur et/ou la matière d'un élément, ou d'une seule de ses parties : un objet
    /// ou une ouverture en a plusieurs (tissu et pieds d'un canapé, battant et poignée d'une
    /// porte…), listées par `maison3d catalog` (`parts`). Peint `target` puis chaque élément de
    /// `targets` (au moins un des deux) ; refusée en entier si l'un d'eux n'existe pas.
    Paint {
        /// Élément à peindre.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<ElemRef>,
        /// Autres éléments à peindre de la même façon, par exemple les 4 murs d'une pièce :
        /// `"targets": [{"Wall": 0}, {"Wall": 1}, {"Wall": 2}, {"Wall": 3}]`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        targets: Vec<ElemRef>,
        /// Indice de la partie à peindre dans `parts` (0 par défaut : la partie principale). Murs,
        /// sols, toits, piscines et routes n'ont que la partie 0.
        #[serde(default)]
        part: usize,
        /// Nouvelle couleur `[r, g, b]` (0 à 1) ; absente ou `null` : inchangée.
        color: Option<Rgb>,
        /// Nouvel indice de matière (`maison3d textures`) ; absent ou `null` : inchangé.
        texture: Option<usize>,
    },
    /// Modifie des champs d'un élément existant, sans changer son indice : par exemple
    /// `{"op": "set", "target": {"Prop": 3}, "fields": {"height": 4.5}}`. Les noms et les bornes
    /// sont ceux de `add_wall`, `add_slab`, `add_roof`, `add_prop`, `add_pool`, `add_road` ou de
    /// l'ouverture ; le résultat est validé comme un ajout. Si l'emprise, l'étage ou la largeur
    /// change, le relief suit l'élément comme pour `move` ; sinon il n'est pas touché.
    Set {
        target: ElemRef,
        /// Champs à remplacer et leur nouvelle valeur.
        fields: serde_json::Map<String, serde_json::Value>,
    },
}

impl Command {
    /// L'élément existant que vise la commande, s'il y en a un.
    fn target(&self) -> Option<ElemRef> {
        match self {
            Command::Remove { target }
            | Command::Move { target, .. }
            | Command::Duplicate { target, .. }
            | Command::Rotate { target, .. }
            | Command::Set { target, .. } => Some(*target),
            _ => None,
        }
    }
}

impl House {
    /// Applique une commande ; renvoie l'élément créé ou modifié s'il y en a un.
    /// En cas d'erreur la maison est rendue intacte.
    pub fn apply(&mut self, cmd: Command) -> Result<Option<ElemRef>, EditError> {
        // une commande composée (pièce, rangée, set…) peut échouer à mi-chemin
        let before = self.clone();
        let res = self.apply_one(cmd);
        if res.is_err() {
            *self = before;
        }
        res
    }

    fn apply_one(&mut self, cmd: Command) -> Result<Option<ElemRef>, EditError> {
        if let Some(target) = cmd.target() {
            self.found(target)?;
        }
        match cmd {
            Command::Room { min, max, level, height, thick, wall_texture, floor_texture, name } => {
                let walls = (height.unwrap_or(LEVEL_H), thick.unwrap_or(0.2));
                let textures = (wall_texture.unwrap_or_else(|| find("Crépi blanc")), floor_texture);
                self.add_room(min, max, level, walls, textures, name)
            }
            Command::NameRoom { name, level, at } => self.name_room(name, level, at),
            Command::UnnameRoom { name } => self.unname_room(name),
            Command::PlaceOpening { wall, kind, t } => self.add_opening(wall, Opening::new(kind, t)),
            Command::PlaceProp { kind, pos, level, rot, scale } => {
                self.add_prop(Prop { scale: scale.unwrap_or(1.0), ..Prop::new(kind, pos, level, rot) })
            }
            Command::PlaceAgainst { kind, wall, t, side } => self.place_against(kind, wall, t, side),
            Command::PlaceRow { kind, from, to, step, level, rot } => {
                self.place_row(kind, (from, to), step, level, rot)
            }
            Command::PlaceRoof { min, max, level, kind, texture } => self.place_roof((min, max), level, kind, texture),
            Command::AddWall(w) => self.add_wall(w),
            Command::AddSlab(s) => self.add_slab(s),
            Command::AddRoof(r) => self.add_roof(r),
            Command::AddProp(p) => self.add_prop(p),
            Command::AddPool(p) => self.add_checked_pool(p),
            Command::AddRoad(r) => self.add_checked_road(r),
            Command::AddOpening { wall, opening } => self.add_opening(wall, opening),
            Command::AddStorey { level, min, max } => self.add_storey_checked(level, min, max),
            Command::GenerateTerrain { seed, amp, scale, mountains } => {
                self.generate_checked(seed, amp, scale, mountains)
            }
            Command::FlattenTerrain => {
                self.terrain = crate::terrain::Terrain::default();
                Ok(None)
            }
            Command::Remove { target } => {
                self.remove_settled(target);
                Ok(None)
            }
            Command::Move { target, by } => self.move_elem(target, by),
            Command::Duplicate { target, by } => self.duplicate_checked(target, by),
            Command::Rotate { target, deg } => self.rotate(target, deg),
            Command::Paint { target, targets, part, color, texture } => {
                self.paint_all(target.into_iter().chain(targets), part, color, texture)
            }
            Command::Set { target, fields } => self.set(target, fields),
        }
    }

    /// Toit avec la hauteur et la matière par défaut de son type (zinc pour un toit plat).
    fn place_roof(
        &mut self,
        (min, max): (Vec2, Vec2),
        level: i32,
        kind: RoofKind,
        texture: Option<usize>,
    ) -> Result<Option<ElemRef>, EditError> {
        let flat = kind == RoofKind::Flat;
        let texture = texture.unwrap_or_else(|| find(if flat { "Zinc" } else { "Tuiles terre cuite" }));
        let height = match kind {
            RoofKind::Flat => 0.3,
            RoofKind::Butterfly => 1.2,
            RoofKind::Sawtooth => 1.5,
            _ => 2.5,
        };
        self.add_roof(Roof { min, max, level, kind, height, overhang: 0.5, color: WHITE_TEX, texture, tag: None })
    }

    fn generate_checked(
        &mut self,
        seed: u32,
        amp: f32,
        scale: f32,
        mountains: bool,
    ) -> Result<Option<ElemRef>, EditError> {
        validate::range("amp", amp, 0.0, 60.0)?;
        validate::range("scale", scale, 5.0, 200.0)?;
        self.generate_terrain(seed, amp, scale, mountains);
        Ok(None)
    }

    fn duplicate_checked(&mut self, target: ElemRef, by: Vec2) -> Result<Option<ElemRef>, EditError> {
        validate::offset(by)?;
        let copy = self.duplicate(target, by).ok_or(EditError::OpeningNotDuplicable)?;
        self.check_elem(copy)?;
        self.settle(copy);
        Ok(Some(copy))
    }

    fn found(&self, target: ElemRef) -> Result<(), EditError> {
        if self.exists(target) { Ok(()) } else { Err(EditError::NotFound(target, self.count(target))) }
    }

    /// Peint chaque cible ; renvoie la dernière.
    fn paint_all(
        &mut self,
        targets: impl Iterator<Item = ElemRef>,
        part: usize,
        color: Option<Rgb>,
        texture: Option<usize>,
    ) -> Result<Option<ElemRef>, EditError> {
        let mut last = None;
        for target in targets {
            self.found(target)?;
            last = self.paint(target, part, color, texture)?;
        }
        last.map(Some).ok_or(EditError::NoTarget)
    }

    fn paint(
        &mut self,
        target: ElemRef,
        part: usize,
        color: Option<Rgb>,
        texture: Option<usize>,
    ) -> Result<Option<ElemRef>, EditError> {
        color.map_or(Ok(()), validate::color)?;
        texture.map_or(Ok(()), validate::texture)?;
        let count = self.part_count(target);
        let (c, t) = self.finish_mut(target, part).ok_or(EditError::NoSuchPart { part, count })?;
        if let Some(color) = color {
            *c = color;
        }
        if let Some(texture) = texture {
            *t = texture;
        }
        Ok(Some(target))
    }

    fn add_room(
        &mut self,
        min: Vec2,
        max: Vec2,
        level: i32,
        (height, thick): (f32, f32),
        (wall_texture, floor_texture): (usize, Option<usize>),
        name: Option<String>,
    ) -> Result<Option<ElemRef>, EditError> {
        validate::rect(min, max)?;
        for mut w in room_walls(min, max, level, height, thick, WHITE_TEX) {
            w.texture = wall_texture;
            self.add_wall(w)?;
        }
        let texture = floor_texture.unwrap_or_else(|| find("Parquet chêne clair"));
        let made = self.add_slab(Slab { min, max, level, color: WHITE_TEX, texture, tag: None })?;
        match name {
            Some(name) => self.name_room(name, level, (min + max) / 2.0).map(|_| made),
            None => Ok(made),
        }
    }

    /// Pose ou déplace le nom `name` (un même nom ne désigne qu'une pièce).
    fn name_room(&mut self, name: String, level: i32, at: Vec2) -> Result<Option<ElemRef>, EditError> {
        validate::room_name(&name)?;
        validate::level(level)?;
        validate::point(at)?;
        self.names.retain(|n| !same_name(&n.name, &name));
        self.names.push(RoomName { name, level, at });
        Ok(None)
    }

    fn unname_room(&mut self, name: String) -> Result<Option<ElemRef>, EditError> {
        let before = self.names.len();
        self.names.retain(|n| !same_name(&n.name, &name));
        if self.names.len() == before { Err(EditError::NoSuchName(name)) } else { Ok(None) }
    }

    fn add_wall(&mut self, w: Wall) -> Result<Option<ElemRef>, EditError> {
        validate::wall(&w)?;
        self.walls.push(w);
        Ok(Some(self.settle(ElemRef::Wall(self.walls.len() - 1))))
    }

    fn add_slab(&mut self, s: Slab) -> Result<Option<ElemRef>, EditError> {
        validate::slab(&s)?;
        self.slabs.push(s);
        Ok(Some(self.settle(ElemRef::Slab(self.slabs.len() - 1))))
    }

    /// Le relief suit l'élément posé, déplacé ou modifié : routes et piscines sont terrassées, et
    /// un mur ou un sol du rez-de-chaussée aplanit le relief autour pour ne pas s'y enterrer.
    fn settle(&mut self, e: ElemRef) -> ElemRef {
        if self.terrain.h.is_empty() {
            return e;
        }
        match e {
            ElemRef::Wall(i) if self.walls[i].level == 0 => {
                let w = &self.walls[i];
                self.terrain.keep_flat(w.a.min(w.b), w.a.max(w.b));
            }
            ElemRef::Slab(i) if self.slabs[i].level == 0 => {
                let s = &self.slabs[i];
                self.terrain.keep_flat(s.min, s.max);
            }
            ElemRef::Opening(..) | ElemRef::Roof(_) | ElemRef::Prop(_) => return e,
            // routes, piscines, et murs ou sols d'étage dont l'emprise au sol a pu changer
            _ => {}
        }
        self.regrade();
        e
    }

    /// Retire l'élément ; routes et piscines restantes lisaient peut-être le relief qu'il
    /// portait : elles sont terrassées à nouveau.
    fn remove_settled(&mut self, target: ElemRef) {
        self.remove(target);
        let ground = !matches!(target, ElemRef::Opening(..) | ElemRef::Roof(_) | ElemRef::Prop(_));
        if ground && !self.terrain.h.is_empty() {
            self.regrade();
        }
    }

    /// Ce qui pose l'élément sur le relief : ses deux points, et son étage ou sa largeur.
    fn footprint(&self, e: ElemRef) -> Option<(Vec2, Vec2, f32)> {
        match e {
            // sans ordre : un mur aux extrémités échangées occupe la même place
            ElemRef::Wall(i) => {
                let w = &self.walls[i];
                Some((w.a.min(w.b), w.a.max(w.b), w.level as f32))
            }
            ElemRef::Slab(i) => Some((self.slabs[i].min, self.slabs[i].max, self.slabs[i].level as f32)),
            ElemRef::Pool(i) => Some((self.pools[i].min, self.pools[i].max, 0.0)),
            ElemRef::Road(i) => Some((self.roads[i].a, self.roads[i].b, self.roads[i].width)),
            ElemRef::Opening(..) | ElemRef::Roof(_) | ElemRef::Prop(_) => None,
        }
    }

    /// `settle` seulement si l'élément a bougé : aplanir à nouveau sur place creuserait un peu
    /// plus le relief autour à chaque modification (couleur, matière…).
    fn settle_if_moved(&mut self, e: ElemRef, before: Option<(Vec2, Vec2, f32)>) -> ElemRef {
        if self.footprint(e) == before { e } else { self.settle(e) }
    }

    fn add_roof(&mut self, r: Roof) -> Result<Option<ElemRef>, EditError> {
        validate::roof(&r)?;
        self.roofs.push(r);
        Ok(Some(ElemRef::Roof(self.roofs.len() - 1)))
    }

    fn place_against(
        &mut self,
        kind: PropKind,
        wall: usize,
        t: f32,
        side: Option<Side>,
    ) -> Result<Option<ElemRef>, EditError> {
        let n = self.walls.len();
        let w = self.walls.get(wall).ok_or(EditError::WallNotFound(wall, n))?;
        let side = side.or_else(|| self.floor_side(w, t)).ok_or(EditError::SideUnknown(wall))?;
        let p = Prop::against(kind, w, t, side);
        let (lo, hi) = p.local_box();
        // même marge qu'une ouverture : l'objet ne mord pas sur les murs perpendiculaires
        let (t_min, t_max) = w.t_range(2.0 * hi.x.max(-lo.x));
        if !(t_min..=t_max).contains(&t) {
            return Err(EditError::PropOutsideWall { t, t_min, t_max, wall_len: w.len() });
        }
        self.add_prop(p)
    }

    fn add_prop(&mut self, mut p: Prop) -> Result<Option<ElemRef>, EditError> {
        validate::prop(&p)?;
        p.rot = p.rot.rem_euclid(360.0);
        self.props.push(p);
        Ok(Some(ElemRef::Prop(self.props.len() - 1)))
    }

    fn place_row(
        &mut self,
        kind: PropKind,
        (from, to): (Vec2, Vec2),
        step: f32,
        level: i32,
        rot: Option<f32>,
    ) -> Result<Option<ElemRef>, EditError> {
        validate::range("step", step, 0.1, 2.0 * crate::terrain::HALF)?;
        let d = to - from;
        let count = (d.length() / step + 1e-3).floor() as usize + 1;
        if count > 200 {
            return Err(EditError::TooMany { count, max: 200 });
        }
        // l'axe local +x devient (cos rot, -sin rot) : on l'aligne sur la rangée
        let rot = rot.unwrap_or_else(|| (-d.y).atan2(d.x).to_degrees());
        let mut last = None;
        for k in 0..count {
            let pos = from + d.normalize_or_zero() * step * k as f32;
            last = self.add_prop(Prop::new(kind, pos, level, rot))?;
        }
        Ok(last)
    }

    fn add_checked_pool(&mut self, p: Pool) -> Result<Option<ElemRef>, EditError> {
        validate::pool(&p)?;
        self.pools.push(p);
        Ok(Some(self.settle(ElemRef::Pool(self.pools.len() - 1))))
    }

    fn add_checked_road(&mut self, r: Road) -> Result<Option<ElemRef>, EditError> {
        validate::road(&r)?;
        self.roads.push(r);
        Ok(Some(self.settle(ElemRef::Road(self.roads.len() - 1))))
    }

    fn add_opening(&mut self, wall: usize, opening: Opening) -> Result<Option<ElemRef>, EditError> {
        let n = self.walls.len();
        let w = self.walls.get_mut(wall).ok_or(EditError::WallNotFound(wall, n))?;
        validate::opening(w, &opening, None)?;
        w.openings.push(opening);
        Ok(Some(ElemRef::Opening(wall, w.openings.len() - 1)))
    }

    fn add_storey_checked(
        &mut self,
        level: i32,
        min: Option<Vec2>,
        max: Option<Vec2>,
    ) -> Result<Option<ElemRef>, EditError> {
        validate::level(level)?;
        validate::level(level.saturating_add(1))?;
        let area = match (min, max) {
            (Some(lo), Some(hi)) => Some((lo, hi)),
            (None, None) => None,
            _ => return Err(EditError::HalfArea),
        };
        if let Some((lo, hi)) = area {
            validate::rect(lo, hi)?;
        }
        self.add_storey(level, area);
        // un élément déjà au dernier étage vient de monter trop haut : apply restaure la maison
        let walls = self.walls.iter().map(|w| w.level);
        let slabs = self.slabs.iter().map(|s| s.level);
        let roofs = self.roofs.iter().map(|r| r.level);
        let top = walls.chain(slabs).chain(roofs).chain(self.props.iter().map(|p| p.level)).max();
        top.map_or(Ok(()), validate::level)?;
        Ok(None)
    }

    fn move_elem(&mut self, target: ElemRef, by: Vec2) -> Result<Option<ElemRef>, EditError> {
        validate::offset(by)?;
        if let ElemRef::Opening(wi, oi) = target {
            let w = &self.walls[wi];
            let o = &w.openings[oi];
            validate::place(w, o.kind, o.t + by.dot(w.dir()), o.width, Some(oi))?;
        }
        let before = self.footprint(target);
        self.translate(target, by);
        self.check_elem(target)?;
        Ok(Some(self.settle_if_moved(target, before)))
    }

    fn rotate(&mut self, target: ElemRef, deg: f32) -> Result<Option<ElemRef>, EditError> {
        let ElemRef::Prop(i) = target else { return Err(EditError::NotRotatable) };
        validate::range("deg", deg, -36_000.0, 36_000.0)?;
        let p = &mut self.props[i];
        p.rot = (p.rot + deg).rem_euclid(360.0);
        Ok(Some(target))
    }

    fn set(
        &mut self,
        target: ElemRef,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> Result<Option<ElemRef>, EditError> {
        let before = self.footprint(target);
        match target {
            ElemRef::Wall(i) => self.walls[i] = merged(&self.walls[i], fields)?,
            ElemRef::Opening(w, o) => self.walls[w].openings[o] = merged(&self.walls[w].openings[o], fields)?,
            ElemRef::Slab(i) => self.slabs[i] = merged(&self.slabs[i], fields)?,
            ElemRef::Roof(i) => self.roofs[i] = merged(&self.roofs[i], fields)?,
            ElemRef::Prop(i) => self.props[i] = merged(&self.props[i], fields)?,
            ElemRef::Pool(i) => self.pools[i] = merged(&self.pools[i], fields)?,
            ElemRef::Road(i) => self.roads[i] = merged(&self.roads[i], fields)?,
        }
        self.check_elem(target)?;
        Ok(Some(self.settle_if_moved(target, before)))
    }
}

/// `elem` avec ses champs remplacés par `fields`, relu avec les mêmes règles qu'un ajout.
fn merged<T: Serialize + serde::de::DeserializeOwned>(
    elem: &T,
    fields: serde_json::Map<String, serde_json::Value>,
) -> Result<T, EditError> {
    let bad = |e: serde_json::Error| EditError::BadFields(e.to_string());
    let mut v = serde_json::to_value(elem).map_err(bad)?;
    if let Some(obj) = v.as_object_mut() {
        obj.extend(fields);
    }
    serde_json::from_value(v).map_err(bad)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod settle_tests;

#[cfg(test)]
mod tag_tests;
