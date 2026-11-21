//! État de l'éditeur, historique d'annulation et interactions souris dans le monde 3D.

mod input;
mod panel;
mod tools;

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

pub use input::editor_input;

use crate::command::{Command, EditError};
use crate::model::{ElemRef, House, LEVEL_H, OpeningKind, Part, PropKind, Rgb, RoofKind, WHITE_TEX};
use crate::terrain::Brush;
use crate::textures::find;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Wall,
    Room,
    Floor,
    Roof,
    Pool,
    Road,
    Terrain,
    Opening(OpeningKind),
    Prop(PropKind),
    Paint,
    Eyedropper,
    Erase,
    Visitor,
}

impl Tool {
    pub fn hint(self) -> &'static str {
        match self {
            Tool::Select => {
                "Clic : sélectionner  •  Glisser : déplacer  •  Suppr : effacer  •  R : pivoter  •  Ctrl+D : dupliquer  •  Flèches : décaler"
            }
            Tool::Wall => {
                "Glisser : un mur  •  ou clic coin après coin (les murs s'enchaînent)  •  Échap / clic droit : terminer  •  Maj : sans grille"
            }
            Tool::Room => "Glisser un rectangle (ou cliquer 2 coins opposés) : 4 murs + le sol d'une pièce",
            Tool::Floor => "Glisser un rectangle (ou cliquer 2 coins opposés) : un sol (plancher, dalle, terrasse)",
            Tool::Roof => {
                "Glisser un rectangle (ou 2 clics) au-dessus des murs — montez d'un étage avec + pour le poser sur les murs"
            }
            Tool::Pool => "Glisser un rectangle (ou 2 clics) : une piscine creusée dans le terrain, coins au mètre",
            Tool::Road => {
                "Glisser : un tronçon  •  ou clic point après point (la route s'enchaîne)  •  Échap / clic droit : terminer"
            }
            Tool::Terrain => "Clic maintenu : sculpter le relief avec le pinceau  •  réglez taille et force à gauche",
            Tool::Opening(_) => "Survoler un mur puis cliquer pour poser  •  vert = ça rentre, rouge = pas la place",
            Tool::Prop(_) => "Clic : poser  •  R : pivoter de 45°  •  Maj : placement libre",
            Tool::Paint => "Clic sur un élément : appliquer la couleur",
            Tool::Eyedropper => "Clic sur un élément : récupérer sa couleur (puis passe en peinture)",
            Tool::Erase => "Clic sur un élément : le supprimer",
            Tool::Visitor => "Cliquez où poser le bonhomme (sol, pièce, étage…) pour visiter  •  Tab : annuler",
        }
    }
}

#[derive(Resource)]
pub struct Editor {
    pub house: House,
    undo: VecDeque<House>,
    redo: Vec<House>,
    pub dirty: bool,
    /// Éléments animés (portes) dont seules les pièces sont à refaire, sans toute la maison.
    pub moved: HashSet<ElemRef>,
    /// Pièces actuellement affichées (pour la sélection par rayon).
    pub parts: Vec<Part>,
    pub tool: Tool,
    pub level: i32,
    pub hide_upper: bool,
    pub grid: f32,
    pub show_grid: bool,

    pub wall_height: f32,
    pub wall_thick: f32,
    pub wall_color: Rgb,
    pub wall_texture: usize,
    pub floor_color: Rgb,
    pub floor_texture: usize,
    pub roof_kind: RoofKind,
    pub roof_height: f32,
    pub roof_overhang: f32,
    pub roof_color: Rgb,
    pub roof_texture: usize,
    /// (largeur, hauteur, allège) et couleur par type d'ouverture.
    pub op_dims: [(f32, f32, f32); OpeningKind::ALL.len()],
    pub op_colors: [Rgb; OpeningKind::ALL.len()],
    pub prop_rot: f32,
    pub prop_height: f32,
    pub pool_depth: f32,
    pub pool_color: Rgb,
    pub pool_texture: usize,
    pub road_width: f32,
    pub road_color: Rgb,
    pub road_texture: usize,
    pub brush: Brush,
    pub brush_radius: f32,
    /// Mètres par seconde de clic maintenu.
    pub brush_strength: f32,
    /// Altitude visée par « Aplanir » : celle sous le pinceau au début du geste.
    brush_target: f32,
    pub brush_seed: u32,
    sculpting: bool,
    pub gen_amp: f32,
    pub gen_scale: f32,
    pub gen_mountains: bool,
    pub prop_color: Rgb,
    pub prop_texture: usize,
    pub paint_color: Rgb,
    pub paint_texture: usize,

