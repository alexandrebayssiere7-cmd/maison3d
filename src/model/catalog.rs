//! Catalogue des objets posables, rangés par catégorie : nom affiché, catégorie, intérieur ou
//! extérieur et dimension réglable de chaque sorte. Leur géométrie est dans `shapes`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Rayon de la bibliothèque d'objets, dans l'ordre du menu de l'éditeur.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Category {
    Living,
    Bedroom,
    Kitchen,
    Bathroom,
    Office,
    Garage,
    Store,
    Street,
    Signage,
    Trees,
    Plants,
    Garden,
    Leisure,
    Sport,
    Markings,
    Structure,
    Vehicles,
}

impl Category {
    pub const ALL: [Category; 17] = [
        Self::Living,
        Self::Bedroom,
        Self::Kitchen,
        Self::Bathroom,
        Self::Office,
        Self::Garage,
        Self::Store,
        Self::Street,
        Self::Signage,
        Self::Trees,
        Self::Plants,
        Self::Garden,
        Self::Leisure,
        Self::Sport,
        Self::Markings,
        Self::Structure,
        Self::Vehicles,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Living => "Salon",
            Self::Bedroom => "Chambre",
            Self::Kitchen => "Cuisine",
            Self::Bathroom => "Salle de bain",
            Self::Office => "Bureau",
            Self::Garage => "Garage",
            Self::Store => "Magasin",
            Self::Street => "Mobilier urbain",
            Self::Signage => "Signalisation",
            Self::Trees => "Arbres",
            Self::Plants => "Plantes et fleurs",
            Self::Garden => "Jardin",
            Self::Leisure => "Terrasse et jeux",
            Self::Sport => "Sport",
            Self::Markings => "Tracés de terrain",
            Self::Structure => "Escaliers et structure",
            Self::Vehicles => "Véhicules",
        }
    }
    /// Sortes d'objets de ce rayon, dans l'ordre du catalogue.
    pub fn kinds(self) -> impl Iterator<Item = PropKind> {
        PropKind::ALL.into_iter().filter(move |k| k.category() == self)
    }
}

/// Valeurs de la colonne « dehors » du catalogue.
const DEHORS: bool = true;
const DEDANS: bool = false;

macro_rules! dim {
    () => {
        None
    };
    ($label:literal) => {
        Some($label)
    };
}

