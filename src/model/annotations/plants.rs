//! Annotations : arbres, plantes et fleurs.

use super::Placement::{Floor, OnTop, Row};
use super::Room::{Garden, Hall, Living, Office, Store, Street};
use super::{Annotation, note};
use crate::model::PropKind as K;

pub(super) const NOTES: &[(K, Annotation)] = &[
    (K::Tree, note(Floor, &[Garden, Street], 0.0, &[K::Bush], &[], &["arbre", "arbre feuillu", "feuillu", "tree"])),
    (K::Pine, note(Floor, &[Garden], 0.0, &[K::Birch], &[], &["sapin", "pin", "conifère", "pine", "fir"])),
    (K::Birch, note(Floor, &[Garden], 0.0, &[K::Pine], &[], &["bouleau", "birch"])),
    (K::Palm, note(Floor, &[Garden, Street], 0.0, &[K::Lounger], &[], &["palmier", "tropical", "palm tree"])),
    (K::Cypress, note(Row, &[Garden], 0.0, &[K::OliveTree], &[], &["cyprès", "conifère étroit", "cypress"])),
    (
        K::AppleTree,
        note(
            Floor,
            &[Garden],
            0.0,
            &[K::VegetablePatch],
            &[],
            &["pommier", "arbre fruitier", "verger", "apple tree", "fruit tree"],
        ),
    ),
    (K::Oak, note(Floor, &[Garden], 0.0, &[K::Bench], &[], &["chêne", "grand arbre", "oak"])),
    (K::WeepingWillow, note(Floor, &[Garden], 0.0, &[K::Pond], &[], &["saule pleureur", "saule", "weeping willow"])),
    (
        K::OliveTree,
        note(Floor, &[Garden], 0.0, &[K::Lavender, K::Cypress], &[], &["olivier", "méditerranéen", "olive tree"]),
    ),
    (K::Maple, note(Floor, &[Garden], 0.0, &[K::Bush], &[], &["érable rouge", "érable", "automne", "maple"])),
    (
        K::CherryBlossom,
        note(
            Floor,
            &[Garden],
            0.0,
            &[K::StoneLantern, K::Pond],
            &[],
            &["cerisier du japon", "sakura", "japonais", "cherry blossom"],
        ),
    ),
    (K::PlaneTree, note(Row, &[Street], 0.0, &[K::Bench], &[], &["platane", "arbre d'alignement", "plane tree"])),
    (
        K::DeadTree,
        note(Floor, &[Garden], 0.0, &[K::Stump, K::Rock], &[], &["arbre mort", "arbre sec", "friche", "dead tree"]),
    ),
    (
        K::Plant,
        note(
            Floor,
            &[Living, Office, Hall, Store],
            0.0,
            &[K::Sofa],
            &[],
            &["plante", "plante verte", "plante en pot", "house plant"],
        ),
    ),
    (K::Bush, note(Floor, &[Garden], 0.0, &[K::Tree], &[], &["buisson", "arbuste", "bush", "shrub"])),
    (K::Flowers, note(Floor, &[Garden], 0.0, &[K::Rosebush], &[], &["fleurs", "massif", "parterre", "flower bed"])),
    (K::Rosebush, note(Floor, &[Garden], 0.0, &[K::Flowers], &[], &["rosier", "roses", "rosebush"])),
    (K::Sunflower, note(Floor, &[Garden], 0.0, &[K::Flowers], &[], &["tournesols", "tournesol", "sunflower"])),
    (K::Cactus, note(Floor, &[Garden], 0.0, &[K::Rock], &[], &["cactus", "désert", "succulente"])),
    (K::Bamboo, note(Row, &[Garden], 0.0, &[K::Fence], &[], &["bambous", "bambou", "écran végétal", "bamboo"])),
    (K::Fern, note(Floor, &[Garden], 0.0, &[K::Stump, K::Mushrooms], &[], &["fougère", "sous-bois", "fern"])),
    (
        K::TallGrass,
        note(Floor, &[Garden], 0.0, &[K::Lavender], &[], &["herbes hautes", "graminées", "herbes", "tall grass"]),
    ),
    (K::Reeds, note(Floor, &[Garden], 0.0, &[K::Pond], &[], &["roseaux", "massettes", "bord de l'eau", "reeds"])),
    (K::Lavender, note(Row, &[Garden], 0.0, &[K::OliveTree], &[], &["lavande", "provence", "bordure", "lavender"])),
    (K::Tulips, note(Floor, &[Garden], 0.0, &[K::Flowers], &[], &["tulipes", "tulipe", "plate-bande", "tulips"])),
    (
        K::Hydrangea,
        note(Floor, &[Garden], 0.0, &[K::Bush], &[], &["hortensia", "hortensias", "fleurs bleues", "hydrangea"]),
    ),
    (K::Rock, note(Floor, &[Garden], 0.0, &[K::Cactus], &[], &["rocher", "pierre", "rocaille", "rock", "boulder"])),
    (K::Stump, note(Floor, &[Garden], 0.0, &[K::Fern, K::Mushrooms], &[], &["souche", "tronc coupé", "stump"])),
    (
        K::Mushrooms,
        note(Floor, &[Garden], 0.0, &[K::Stump, K::Fern], &[], &["champignons", "champignon", "amanite", "mushrooms"]),
    ),
    (K::Topiary, note(Floor, &[Garden, Hall], 0.0, &[K::Gate], &[], &["buis en boule", "buis", "boule", "topiary"])),
    (
        K::Ficus,
        note(Floor, &[Living, Office, Hall], 0.0, &[K::Sofa], &[], &["ficus", "grande plante", "plante d'intérieur"]),
    ),
    (
        K::Bonsai,
        note(OnTop, &[Living, Office], 0.0, &[K::Sideboard], &[], &["bonsaï", "bonsai", "petit arbre", "bonsai tree"]),
    ),
];