    pub selected: Option<ElemRef>,
    pub hovered: Option<ElemRef>,
    wall_start: Option<Vec2>,
    drag_start: Option<Vec2>,
    /// Déplacement en cours : (point d'appui, maison avant, décalage actuel).
    move_drag: Option<(Vec2, House, Vec2)>,
    pub cursor: Option<Vec2>,
    /// Cote affichée près du pointeur (longueur, surface…).
    pub measure: Option<String>,
    pub message: String,
    pub file: String,
    pub hour: f32,
    pub walk: bool,
    /// Où le bonhomme a été posé (pieds), consommé par la caméra à l'entrée en visite.
    pub walk_start: Option<Vec3>,
    /// Portes et placards : (ouverture actuelle 0..1, ouvert demandé). État de visite, non enregistré.
    pub doors: HashMap<ElemRef, (f32, bool)>,
    /// Voiture en cours de conduite (index dans `house.props`), masquée du rendu normal.
    pub driving: Option<usize>,
    pub show_help: bool,
    /// Une modification de propriété est en cours (un seul point d'annulation par geste).
    pub prop_edit_active: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            house: House::default(),
            undo: VecDeque::new(),
            redo: vec![],
            dirty: true,
            moved: HashSet::new(),
            parts: vec![],
            tool: Tool::Select,
            level: 0,
            hide_upper: false,
            grid: 0.5,
            show_grid: true,
            wall_height: LEVEL_H,
            wall_thick: 0.2,
            wall_color: WHITE_TEX,
            wall_texture: find("Crépi blanc"),
            floor_color: WHITE_TEX,
            floor_texture: find("Parquet chêne clair"),
            roof_kind: RoofKind::Gable,
            roof_height: 2.5,
            roof_overhang: 0.5,
            roof_color: WHITE_TEX,
            roof_texture: find("Tuiles terre cuite"),
            op_dims: OpeningKind::ALL.map(super::model::OpeningKind::default_dims),
            op_colors: OpeningKind::ALL.map(super::model::OpeningKind::default_color),
            prop_rot: 0.0,
            prop_height: LEVEL_H,
            pool_depth: 1.5,
            pool_color: WHITE_TEX,
            pool_texture: find("Mosaïque bleue"),
            road_width: 4.0,
            road_color: [0.55, 0.55, 0.57],
            road_texture: find("Béton brut"),
            brush: Brush::Raise,
            brush_radius: 6.0,
            brush_strength: 2.0,
            brush_target: 0.0,
            brush_seed: 1,
            sculpting: false,
            gen_amp: 8.0,
            gen_scale: 40.0,
            gen_mountains: false,
            prop_color: PropKind::Table.default_color(),
            prop_texture: 0,
            paint_color: [0.8, 0.3, 0.3],
            paint_texture: 0,
            selected: None,
            hovered: None,
            wall_start: None,
            drag_start: None,
            move_drag: None,
            cursor: None,
            measure: None,
            message: String::new(),
            file: "maison.json".into(),
            hour: 14.0,
            walk: false,
            walk_start: None,
            doors: HashMap::new(),
            driving: None,
            show_help: true,
            prop_edit_active: false,
        }
    }
}

const MAX_UNDO: usize = 300;

impl Editor {
    /// À appeler juste avant une modification de la maison.
    pub fn checkpoint(&mut self) {
        let snapshot = self.house.clone();
        self.push_undo(snapshot);
    }

