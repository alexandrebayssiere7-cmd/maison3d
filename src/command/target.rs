//! Cible de `remove`, `move`, `paint` et `set` : un élément par son indice, ou tous ceux qui
//! portent une étiquette.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::EditError;
use crate::model::{ElemRef, House};

/// Élément visé par son indice (`{"Wall": 0}`, `{"Opening": [mur, n]}`, comme dans `describe`),
/// ou groupe d'éléments par leur étiquette (`{"tag": "haie-nord"}`) : plus besoin de compter
/// les indices, qu'une suppression décale.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Target {
    /// Un seul élément, par son indice dans sa liste.
    Elem(ElemRef),
    /// Tous les murs, sols, toits, objets, piscines et routes dont le champ `tag` vaut `tag`.
    Tag {
        /// Étiquette, comparée à l'identique (casse et accents compris) ; refusée si aucun
        /// élément ne la porte.
        tag: String,
    },
}

impl From<ElemRef> for Target {
    fn from(e: ElemRef) -> Self {
        Target::Elem(e)
    }
}

impl House {
    /// Les éléments existants que désigne `target`, dans l'ordre des listes de `describe`.
    pub(super) fn resolve(&self, target: &Target) -> Result<Vec<ElemRef>, EditError> {
        match target {
            Target::Elem(e) => {
                self.found(*e)?;
                Ok(vec![*e])
            }
            Target::Tag { tag } => {
                let refs = self.tagged(tag);
                if refs.is_empty() { Err(EditError::NoSuchTag(tag.clone())) } else { Ok(refs) }
            }
        }
    }

    /// Applique `f` à chaque élément visé ; renvoie ce que renvoie le dernier. Un échec arrête
    /// tout : `apply` restaure alors la maison.
    pub(super) fn each(
        &mut self,
        target: &Target,
        mut f: impl FnMut(&mut Self, ElemRef) -> Result<Option<ElemRef>, EditError>,
    ) -> Result<Option<ElemRef>, EditError> {
        let mut last = None;
        for e in self.resolve(target)? {
            last = f(self, e)?;
        }
        Ok(last)
    }

    /// Supprime chaque élément visé, du plus grand indice au plus petit dans chaque liste pour
    /// que les suppressions ne décalent pas les éléments restant à supprimer.
    pub(super) fn remove_all(&mut self, target: &Target) -> Result<Option<ElemRef>, EditError> {
        for e in self.resolve(target)?.into_iter().rev() {
            self.remove_settled(e);
        }
        Ok(None)
    }

    fn tagged(&self, tag: &str) -> Vec<ElemRef> {
        fn pick<'a>(
            tags: impl Iterator<Item = &'a Option<String>> + 'a,
            tag: &'a str,
            elem: fn(usize) -> ElemRef,
        ) -> impl Iterator<Item = ElemRef> + 'a {
            tags.enumerate().filter(move |(_, t)| t.as_deref() == Some(tag)).map(move |(i, _)| elem(i))
        }
        pick(self.walls.iter().map(|w| &w.tag), tag, ElemRef::Wall)
            .chain(pick(self.slabs.iter().map(|s| &s.tag), tag, ElemRef::Slab))
            .chain(pick(self.roofs.iter().map(|r| &r.tag), tag, ElemRef::Roof))
            .chain(pick(self.props.iter().map(|p| &p.tag), tag, ElemRef::Prop))
            .chain(pick(self.pools.iter().map(|p| &p.tag), tag, ElemRef::Pool))
            .chain(pick(self.roads.iter().map(|r| &r.tag), tag, ElemRef::Road))
            .collect()
    }
}