/// Une ligne par sorte : `Variante = "Nom affiché", Catégorie, DEHORS | DEDANS[, "dimension réglable"];`.
macro_rules! catalog {
    ($( $(#[doc = $doc:literal])+ $kind:ident = $name:literal, $cat:ident, $place:ident $(, $dim:literal)?; )+) => {
        /// Type d'objet posé. À `rot` 0, l'avant de l'objet regarde +z (le sud). Catégorie,
        /// encombrement, parties colorables et extérieur ou non : `maison3d catalog`.
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
        pub enum PropKind {
            $( $(#[doc = $doc])+ $kind, )+
        }

        impl PropKind {
            pub const ALL: [PropKind; [$(stringify!($kind)),+].len()] = [$(Self::$kind),+];
            pub fn name(self) -> &'static str {
                match self { $(Self::$kind => $name,)+ }
            }
            pub fn category(self) -> Category {
                match self { $(Self::$kind => Category::$cat,)+ }
            }
            /// Posé dehors : sur le relief du terrain plutôt qu'à plat.
            pub fn outdoor(self) -> bool {
                match self { $(Self::$kind => $place,)+ }
            }
            /// Ce que règle le champ `height` (« hauteur », « longueur », « rayon »), s'il sert.
            pub fn dim(self) -> Option<&'static str> {
                match self { $(Self::$kind => dim!($($dim)?),)+ }
            }
        }
    };
}

catalog! {
    // ------------------------------------------------------------ salon
    /// Table de repas rectangulaire.
    Table = "Table", Living, DEDANS;
    /// Chaise ; l'avant est le côté où l'on s'assoit (dossier à l'arrière). À tourner vers la table.
    Chair = "Chaise", Living, DEDANS;
    /// Canapé trois places ; l'avant est l'assise, à tourner vers la pièce ou la télé, dossier contre un mur.
    Sofa = "Canapé", Living, DEDANS;
    /// Lampe sur pied d'intérieur (1,85 m) qui éclaire réellement la pièce. Pour l'éclairage public : `StreetLamp`.
    Lamp = "Lampe sur pied", Living, DEDANS;

    // ------------------------------------------------------------ chambre
    /// Lit double ; la tête est à l'arrière (à coller au mur), les pieds vers l'avant.
    Bed = "Lit", Bedroom, DEDANS;
    /// Armoire, s'ouvre pendant la visite ; dos contre un mur, portes vers l'avant.
    Wardrobe = "Armoire", Bedroom, DEDANS;

    // ------------------------------------------------------------ cuisine
    /// Bloc cuisine (meubles bas, évier, plaques), placards ouvrables ; dos contre un mur.
    Kitchen = "Cuisine", Kitchen, DEDANS;
    /// Réfrigérateur, s'ouvre pendant la visite ; dos contre un mur.
    Fridge = "Frigo", Kitchen, DEDANS;

    // ------------------------------------------------------------ salle de bain
    /// Baignoire.
    Bathtub = "Baignoire", Bathroom, DEDANS;
    /// WC ; réservoir à l'arrière, contre un mur.
    Toilet = "WC", Bathroom, DEDANS;

    // ------------------------------------------------------------ bureau

    // ------------------------------------------------------------ garage

    // ------------------------------------------------------------ magasin
    /// Étagère ouverte de magasin ou de bibliothèque (1 × 0,4 × 1,8 m), dos contre un mur ou dos à
    /// dos avec une autre pour former un rayon.
    Shelf = "Étagère", Store, DEDANS;

    // ------------------------------------------------------------ mobilier urbain
    /// Lampadaire de rue de 4,5 m qui éclaire réellement ; la lanterne avance vers l'avant de l'objet
    /// (à tourner vers la chaussée). Un tous les 10 à 15 m sur un trottoir.
    StreetLamp = "Lampadaire de rue", Street, DEHORS;

    // ------------------------------------------------------------ signalisation

    // ------------------------------------------------------------ arbres
    /// Arbre feuillu.
    Tree = "Arbre", Trees, DEHORS;
    /// Sapin.
    Pine = "Sapin", Trees, DEHORS;
    /// Bouleau.
    Birch = "Bouleau", Trees, DEHORS;
    /// Palmier.
    Palm = "Palmier", Trees, DEHORS;
    /// Cyprès, haut et étroit (alignements, bords de terrain).
    Cypress = "Cyprès", Trees, DEHORS;
    /// Pommier.
    AppleTree = "Pommier", Trees, DEHORS;

    // ------------------------------------------------------------ plantes et fleurs
    /// Plante d'intérieur en pot.
    Plant = "Plante", Plants, DEDANS;
    /// Buisson rond.
    Bush = "Buisson", Plants, DEHORS;
    /// Massif de fleurs.
    Flowers = "Fleurs", Plants, DEHORS;
    /// Rosier.
    Rosebush = "Rosier", Plants, DEHORS;
    /// Tournesols.
    Sunflower = "Tournesols", Plants, DEHORS;

    // ------------------------------------------------------------ jardin
    /// Module de haie de 2 m de long (axe gauche-droite de l'objet). Aligner des modules tous les 2 m pour une haie continue ; `rot` 90 pour une haie nord-sud.
    Hedge = "Haie", Garden, DEHORS;
    /// Module de clôture de 2 m de long, à aligner comme la haie.
    Fence = "Clôture", Garden, DEHORS;

    // ------------------------------------------------------------ terrasse et jeux

    // ------------------------------------------------------------ sport

    // ------------------------------------------------------------ tracés de terrain

    // ------------------------------------------------------------ escaliers et structure
    /// Escalier droit qui monte vers l'avant sur `height` mètres (3 = un étage). Le palier d'arrivée doit tomber dans un sol de l'étage au-dessus.
    Stairs = "Escalier", Structure, DEDANS, "hauteur";
    /// Escalier en colimaçon qui monte de `height` mètres, encombrement réduit.
    SpiralStairs = "Colimaçon", Structure, DEDANS, "hauteur";
    /// Escalier quart tournant : une volée vers l'avant jusqu'à un palier, puis une volée vers le +x local de l'objet (l'est à rot 0), sur `height` mètres.
    LStairs = "Quart tournant", Structure, DEDANS, "hauteur";
    /// Pilier carré de 30 cm, haut de `height` mètres (portique, auvent, véranda).
    Pillar = "Pilier", Structure, DEDANS, "hauteur";

    // ------------------------------------------------------------ véhicules
    /// Berline de 4,2 m que l'on peut conduire pendant la visite ; à poser sur une route ou une allée.
    Car = "Voiture", Vehicles, DEHORS;
}

impl PropKind {
    /// Escalier : son palier d'arrivée doit tomber dans un sol de l'étage visé (`maison3d check`).
    pub fn climbs(self) -> bool {
        matches!(self, Self::Stairs | Self::SpiralStairs | Self::LStairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_is_listed_once_and_used() {
        for (i, c) in Category::ALL.into_iter().enumerate() {
            assert_eq!(c as usize, i, "{c:?} : ALL suit l'ordre de déclaration");
            assert_ne!(c.name(), "");
        }
        let used = |c: Category| c.kinds().next().is_some();
        assert!(used(Category::Living) && used(Category::Vehicles));
        assert_eq!(Category::ALL.iter().map(|c| c.kinds().count()).sum::<usize>(), PropKind::ALL.len());
    }
    #[test]
    fn catalog_columns() {
        assert_eq!((PropKind::Table.name(), PropKind::Table.category()), ("Table", Category::Living));
        assert!(PropKind::Pine.outdoor() && !PropKind::Sofa.outdoor());
        assert_eq!((PropKind::Stairs.dim(), PropKind::Bed.dim()), (Some("hauteur"), None));
        assert!(PropKind::LStairs.climbs() && !PropKind::Pillar.climbs());
    }
}