    pub fn push_undo(&mut self, h: House) {
        self.undo.push_back(h);
        if self.undo.len() > MAX_UNDO {
            self.undo.pop_front();
        }
        self.redo.clear();
        self.dirty = true;
        // les indices des éléments peuvent changer : on referme tout
        self.doors.clear();
        // un autre point d'annulation clôt le geste du panneau, qui rejoue depuis le dernier
        self.prop_edit_active = false;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        // pendant un déplacement, annuler l'abandonne sans toucher à l'historique
        if self.move_drag.is_some() {
            self.cancel();
            self.message = "Déplacement annulé".into();
            return;
        }
        if let Some(h) = self.undo.pop_back() {
            self.redo.push(std::mem::replace(&mut self.house, h));
            self.after_restore("Annulé");
        }
    }

    pub fn redo(&mut self) {
        self.cancel();
        if let Some(h) = self.redo.pop() {
            self.undo.push_back(std::mem::replace(&mut self.house, h));
            self.after_restore("Rétabli");
        }
    }

    fn after_restore(&mut self, msg: &str) {
        self.selected = None;
        self.prop_edit_active = false;
        self.doors.clear();
        self.cancel();
        self.dirty = true;
        self.message = msg.into();
    }

    /// Remplace toute la maison (nouveau, exemple, ouverture) de façon annulable.
    pub fn replace_house(&mut self, h: House, msg: String) {
        self.checkpoint();
        self.house = h;
        self.after_restore(&msg);
    }

    /// Abandonne l'action en cours (mur en cours de tracé, rectangle, déplacement).
    pub fn cancel(&mut self) {
        self.wall_start = None;
        self.drag_start = None;
        self.abandon_move();
    }

    /// Remet la maison d'avant le glisser-déplacer en cours, s'il y en a un.
    fn abandon_move(&mut self) {
        if let Some((_, orig, _)) = self.move_drag.take() {
            self.house = orig;
            self.dirty = true;
        }
    }

    pub fn set_tool(&mut self, t: Tool) {
        self.cancel();
        if let Tool::Prop(k) = t
            && self.tool != t
        {
            self.prop_color = k.default_color();
            self.prop_texture = k.default_texture();
            self.prop_height = k.default_height();
        }
        self.tool = t;
    }

    pub fn toggle_door(&mut self, e: ElemRef) {
        let d = self.doors.entry(e).or_insert((0.0, false));
        d.1 = !d.1;
    }

    /// Fait pivoter progressivement les portes vers leur état demandé.
    pub fn animate_doors(&mut self, dt: f32) {
        for (e, (cur, open)) in &mut self.doors {
            // cur reste dans [0, 1] : il bouge tant qu il n a pas atteint la borne visée
            if (*open && *cur < 1.0) || (!*open && *cur > 0.0) {
                *cur = if *open { (*cur + dt * 2.5).min(1.0) } else { (*cur - dt * 2.5).max(0.0) };
                self.moved.insert(*e);
            }
        }
    }

    pub fn set_level(&mut self, l: i32) {
        self.cancel();
        self.level = l.clamp(-1, 9);
        self.dirty = true;
    }

    /// Applique des commandes en un seul point d'annulation : toutes ou aucune.
    /// En cas d'erreur, la maison est intacte et le message dit pourquoi.
    /// Un glisser-déplacer en cours est d'abord abandonné : son aperçu n'est pas dans l'historique.
    pub fn apply(&mut self, cmds: impl IntoIterator<Item = Command>) -> Result<Option<ElemRef>, EditError> {
        self.abandon_move();
        let mut h = self.house.clone();
        let mut last = None;
        for c in cmds {
            match h.apply(c) {
                Ok(e) => last = e.or(last),
                Err(e) => {
                    self.message = e.to_string();
                    return Err(e);
                }
            }
        }
        let old = std::mem::replace(&mut self.house, h);
        self.push_undo(old);
        Ok(last)
    }

