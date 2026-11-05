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
    /// Îlot central (1,9 × 1 × 0,92 m) : placards ouvrables à l'avant, plan de travail qui déborde
    /// de 40 cm à l'arrière pour s'y asseoir (tabourets `BarStool` non compris, à poser côté arrière).
    /// Au milieu de la cuisine, à 1 m au moins des meubles muraux.
    KitchenIsland = "Îlot central", Kitchen, DEDANS;
    /// Meuble bas de cuisine, module de 60 × 62 × 90 cm plan de travail compris, porte qui s'ouvre ;
    /// dos contre un mur, à aligner tous les 60 cm avec `Range` et `Dishwasher` (même gabarit).
    BaseCabinet = "Meuble bas", Kitchen, DEDANS;
    /// Meuble haut de cuisine (60 × 38 cm), fixé au mur entre 1,4 et 2,1 m, porte qui s'ouvre ; dos
    /// contre un mur, au-dessus des meubles bas, à aligner tous les 60 cm.
    WallCabinet = "Meuble haut", Kitchen, DEDANS;
    /// Cuisinière (60 × 60 × 90 cm) : quatre foyers, boutons à l'avant, four dont la porte s'abat
    /// vers l'avant ; dos contre un mur, alignée avec les meubles bas, `RangeHood` au-dessus.
    Range = "Cuisinière", Kitchen, DEDANS;
    /// Colonne four (60 × 60 × 220 cm) : placard bas, four encastré à abattant, micro-ondes et
    /// placard haut ; dos contre un mur, en bout de rangée de meubles bas.
    OvenTower = "Colonne four", Kitchen, DEDANS;
    /// Lave-vaisselle (60 × 60 × 85 cm), porte qui s'abat vers l'avant sur deux paniers garnis ;
    /// dos contre un mur, aligné avec les meubles bas.
    Dishwasher = "Lave-vaisselle", Kitchen, DEDANS;
    /// Hotte murale (60 × 50 cm), bas à 1,6 m et cheminée jusqu'à 2,4 m ; dos contre un mur, à la
    /// même position que la cuisinière `Range` ou les plaques, au-dessus d'elles.
    RangeHood = "Hotte", Kitchen, DEDANS;
    /// Tabouret de bar rond (assise à 72 cm) sur pied central ; autour d'un `KitchenIsland` ou d'un
    /// `HighTable`, tous les 50 cm environ.
    BarStool = "Tabouret de bar", Kitchen, DEDANS;
    /// Poubelle de cuisine à pédale (30 cm de diamètre, 66 cm de haut), pédale à l'avant ; contre un
    /// mur ou au bout d'un meuble bas.
    KitchenBin = "Poubelle de cuisine", Kitchen, DEDANS;
    /// Mange-debout rond (70 cm de diamètre, 1,09 m de haut) sur pied central ; avec deux à quatre
    /// tabourets `BarStool` autour.
    HighTable = "Mange-debout", Kitchen, DEDANS;

    // ------------------------------------------------------------ salle de bain
    /// Baignoire.
    Bathtub = "Baignoire", Bathroom, DEDANS;
    /// WC ; réservoir à l'arrière, contre un mur.
    Toilet = "WC", Bathroom, DEDANS;
    /// Douche à l'italienne (1,4 × 0,9 m) : receveur extra-plat, paroi de verre fixe sur la gauche
    /// de l'avant (entrée de 60 cm à droite), colonne et pommeau contre le mur du fond ; dos contre un
    /// mur, idéalement dans un angle côté gauche.
    WalkInShower = "Douche à l'italienne", Bathroom, DEDANS;
    /// Cabine de douche vitrée (90 × 90 × 210 cm), paroi pleine au fond, porte vitrée qui s'ouvre à
    /// l'avant ; dos contre un mur, souvent dans un angle.
    ShowerCabin = "Cabine de douche", Bathroom, DEDANS;
    /// Lavabo sur colonne (56 × 45 cm, plan à 86 cm), robinet à l'arrière ; dos contre un mur.
    Washbasin = "Lavabo sur colonne", Bathroom, DEDANS;
    /// Meuble vasque (82 × 48 cm) : deux portes ouvrables, vasque à poser, mitigeur et miroir au mur
    /// au-dessus (1,15 à 1,85 m) ; dos contre un mur.
    VanityUnit = "Meuble vasque", Bathroom, DEDANS;
    /// Miroir mural encadré (66 × 86 cm, de 1,07 à 1,93 m de haut, 3 cm d'épaisseur) ; dos contre un
    /// mur, au-dessus d'un lavabo `Washbasin`.
    Mirror = "Miroir mural", Bathroom, DEDANS;
    /// Sèche-serviettes à barreaux (54 cm de large, de 25 cm à 1,5 m), serviette posée dessus ; dos
    /// contre un mur, près de la douche ou de la baignoire.
    TowelRadiator = "Sèche-serviettes", Bathroom, DEDANS;
    /// Lave-linge à hublot (60 × 60 × 85 cm), le hublot s'ouvre à l'avant ; dos contre un mur, dans la
    /// salle de bain, la buanderie ou le garage.
    WashingMachine = "Lave-linge", Bathroom, DEDANS;
    /// Panier à linge en osier (42 cm de diamètre, 60 cm de haut) ; dans un coin ou contre un mur.
    Hamper = "Panier à linge", Bathroom, DEDANS;

    // ------------------------------------------------------------ bureau

    // ------------------------------------------------------------ garage
    /// Établi d'atelier (1,8 × 0,7 m, plan à 90 cm) avec étau, tablette basse et panneau à outils
    /// accroché au mur derrière (jusqu'à 1,8 m) ; dos contre un mur.
    Workbench = "Établi", Garage, DEDANS;
    /// Servante d'atelier à roulettes (70 × 47 × 103 cm), cinq tiroirs qui sortent vers l'avant ; près
    /// de l'établi.
    ToolChest = "Servante d'atelier", Garage, DEDANS;
    /// Rayonnage métallique (1,2 × 0,5 × 1,9 m), cinq tablettes garnies de cartons, bacs et pots ; dos
    /// contre un mur, à aligner tous les 1,2 m.
    MetalShelf = "Rayonnage métallique", Garage, DEDANS;
    /// Pile de quatre pneus couchés avec leurs jantes (62 cm de diamètre, 80 cm de haut) ; dans un
    /// coin ou le long d'un mur.
    TireStack = "Pile de pneus", Garage, DEDANS;
    /// Trois bidons de 20 l côte à côte (deux rouges, un vert ; 57 × 35 × 50 cm) ; au pied d'un mur.
    JerryCans = "Bidons", Garage, DEDANS;
    /// Chauffe-eau de 200 l sur trépied (56 cm de diamètre, 2,1 m avec ses tuyaux) ; contre un mur,
    /// dans le garage ou la buanderie.
    WaterHeater = "Chauffe-eau", Garage, DEDANS;
    /// Tondeuse à gazon poussée (0,6 × 1,05 m) : l'avant (carter) regarde +z, le guidon et le bac
    /// sont à l'arrière ; rangée dans le garage ou posée sur la pelouse.
    Lawnmower = "Tondeuse", Garage, DEDANS;
    /// Escabeau ouvert (0,47 × 0,76 m, plateau à 1,46 m), marches à l'avant ; contre un mur ou
    /// sous un plafond à repeindre.
    Stepladder = "Escabeau", Garage, DEDANS;
    /// Pile de cartons scotchés (environ 1 × 0,4 × 0,7 m) ; contre un mur ou sur un rayonnage vide.
    Boxes = "Cartons", Garage, DEDANS;
    /// Congélateur coffre (1,1 × 0,65 × 0,88 m), couvercle articulé à l'arrière qui se lève par
    /// l'avant ; dos contre un mur, 1 m de dégagement devant.
    ChestFreezer = "Congélateur coffre", Garage, DEDANS;
    /// Pont élévateur deux colonnes (3,8 × 2 m bras compris, 2,8 m de haut) ; une voiture `Car` se gare entre
    /// les colonnes en entrant par l'avant (+z), bras repliés de part et d'autre. Plafond de 2,8 m
    /// au moins.
    CarLift = "Pont élévateur", Garage, DEDANS;

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
    /// Chêne : grand feuillu étalé au tronc épais (couronne de 7 m de large, 7 m de haut). Isolé
    /// au milieu d'une pelouse ou d'un parc, à 5 m au moins de la maison.
    Oak = "Chêne", Trees, DEHORS;
    /// Saule pleureur (5,4 m de large, 5,7 m de haut) : rameaux qui retombent en rideau presque
    /// jusqu'au sol. Au bord d'un bassin, d'une mare ou d'une rivière.
    WeepingWillow = "Saule pleureur", Trees, DEHORS;
    /// Olivier (4 m de large, 4 m de haut) : tronc court et tortueux, feuillage gris-vert léger.
    /// Jardin méditerranéen, cour ou patio.
    OliveTree = "Olivier", Trees, DEHORS;
    /// Érable rouge (4,6 m de large, 7 m de haut), feuillage d'automne. Pour une touche de couleur
    /// au fond d'un jardin ou dans un parc.
    Maple = "Érable rouge", Trees, DEHORS;
    /// Cerisier du Japon en fleurs (5 m de large, 4,8 m de haut) : couronne rose étalée sur un
    /// tronc court. Jardin d'agrément ou japonais, allée de parc.
    CherryBlossom = "Cerisier du Japon", Trees, DEHORS;
    /// Platane d'alignement (5,6 m de large, 8,7 m de haut) à l'écorce claire ; sa couronne laisse
    /// 4 m libres au-dessus du sol. À aligner tous les 8 à 10 m le long d'un trottoir ou d'un mail.
    PlaneTree = "Platane", Trees, DEHORS;
    /// Arbre mort (4 m de large, 5 m de haut) : tronc et branches nues, sans feuillage. Friche,
    /// lisière de forêt, décor lugubre.
    DeadTree = "Arbre mort", Trees, DEHORS;

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
    /// Cactus cierge à deux bras (1 m de large, 2,1 m de haut), fleuri au sommet. Jardin sec, rocaille,
    /// décor de désert.
    Cactus = "Cactus", Plants, DEHORS;
    /// Touffe de bambous (2,5 m de large, 4,5 m de haut). En rangée pour un écran végétal contre une
    /// clôture, ou isolée dans un jardin japonais.
    Bamboo = "Bambous", Plants, DEHORS;
    /// Fougère en rosette (1,4 m de large, 0,5 m de haut). Sous-bois, pied d'un arbre, coin d'ombre.
    Fern = "Fougère", Plants, DEHORS;
    /// Touffe d'herbes hautes ornementales (1,2 m de large, 1,6 m de haut) aux épis clairs. Massif,
    /// bord d'allée, prairie ; à grouper par trois ou cinq.
    TallGrass = "Herbes hautes", Plants, DEHORS;
    /// Roseaux (massettes) aux épis bruns (0,8 m de large, 2 m de haut). Au bord d'une mare, d'un
    /// bassin ou d'une rivière.
    Reeds = "Roseaux", Plants, DEHORS;
    /// Touffe de lavande (1 m de large, 0,8 m de haut). En bordure d'allée, à aligner tous les
    /// 0,8 m pour une haie basse fleurie.
    Lavender = "Lavande", Plants, DEHORS;
    /// Carré de seize tulipes rouges et jaunes (1 × 0,7 m, 0,6 m de haut). Plate-bande, devant
    /// de massif.
    Tulips = "Tulipes", Plants, DEHORS;
    /// Hortensia : buisson rond (1,5 m de large, 1 m de haut) couvert de grosses fleurs bleues.
    /// Au pied d'un mur ou d'une façade à mi-ombre.
    Hydrangea = "Hortensia", Plants, DEHORS;
    /// Rocher (2,1 × 1,6 m, 1 m de haut), légèrement enfoncé dans le sol. Rocaille, bord de
    /// chemin, paysage de montagne.
    Rock = "Rocher", Plants, DEHORS;
    /// Souche d'arbre sciée (1,3 m de large avec les racines, 0,5 m de haut). Sous-bois, clairière ;
    /// peut servir de tabouret.
    Stump = "Souche", Plants, DEHORS;
    /// Petit groupe de champignons rouges à points blancs (0,4 m de large, 0,2 m de haut). Sous-bois,
    /// pied d'arbre, à côté d'une souche.
    Mushrooms = "Champignons", Plants, DEHORS;
    /// Buis taillé en boule sur une tige, en pot (Ø 0,5 m, 1,6 m de haut). Par paire de part et
    /// d'autre d'une porte d'entrée, sur une terrasse ou un perron.
    Topiary = "Buis en boule", Plants, DEHORS;
    /// Ficus : grande plante d'intérieur en pot (1 m de large, 2,1 m de haut). Dans un angle
    /// de pièce ou près d'une fenêtre.
    Ficus = "Ficus", Plants, DEDANS;
    /// Bonsaï dans sa coupe basse (0,5 × 0,35 m, 0,55 m de haut). Sur un meuble bas, une console
    /// ou au sol près d'une baie.
    Bonsai = "Bonsaï", Plants, DEDANS;

    // ------------------------------------------------------------ jardin
    /// Module de haie de 2 m de long (axe gauche-droite de l'objet). Aligner des modules tous les 2 m pour une haie continue ; `rot` 90 pour une haie nord-sud.
    Hedge = "Haie", Garden, DEHORS;
    /// Module de clôture de 2 m de long, à aligner comme la haie.
    Fence = "Clôture", Garden, DEHORS;
    /// Module de clôture blanche à lattes pointues, 2 m de long (axe gauche-droite) sur 1 m de haut. À aligner tous les 2 m (`place_row`, `step` 2) : les poteaux des bouts se confondent avec ceux des modules voisins.
    PicketFence = "Clôture blanche", Garden, DEHORS;
    /// Module de muret en pierre de 2 m de long, 0,62 m de haut et 0,42 m d'épaisseur, avec chaperon. À aligner tous les 2 m comme la haie.
    LowWall = "Muret de pierre", Garden, DEHORS;
    /// Portail à deux battants : 3 m de passage entre deux piliers de pierre (3,9 m hors tout, 1,8 m de haut). L'avant regarde la rue ; les battants s'ouvrent vers l'arrière (côté jardin) pendant la visite. À placer dans l'alignement d'une clôture ou d'un muret.
    Gate = "Portail", Garden, DEHORS;
    /// Abri de jardin en bois de 2,4 × 2 m, 2,6 m au faîte, toit à deux pans, porte à l'avant qui s'ouvre pendant la visite, fenêtre sur le côté droit. Laisser 1 m libre devant la porte.
    GardenShed = "Abri de jardin", Garden, DEHORS;
    /// Serre vitrée de 2,4 × 3 m, 2,5 m au faîte, sur un socle de briques, porte vitrée à l'avant, tablettes de semis le long des côtés.
    Greenhouse = "Serre", Garden, DEHORS;
    /// Pergola en bois de 3 × 3 m, 2,7 m de haut : quatre poteaux, deux poutres et des chevrons. À poser sur une terrasse, au-dessus d'une table de jardin ou d'un salon de jardin.
    Pergola = "Pergola", Garden, DEHORS;
    /// Puits en pierre de 1,3 m de diamètre et 2,4 m de haut : margelle, toit de tuiles, treuil avec seau ; la manivelle est à l'avant.
    Well = "Puits", Garden, DEHORS;
    /// Fontaine de 2,5 m de diamètre et 1,9 m de haut : bassin rond, vasque et jet d'eau. Au centre d'un jardin ou d'une place, à l'écart des murs.
    Fountain = "Fontaine", Garden, DEHORS;
    /// Mare ovale de 3,1 × 2,4 m (pierres comprises) : eau au ras du sol entourée de pierres, nénuphars, roseaux sur le côté gauche.
    Pond = "Mare", Garden, DEHORS;
    /// Jardinière en bois de 1 × 0,35 m garnie de fleurs (0,6 m de haut). Au pied d'un mur, d'une fenêtre ou en bord de terrasse.
    Planter = "Jardinière", Garden, DEHORS;
    /// Carré potager de 1,2 × 1,2 m à bordure de bois : salades à l'avant, trois pieds de tomates tuteurés à l'arrière (1,2 m). En plein soleil ; plusieurs côte à côte tous les 1,5 m pour un potager.
    VegetablePatch = "Carré potager", Garden, DEHORS;
    /// Chemin de cinq pierres plates (4 cm) espacées de 60 cm de l'arrière vers l'avant, 3 m de long : orienter l'avant dans le sens de la marche et en mettre plusieurs bout à bout tous les 3 m. Objet plat : les autres objets peuvent le chevaucher.
    SteppingStones = "Pas japonais", Garden, DEHORS;
    /// Lanterne japonaise en pierre (tōrō) de 1,2 m, qui éclaire réellement. Au bord d'une mare, d'un chemin ou près d'un torii.
    StoneLantern = "Lanterne japonaise", Garden, DEHORS;
    /// Torii, portique japonais rouge de 5,1 m de large et 3,7 m de haut, 2,8 m de passage entre ses piliers ; on passe dessous dans le sens avant-arrière. À l'entrée d'un jardin japonais ou d'une allée.
    Torii = "Torii", Garden, DEHORS;
    /// Tas de bois de chauffage de 1,4 × 0,5 m et 0,7 m de haut, bûches couchées (bouts vers l'avant) entre quatre piquets. Dos contre un mur ou un abri.
    Woodpile = "Tas de bois", Garden, DEHORS;
    /// Brouette de 1,6 m de long, roue à l'avant et poignées à l'arrière.
    Wheelbarrow = "Brouette", Garden, DEHORS;
    /// Nain de jardin de 47 cm, tourné vers l'avant. Dans une pelouse ou au bord d'un massif.
    GardenGnome = "Nain de jardin", Garden, DEHORS;
    /// Niche de chien de 0,8 × 1 m, 0,9 m au faîte, entrée à l'avant avec une gamelle devant. Dos contre une haie ou un mur.
    Doghouse = "Niche", Garden, DEHORS;
    /// Nichoir à oiseaux sur un poteau de 1,9 m, trou d'envol vers l'avant.
    Birdhouse = "Nichoir", Garden, DEHORS;

    // ------------------------------------------------------------ terrasse et jeux
    /// Table de jardin à lattes de 1,6 × 0,9 m (six couverts), 0,74 m de haut. Avec des `GardenChair` tournées vers elle, sur une terrasse ou sous une pergola.
    GardenTable = "Table de jardin", Leisure, DEHORS;
    /// Chaise de jardin à lattes ; l'avant est le côté où l'on s'assoit (dossier à l'arrière). À tourner vers la table de jardin.
    GardenChair = "Chaise de jardin", Leisure, DEHORS;
    /// Parasol de 2,8 m de diamètre et 2,5 m de haut sur pied lesté ; la toile penche légèrement vers l'avant (côté à ombrager). À côté d'une table, d'un transat ou d'un salon de jardin.
    Parasol = "Parasol", Leisure, DEHORS;
    /// Transat (bain de soleil) de 0,65 × 1,8 m, dossier relevé à l'arrière, pieds vers l'avant. Au bord d'une piscine, avant tourné vers l'eau ou le soleil ; plusieurs côte à côte tous les 0,9 m.
    Lounger = "Transat", Leisure, DEHORS;
    /// Salon de jardin de 2 × 1,7 m : canapé bas trois places à l'arrière, tourné vers l'avant, et table basse devant lui. Dos contre un mur de la maison ou une haie.
    LoungeSet = "Salon de jardin", Leisure, DEHORS;
    /// Barbecue boule à charbon de 0,6 m de diamètre et 1,1 m de haut, sur trois pieds. Sur la terrasse, à 3 m au moins des haies.
    Barbecue = "Barbecue", Leisure, DEHORS;
    /// Brasero : vasque d'acier de 0,85 m sur trois pieds, bûches et flammes qui éclairent réellement. Au centre d'un cercle de chaises ou de transats.
    Firepit = "Brasero", Leisure, DEHORS;
    /// Hamac tendu entre deux poteaux, 3,7 m de long (axe gauche-droite) sur 0,8 m. Laisser la place libre entre les poteaux.
    Hammock = "Hamac", Leisure, DEHORS;
    /// Spa carré de 2,1 m et 0,9 m de haut, rempli d'eau, quatre appuie-tête. Sur une terrasse, près de la maison.
    HotTub = "Spa", Leisure, DEHORS;
    /// Balançoire : portique en A de 2,9 × 1,7 m, poutre à 2,1 m, deux sièges suspendus. Laisser 2 m libres devant et derrière (axe avant-arrière) pour le balancement.
    Swing = "Balançoire", Leisure, DEHORS;
    /// Toboggan de 3,1 m de long : échelle à l'arrière, plateforme à 1,2 m, glissière qui descend vers l'avant. Laisser 1,5 m libre devant le bas de la glissière.
    Slide = "Toboggan", Leisure, DEHORS;
    /// Trampoline rond de 3 m de diamètre, toile à 0,87 m. À 2 m au moins des murs, arbres et clôtures.
    Trampoline = "Trampoline", Leisure, DEHORS;
    /// Bac à sable carré de 1,6 m à bordure de bois, sièges d'angle, seau et pelle.
    Sandbox = "Bac à sable", Leisure, DEHORS;
    /// Tape-cul (bascule) de 3 m de long (axe gauche-droite), un bout posé au sol, siège et poignée à chaque bout.
    Seesaw = "Tape-cul", Leisure, DEHORS;
    /// Cabane d'enfant en bois de 1,4 × 1,2 m, 1,85 m au faîte : porte à l'avant qui s'ouvre pendant la visite, fenêtre en façade. Laisser 1 m libre devant la porte.
    PlayHouse = "Cabane d'enfant", Leisure, DEHORS;

    // ------------------------------------------------------------ sport
    /// Cage de foot à 11 (7,32 × 2,44 m entre les poteaux, filet profond de 2 m). Les poteaux sont
    /// 1 m devant `pos`, l'ouverture vers l'avant : `pos` 1 m derrière la ligne de but, tournée vers
    /// le terrain (pour un `SoccerPitch` : x = -53,5 et `rot` 90, x = 53,5 et `rot` 270).
    SoccerGoal = "Cage de foot", Sport, DEHORS;
    /// But de handball ou de futsal (3 × 2 m, poteaux rayés, filet profond de 1 m). Poteaux 0,5 m
    /// devant `pos`, ouverture vers l'avant : `pos` 0,5 m derrière la ligne de but (x = ±20,5 pour
    /// un `HandballCourt` ou un `SoccerPitch5`), tourné vers le terrain.
    HandballGoal = "But de hand et futsal", Sport, DEHORS;
    /// Ballon de foot (22 cm).
    SoccerBall = "Ballon de foot", Sport, DEHORS;
    /// Ballon de basket (24 cm).
    Basketball = "Ballon de basket", Sport, DEHORS;
    /// Panier de basket sur pied : socle et pied à l'arrière, planche de 1,8 m dont la face est
    /// 0,375 m devant `pos`, cercle à 3,05 m. Pour un `BasketCourt` : x = -13,18 et `rot` 90,
    /// x = 13,18 et `rot` 270 (planche à 1,2 m de la ligne de fond).
    BasketHoop = "Panier de basket", Sport, DEHORS;
    /// Filet de tennis de 12,8 m le long de l'axe x (poteaux à ±6,4 m), haut de 1,07 m aux poteaux
    /// et de 0,914 m au centre. Au centre d'un `TennisCourt` : même `pos`, `rot` du court + 90.
    TennisNet = "Filet de tennis", Sport, DEHORS;
    /// Filet de volley de 10 m le long de l'axe x, haut de 2,43 m, poteaux à ±5,5 m, antennes à
    /// ±4,5 m. Au centre d'un `VolleyCourt` : même `pos`, `rot` du terrain + 90.
    VolleyNet = "Filet de volley", Sport, DEHORS;
    /// Poteaux de rugby en H le long de l'axe x (5,6 m entre les poteaux, barre à 3 m, 11 m de
    /// haut), centrés sur la ligne de but.
    RugbyPosts = "Poteaux de rugby", Sport, DEHORS;
    /// Table de ping-pong (2,74 × 1,525 m, plateau à 76 cm), filet le long de l'axe z au centre.
    /// Laisser 2 m libres à chaque bout (axe x).
    PingPongTable = "Table de ping-pong", Sport, DEDANS;
    /// Gradins de 6 × 4 m, cinq rangs qui montent vers l'arrière jusqu'à 3 m (garde-corps) ; les
    /// spectateurs regardent l'avant : à tourner vers le terrain, l'arrière contre une clôture.
    Bleachers = "Gradins", Sport, DEHORS;
    /// Banc de touche abrité (4 × 1,5 × 2,2 m), vitré au fond et sur les côtés, six sièges, ouvert
    /// vers l'avant : le long d'une ligne de touche, l'avant vers le terrain, à 2 ou 3 m de la ligne.
    DugoutBench = "Banc de touche", Sport, DEHORS;
    /// Drapeau de corner (mât de 1,5 m en `pos`, fanion vers +x). Sur chaque coin d'un terrain de foot.
    CornerFlag = "Drapeau de corner", Sport, DEHORS;
    /// Mât d'éclairage de stade de 20 m, rampe de six projecteurs tournés vers l'avant et vers le
    /// bas, qui éclaire réellement. Hors du terrain, l'avant vers le terrain, un à chaque coin.
    Floodlight = "Projecteur de stade", Sport, DEHORS;
    /// Tableau d'affichage sur deux pieds (écran de 4 × 2 m entre 3 et 5 m), l'écran vers l'avant.
    /// Derrière un but ou au milieu d'une tribune, tourné vers le terrain.
    Scoreboard = "Tableau d'affichage", Sport, DEHORS;
    /// Banc de musculation plat avec chandelles et barre chargée (2,2 m de large) ; la barre est à
    /// l'arrière, on s'allonge la tête vers l'arrière.
    WeightBench = "Banc de musculation", Sport, DEDANS;
    /// Tapis de course (0,8 × 1,9 m), console à l'arrière (peut aller contre un mur), on monte par l'avant.
    Treadmill = "Tapis de course", Sport, DEDANS;
    /// Sac de frappe de 1,2 m suspendu à sa potence (sac à l'avant, mât à l'arrière). Laisser 1 m
    /// libre autour du sac.
    PunchingBag = "Sac de frappe", Sport, DEDANS;
    /// Haie d'athlétisme (1,18 m de large, barre à 1,067 m) en travers d'un couloir (axe x) ; les
    /// pieds pointent vers l'avant, d'où arrivent les coureurs. Tous les 9,14 m sur 110 m haies.
    Hurdle = "Haie d'athlétisme", Sport, DEHORS;

    // ------------------------------------------------------------ tracés de terrain
    /// Ligne blanche droite de 10 cm de large et de `height` mètres de long, le long de l'axe x,
    /// centrée sur `pos` ; `rot` 90 pour une ligne nord-sud. Plate : les objets se posent dessus.
    FieldLine = "Ligne", Markings, DEHORS, "longueur";
    /// Demi-cercle blanc de rayon `height` mètres, centré sur `pos`, ses deux bouts sur l'axe x et
    /// bombé vers l'avant (+z).
    FieldArc = "Demi-cercle", Markings, DEHORS, "rayon";
    /// Quart de cercle blanc de rayon `height` mètres centré sur `pos`, de l'avant (+z) au +x local
    /// (l'est à `rot` 0). Arc de corner (rayon 1 m) : `pos` sur le coin, l'arc dans le terrain.
    FieldQuarterArc = "Quart de cercle", Markings, DEHORS, "rayon";
    /// Cercle blanc de rayon `height` mètres centré sur `pos` (rond central du foot : 9,15 m).
    FieldCircle = "Cercle", Markings, DEHORS, "rayon";
    /// Point blanc de 22 cm (point de penalty, point central).
    FieldDot = "Point", Markings, DEHORS;
    /// Terrain de foot à 5 (40 × 20 m, longueur le long de x) : surfaces de réparation en arcs de
    /// 6 m, points à 6 et 10 m, rond central de 3 m, sur gazon synthétique. Sur un terrain plat
    /// (`flatten_terrain`) ; buts : `HandballGoal`.
    SoccerPitch5 = "Terrain de foot à 5", Markings, DEHORS;
    /// Terrain de foot à 11 (105 × 68 m, longueur le long de x) : surfaces de réparation et de but,
    /// points de penalty, arcs, rond central, corners, sur gazon synthétique. Sur un terrain plat
    /// (`flatten_terrain`) ; buts : `SoccerGoal`, drapeaux : `CornerFlag` aux coins (±52,5 ; ±34).
    SoccerPitch = "Terrain de foot", Markings, DEHORS;
    /// Terrain de basket FIBA (28 × 15 m, longueur le long de x) : raquettes, lancers francs, arcs
    /// à 3 points (6,75 m), rond central, sur parquet de sport. Paniers : `BasketHoop`.
    BasketCourt = "Terrain de basket", Markings, DEHORS;
    /// Court de tennis (23,77 × 10,97 m, longueur le long de x) : couloirs de double, carrés de
    /// service, sur terre battue. Filet : `TennisNet` au centre, `rot` + 90.
    TennisCourt = "Court de tennis", Markings, DEHORS;
    /// Terrain de volley (18 × 9 m, longueur le long de x) : ligne centrale, lignes d'attaque à 3 m,
    /// sur parquet de sport. Filet : `VolleyNet` au centre, `rot` + 90.
    VolleyCourt = "Terrain de volley", Markings, DEHORS;
    /// Terrain de handball (40 × 20 m, longueur le long de x) : zones de but à 6 m, lignes de jet
    /// franc à 9 m en pointillés, marques des 7 m, sur résine bleue. Buts : `HandballGoal`.
    HandballCourt = "Terrain de hand", Markings, DEHORS;

    // ------------------------------------------------------------ escaliers et structure
    /// Escalier droit qui monte vers l'avant sur `height` mètres (3 = un étage). Le palier d'arrivée doit tomber dans un sol de l'étage au-dessus.
    Stairs = "Escalier", Structure, DEDANS, "hauteur";
    /// Escalier en colimaçon qui monte de `height` mètres, encombrement réduit.
    SpiralStairs = "Colimaçon", Structure, DEDANS, "hauteur";
    /// Escalier quart tournant : une volée vers l'avant jusqu'à un palier, puis une volée vers le +x local de l'objet (l'est à rot 0), sur `height` mètres.
    LStairs = "Quart tournant", Structure, DEDANS, "hauteur";
    /// Escalier demi-tour (2,1 m de large) sur `height` mètres : une volée vers l'avant côté -x local, un palier au fond sur toute la largeur, puis une volée qui revient vers l'arrière côté +x. On arrive en haut à l'arrière, côté +x : c'est là que doit tomber le sol de l'étage.
    UStairs = "Demi-tour", Structure, DEDANS, "hauteur";
    /// Échelle de meunier : escalier raide (environ 50°, 62 cm de large) aux marches sans contremarche entre deux limons, qui monte vers l'avant sur `height` mètres. Pour une mezzanine ou un grenier, peu d'emprise au sol.
    Ladder = "Échelle de meunier", Structure, DEDANS, "hauteur";
    /// Escalier suspendu droit qui monte vers l'avant sur `height` mètres : marches épaisses sans contremarche scellées dans un limon côté -x local (l'ouest à rot 0), à placer contre un mur.
    FloatingStairs = "Escalier suspendu", Structure, DEDANS, "hauteur";
    /// Perron en pierre (3 m de large avec ses murets) : un palier d'un mètre à l'arrière, dos contre la façade sous la porte, et des marches de 16 cm environ qui descendent vers l'avant. `height` = hauteur du seuil : 0,5 m donne 3 marches, 0,65 m en donne 4.
    StoneSteps = "Perron", Structure, DEHORS, "hauteur";
    /// Escalator qui monte vers l'avant sur `height` mètres à 30°, avec un palier d'un mètre en bas et en haut, balustrades vitrées et mains courantes (centre commercial, gare).
    Escalator = "Escalator", Structure, DEDANS, "hauteur";
    /// Rampe d'accès PMR qui monte vers l'avant de `height` mètres à 5 %, soit 20 m de pente par mètre monté, puis un palier de 1,4 m ; 1,2 m de passage entre deux garde-corps. Pour une entrée, `height` de 0,2 à 0,8 m.
    Ramp = "Rampe d'accès", Structure, DEHORS, "hauteur";
    /// Pilier carré de 30 cm, haut de `height` mètres (portique, auvent, véranda).
    Pillar = "Pilier", Structure, DEDANS, "hauteur";
    /// Colonne ronde de 34 cm, avec base et chapiteau, haute de `height` mètres (portique, péristyle, entrée). Pour un poteau carré : `Pillar`.
    Column = "Colonne", Structure, DEDANS, "hauteur";
    /// Module de garde-corps de 2 m le long de l'axe gauche-droite de l'objet (main courante à 1 m, barreaux tous les 11 cm). À aligner tous les 2 m au bord d'une mezzanine, d'un balcon ou d'une trémie ; `rot` 90 pour un bord nord-sud.
    Railing = "Garde-corps", Structure, DEDANS;
    /// Module de balustrade en pierre de 2 m le long de l'axe gauche-droite de l'objet (balustres entre un socle et une tablette, 90 cm de haut), à aligner tous les 2 m comme la haie : terrasse, perron, balcon.
    Balustrade = "Balustrade", Structure, DEHORS;
    /// Arche décorative en pierre en plein cintre, 2,1 m de large et 3 m de haut : un passage de 1,2 m sous un arc qui naît à 2,1 m. On la traverse selon l'axe avant-arrière (entrée de jardin, allée).
    Arch = "Arche", Structure, DEHORS;

    // ------------------------------------------------------------ véhicules
    /// Berline de 4,2 m que l'on peut conduire pendant la visite ; à poser sur une route ou une allée.
    Car = "Voiture", Vehicles, DEHORS;
}

impl PropKind {
    /// Escalier : son palier d'arrivée doit tomber dans un sol de l'étage visé (`maison3d check`).
    pub fn climbs(self) -> bool {
        matches!(
            self,
            Self::Stairs
                | Self::SpiralStairs
                | Self::LStairs
                | Self::UStairs
                | Self::Ladder
                | Self::FloatingStairs
                | Self::StoneSteps
                | Self::Escalator
                | Self::Ramp
        )
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
