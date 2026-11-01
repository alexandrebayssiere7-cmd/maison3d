//! Commandes d'édition : le vocabulaire commun à l'éditeur (souris) et aux agents (JSON).
//!
//! Toute modification de la maison passe par `House::apply`, qui valide et renvoie
//! une erreur lisible plutôt que de laisser un monde incohérent.

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::model::*;

/// Une modification de la maison. En JSON : `{"op": "add_wall", "a": [0, 0], "b": [4, 0], ...}`.
/// Unités : mètres et degrés ; `level` va de -1 (sous-sol) à 9.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Command {
    AddWall(Wall),
    AddSlab(Slab),
    AddRoof(Roof),
    AddProp(Prop),
    /// Coins entiers (grille du relief) ; le terrain est aplani dessous.
    AddPool(Pool),
    /// Le terrain est terrassé sous la route.
    AddRoad(Road),
    /// `wall` : indice du mur ; l'ouverture doit tenir sans chevaucher les autres.
    AddOpening { wall: usize, opening: Opening },
    /// Copie murs (sans ouvertures) et sols de `level` à l'étage au-dessus, remonte le toit.
    AddStorey { level: i32 },
    GenerateTerrain { seed: u32, amp: f32, scale: f32, mountains: bool },
    FlattenTerrain,
    Remove { target: ElemRef },
    Move { target: ElemRef, by: Vec2 },
    /// Renvoie la référence de la copie.
    Duplicate { target: ElemRef, by: Vec2 },
    /// Meubles uniquement.
    Rotate { target: ElemRef, deg: f32 },
    Paint { target: ElemRef, color: Option<Rgb>, texture: Option<usize> },
}

fn level_ok(l: i32) -> Result<(), String> {
    if (-1..=9).contains(&l) { Ok(()) } else { Err(format!("Étage {l} hors limites (-1 à 9)")) }
}

fn rect_ok(min: Vec2, max: Vec2) -> Result<(), String> {
    if min.cmplt(max).all() { Ok(()) } else { Err(format!("Rectangle vide : min {min} doit être < max {max}")) }
}

impl House {
    /// Applique une commande ; renvoie l'élément créé s'il y en a un.
    /// En cas d'erreur la maison peut être partiellement modifiée : travailler sur une copie.
    pub fn apply(&mut self, cmd: Command) -> Result<Option<ElemRef>, String> {
        // color_mut couvre toutes les sortes d'éléments : sert de test d'existence
        if let Command::Remove { target } | Command::Move { target, .. } | Command::Duplicate { target, .. } | Command::Rotate { target, .. } | Command::Paint { target, .. } = cmd {
            if self.color_mut(target).is_none() {
                return Err(format!("Élément introuvable : {target:?}"));
            }
        }
        Ok(match cmd {
            Command::AddWall(w) => {
                level_ok(w.level)?;
                if w.len() < 0.05 {
                    return Err("Mur trop court (min 5 cm)".into());
                }
                self.walls.push(w);
                Some(ElemRef::Wall(self.walls.len() - 1))
            }
            Command::AddSlab(s) => {
                level_ok(s.level)?;
                rect_ok(s.min, s.max)?;
                self.slabs.push(s);
                Some(ElemRef::Slab(self.slabs.len() - 1))
            }
            Command::AddRoof(r) => {
                level_ok(r.level)?;
                rect_ok(r.min, r.max)?;
                self.roofs.push(r);
                Some(ElemRef::Roof(self.roofs.len() - 1))
            }
            Command::AddProp(p) => {
                level_ok(p.level)?;
                self.props.push(p);
                Some(ElemRef::Prop(self.props.len() - 1))
            }
            Command::AddPool(p) => {
                rect_ok(p.min, p.max)?;
                if p.min != p.min.round() || p.max != p.max.round() {
                    return Err("Piscine : coins au mètre près exigés".into());
                }
                self.add_pool(p);
                Some(ElemRef::Pool(self.pools.len() - 1))
            }
            Command::AddRoad(r) => {
                if r.a.distance(r.b) < 0.05 {
                    return Err("Route trop courte".into());
                }
                self.add_road(r);
                Some(ElemRef::Road(self.roads.len() - 1))
            }
            Command::AddOpening { wall, opening } => {
                let w = self.walls.get_mut(wall).ok_or(format!("Mur introuvable : {wall}"))?;
                if !w.fits(opening.t, opening.width, None) {
                    return Err(format!("Pas la place pour : {} (mur de {:.2} m)", opening.kind.name(), w.len()));
                }
                w.openings.push(opening);
                Some(ElemRef::Opening(wall, w.openings.len() - 1))
            }
            Command::AddStorey { level } => {
                level_ok(level + 1)?;
                self.add_storey(level);
                None
            }
            Command::GenerateTerrain { seed, amp, scale, mountains } => {
                self.generate_terrain(seed, amp, scale, mountains);
                None
            }
            Command::FlattenTerrain => {
                self.terrain = Default::default();
                None
            }
            Command::Remove { target } => {
                self.remove(target);
                None
            }
            Command::Move { target, by } => {
                self.translate(target, by);
                Some(target)
            }
            Command::Duplicate { target, by } => {
                Some(self.duplicate(target, by).ok_or("Les ouvertures ne se dupliquent pas")?)
            }
            Command::Rotate { target, deg } => {
                let ElemRef::Prop(i) = target else { return Err("Seuls les meubles pivotent".into()) };
                let p = &mut self.props[i];
                p.rot = (p.rot + deg).rem_euclid(360.0);
                Some(target)
            }
            Command::Paint { target, color, texture } => {
                if let (Some(c), Some(col)) = (color, self.color_mut(target)) {
                    *col = c;
                }
                if let (Some(t), Some(tex)) = (texture, self.texture_mut(target)) {
                    *tex = t;
                }
                Some(target)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_commands_build_a_room() {
        let cmds: Vec<Command> = serde_json::from_str(
            r#"[
                {"op": "add_wall", "a": [0, 0], "b": [4, 0], "level": 0, "height": 3, "thick": 0.2, "color": [1, 1, 1], "openings": []},
                {"op": "add_opening", "wall": 0, "opening": {"kind": "Door", "t": 2, "width": 0.9, "height": 2.1, "sill": 0, "color": [1, 1, 1]}},
                {"op": "add_opening", "wall": 0, "opening": {"kind": "Door", "t": 2.5, "width": 0.9, "height": 2.1, "sill": 0, "color": [1, 1, 1]}}
            ]"#,
        )
        .unwrap();
        let mut h = House::default();
        let mut it = cmds.into_iter();
        assert_eq!(h.apply(it.next().unwrap()), Ok(Some(ElemRef::Wall(0))));
        assert_eq!(h.apply(it.next().unwrap()), Ok(Some(ElemRef::Opening(0, 0))));
        // chevauche la première porte
        assert!(h.apply(it.next().unwrap()).is_err());
        assert!(h.apply(Command::Remove { target: ElemRef::Prop(3) }).is_err());
    }
}