    pub fn delete_selected(&mut self) {
        if let Some(target) = self.selected.take() {
            // refus déjà affiché dans le message par Editor::apply
            let _ = self.apply([Command::Remove { target }]);
        }
    }

    pub fn duplicate_selected(&mut self) {
        if let Some(target) = self.selected
            && let Ok(new) = self.apply([Command::Duplicate { target, by: Vec2::splat(self.grid.max(0.5)) }])
        {
            self.selected = new;
        }
    }

    pub fn rotate_selected(&mut self) {
        if let Some(target @ ElemRef::Prop(_)) = self.selected {
            // refus déjà affiché dans le message par Editor::apply
            let _ = self.apply([Command::Rotate { target, deg: 45.0 }]);
        }
    }

    pub fn nudge_selected(&mut self, by: Vec2) {
        if let Some(target) = self.selected {
            // refus déjà affiché dans le message par Editor::apply
            let _ = self.apply([Command::Move { target, by }]);
        }
    }

    /// Renvoie `false` si l'écriture a échoué (le message dit pourquoi).
    pub fn save(&mut self) -> bool {
        let res = self.house.save(&self.file);
        let ok = res.is_ok();
        self.message = match res {
            Ok(()) => format!("Enregistré dans {}", self.file),
            Err(e) => format!("Erreur d'enregistrement : {e}"),
        };
        ok
    }

    pub fn load(&mut self) {
        match House::load(&self.file) {
            Ok(h) => self.replace_house(h, format!("Ouvert : {}", self.file)),
            Err(e) => self.message = format!("Impossible d'ouvrir {} : {e}", self.file),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    /// Un éditeur dont la maison contient une table en (0, 0), sans historique.
    fn with_table() -> Editor {
        let mut ed = Editor::default();
        ed.house.props.push(ed.new_prop(PropKind::Table, Vec2::ZERO));
        ed
    }

    #[test]
    fn undo_during_drag_keeps_history() {
        let mut ed = Editor::default();
        ed.checkpoint();
        ed.house.props.push(ed.new_prop(PropKind::Table, Vec2::ZERO));
        let placed = ed.house.clone();
        // glisser en cours : Ctrl+Z abandonne le déplacement sans consommer l'historique
        ed.move_drag = Some((Vec2::ZERO, placed.clone(), Vec2::X));
        ed.house.translate(ElemRef::Prop(0), Vec2::X);
        ed.undo();
        assert_eq!(ed.house, placed);
        assert!(ed.can_undo());
        ed.undo();
        assert_eq!(ed.house.props, []);
    }

    #[test]
    fn undo_redo_and_history_limit() {
        let mut ed = Editor::default();
        assert!(!ed.can_undo() && !ed.can_redo());
        ed.apply([Command::PlaceProp { kind: PropKind::Lamp, pos: Vec2::ZERO, level: 0, rot: 0.0, scale: None }])
            .unwrap();
        ed.undo();
        assert!(ed.house.props.is_empty() && ed.can_redo());
        assert_eq!(ed.message, "Annulé");
        ed.redo();
        assert_eq!((ed.house.props.len(), ed.message.as_str()), (1, "Rétabli"));
        for _ in 0..MAX_UNDO + 5 {
            ed.checkpoint();
        }
        assert_eq!(ed.undo.len(), MAX_UNDO);
        assert!(!ed.can_redo(), "une modification efface le rétablissement");
    }

    #[test]
    fn editor_apply_is_all_or_nothing() {
        let mut ed = Editor::default();
        let lamp = Command::PlaceProp { kind: PropKind::Lamp, pos: Vec2::ZERO, level: 0, rot: 0.0, scale: None };
        let err = ed.apply([lamp.clone(), Command::Remove { target: ElemRef::Wall(3) }]).unwrap_err();
        assert!(ed.house.props.is_empty() && !ed.can_undo());
        assert!(matches!(err, EditError::NotFound(ElemRef::Wall(3), _)), "{err}");
        assert_eq!(ed.message, err.to_string());
        assert_eq!(
            ed.apply([lamp, Command::FlattenTerrain]),
            Ok(Some(ElemRef::Prop(0))),
            "renvoie le dernier élément créé"
        );
    }

    #[test]
    fn selection_shortcuts() {
        let mut ed = with_table();
        ed.selected = Some(ElemRef::Prop(0));
        ed.rotate_selected();
        ed.nudge_selected(v(0.0, 1.0));
        assert_eq!((ed.house.props[0].rot, ed.house.props[0].pos), (45.0, v(0.0, 1.0)));
        ed.duplicate_selected();
        assert_eq!(ed.selected, Some(ElemRef::Prop(1)));
        assert_eq!(ed.house.props[1].pos, v(0.5, 1.5));
        ed.delete_selected();
        assert_eq!((ed.house.props.len(), ed.selected), (1, None));
        // sans sélection : rien
        ed.delete_selected();
        ed.rotate_selected();
        assert_eq!(ed.house.props.len(), 1);
    }

    #[test]
    fn tool_level_and_doors() {
        let mut ed = Editor::default();
        ed.set_tool(Tool::Prop(PropKind::Pine));
        assert_eq!(
            (ed.prop_color, ed.prop_texture),
            (PropKind::Pine.default_color(), PropKind::Pine.default_texture())
        );
        ed.prop_color = [0.0; 3];
        ed.set_tool(Tool::Prop(PropKind::Pine));
        assert_eq!(ed.prop_color, [0.0; 3], "re-choisir le même outil garde la couleur");
        ed.set_tool(Tool::Prop(PropKind::StoneSteps));
        assert!((ed.prop_height - 0.5).abs() < 1e-6, "un perron monte à une porte, pas d'un étage");
        ed.set_level(42);
        assert_eq!(ed.level, 9);
        ed.set_level(-5);
        assert_eq!(ed.level, -1);

        let door = ElemRef::Opening(0, 0);
        ed.toggle_door(door);
        ed.dirty = false;
        ed.animate_doors(0.2);
        assert_eq!(ed.doors[&door], (0.5, true));
        assert!(!ed.dirty && ed.moved.contains(&door), "seule la porte est à refaire");
        ed.animate_doors(1.0);
        assert_eq!(ed.doors[&door].0, 1.0);
        ed.toggle_door(door);
        ed.animate_doors(1.0);
        assert_eq!(ed.doors[&door].0, 0.0);
        let tools =
            [Tool::Select, Tool::Wall, Tool::Room, Tool::Floor, Tool::Roof, Tool::Pool, Tool::Road, Tool::Terrain]
                .into_iter()
                .chain([
                    Tool::Opening(OpeningKind::Door),
                    Tool::Prop(PropKind::Bed),
                    Tool::Paint,
                    Tool::Eyedropper,
                    Tool::Erase,
                    Tool::Visitor,
                ]);
        assert!(tools.map(Tool::hint).all(|h| !h.is_empty()));
    }

    #[test]
    fn save_then_load_roundtrips() {
        let dir = std::env::temp_dir().join("maison3d_editor_save");
        let _ = std::fs::remove_dir_all(&dir);
        let mut ed = with_table();
        ed.file = dir.join("sous/dossier/maison.json").to_str().unwrap().into();
        assert!(ed.save(), "{}", ed.message);
        let saved = ed.house.clone();
        ed.house = House::default();
        ed.load();
        assert_eq!(ed.house, saved);
        assert!(ed.message.starts_with("Ouvert"));
        ed.undo();
        assert_eq!(ed.house, House::default(), "ouvrir un fichier s'annule");

        ed.file = dir.join("absent.json").to_str().unwrap().into();
        ed.load();
        assert!(ed.message.starts_with("Impossible d'ouvrir"));
        ed.file = dir.to_str().unwrap().into();
        assert!(!ed.save(), "un dossier n'est pas un fichier");
        let _ = std::fs::remove_dir_all(dir);
    }
}
