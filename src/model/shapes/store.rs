//! Magasin et supermarché : rayons, caisses, froid, étals, vitrines et petits commerces.

use bevy::math::{Quat, Vec3};

use super::{Piece, ball, bx, carcass, cyl, door, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

const RED_GOODS: Slot = Slot::plain("Produits rouges", [0.8, 0.2, 0.2]);
const BLUE_GOODS: Slot = Slot::plain("Produits bleus", [0.2, 0.45, 0.85]);
const YELLOW_GOODS: Slot = Slot::plain("Produits jaunes", [0.95, 0.8, 0.3]);
const GREEN_GOODS: Slot = Slot::plain("Produits verts", [0.35, 0.65, 0.3]);
const WHITE_GOODS: Slot = Slot::plain("Produits blancs", [0.96, 0.96, 0.94]);

// Parties colorables de chaque sorte, la partie 0 en tête.
const SHELF: &[Slot] = &[
    Slot::plain("Montants et tablettes", [0.72, 0.56, 0.4]),
    RED_GOODS,
    Slot::plain("Produits bleus", [0.2, 0.5, 0.8]),
    Slot::plain("Produits jaunes", [0.9, 0.7, 0.1]),
    Slot::plain("Produits verts", [0.3, 0.7, 0.3]),
];

const CHECKOUT: &[Slot] = &[
    Slot::plain("Comptoir", [0.86, 0.87, 0.89]),
    Slot::plain("Tapis roulant", [0.08, 0.08, 0.09]),
    Slot::textured("Rebords", "Inox brossé"),
    Slot::plain("Vitre du scanner", [0.6, 0.12, 0.1]),
    Slot::plain("Terminal et supports", DARK),
    Slot::plain("Écran", [0.35, 0.65, 0.95]),
    Slot::plain("Siège", [0.15, 0.3, 0.6]),
    Slot::plain("Numéro de caisse", [0.2, 0.65, 0.35]),
];

const FRIDGE_CASE: &[Slot] = &[
    Slot::plain("Caisson", WHITE),
    Slot::plain("Intérieur et tablettes", [0.72, 0.75, 0.8]),
    Slot::plain("Éclairage", [0.95, 0.97, 1.0]),
    Slot::plain("Bandeau de prix", [0.15, 0.45, 0.75]),
    WHITE_GOODS,
    YELLOW_GOODS,
    RED_GOODS,
    GREEN_GOODS,
];

const FREEZER_CASE: &[Slot] = &[
    Slot::plain("Caisson", [0.85, 0.86, 0.88]),
    Slot::plain("Intérieur", [0.92, 0.94, 0.96]),
    Slot::plain("Poignées", METAL),
    Slot::plain("Clayettes", [0.7, 0.72, 0.75]),
    Slot::plain("Fronton", [0.15, 0.4, 0.8]),
    BLUE_GOODS,
    RED_GOODS,
    YELLOW_GOODS,
];

const PRODUCE_STAND: &[Slot] = &[
    Slot::textured("Bâti", "Bois foncé"),
    Slot::textured("Cagettes", "Bois clair"),
    Slot::plain("Oranges", [0.95, 0.55, 0.1]),
    Slot::plain("Tomates", [0.85, 0.12, 0.08]),
    Slot::plain("Salades", [0.45, 0.75, 0.25]),
    Slot::plain("Citrons", [0.97, 0.88, 0.25]),
    Slot::plain("Aubergines", [0.35, 0.12, 0.4]),
    Slot::plain("Pommes vertes", [0.6, 0.82, 0.25]),
];

const SHOPPING_CART: &[Slot] = &[
    Slot::textured("Grillage", "Chrome"),
    Slot::plain("Poignée", [0.8, 0.12, 0.1]),
    Slot::plain("Roulettes", [0.1, 0.1, 0.11]),
];

const BASKET_STACK: &[Slot] = &[
    Slot::plain("Paniers", [0.8, 0.1, 0.12]),
    Slot::plain("Anses", [0.1, 0.1, 0.11]),
    Slot::plain("Socle", [0.55, 0.57, 0.6]),
];

const GONDOLA: &[Slot] = &[
    Slot::plain("Structure", [0.85, 0.86, 0.88]),
    Slot::plain("Tablettes", [0.7, 0.72, 0.75]),
    RED_GOODS,
    BLUE_GOODS,
    YELLOW_GOODS,
    GREEN_GOODS,
    WHITE_GOODS,
    Slot::plain("Panneau", [0.9, 0.45, 0.1]),
];

const PALLET: &[Slot] = &[
    Slot::textured("Palette", "Bois clair"),
    Slot::plain("Cartons", [0.72, 0.55, 0.36]),
    Slot::plain("Étiquettes", WHITE),
];

const AISLE_SIGN: &[Slot] =
    &[Slot::plain("Panneau", [0.1, 0.35, 0.7]), Slot::plain("Inscriptions", WHITE), Slot::plain("Suspentes", METAL)];

const DISPLAY_CASE: &[Slot] = &[
    Slot::textured("Meuble", "Bois foncé"),
    Slot::textured("Plateau", "Marbre blanc"),
    Slot::plain("Pains", [0.78, 0.52, 0.25]),
    Slot::plain("Viennoiseries", [0.88, 0.62, 0.22]),
    Slot::plain("Gâteaux", [0.4, 0.22, 0.12]),
    Slot::plain("Crème", [0.97, 0.94, 0.88]),
    Slot::plain("Tartes aux fruits", [0.85, 0.2, 0.25]),
];

const SECURITY_GATE: &[Slot] =
    &[Slot::plain("Cadres", [0.82, 0.83, 0.85]), Slot::plain("Socles", DARK), Slot::plain("Voyants", [0.9, 0.15, 0.1])];

const CLOTHES_RACK: &[Slot] = &[
    Slot::textured("Structure", "Chrome"),
    Slot::plain("Cintres", DARK),
    Slot::plain("Vêtements bleus", [0.2, 0.3, 0.6]),
    Slot::plain("Vêtements rouges", [0.75, 0.15, 0.2]),
    Slot::plain("Vêtements blancs", WHITE),
    Slot::plain("Vêtements beiges", [0.8, 0.7, 0.55]),
];

const MANNEQUIN: &[Slot] = &[
    Slot::plain("Corps", [0.93, 0.91, 0.88]),
    Slot::plain("Robe", [0.2, 0.25, 0.5]),
    Slot::textured("Socle", "Chrome"),
];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Shelf => SHELF,
        PropKind::Checkout => CHECKOUT,
        PropKind::FridgeCase => FRIDGE_CASE,
        PropKind::FreezerCase => FREEZER_CASE,
        PropKind::ProduceStand => PRODUCE_STAND,
        PropKind::ShoppingCart => SHOPPING_CART,
        PropKind::BasketStack => BASKET_STACK,
        PropKind::Gondola => GONDOLA,
        PropKind::Pallet => PALLET,
        PropKind::AisleSign => AISLE_SIGN,
        PropKind::DisplayCase => DISPLAY_CASE,
        PropKind::SecurityGate => SECURITY_GATE,
        PropKind::ClothesRack => CLOTHES_RACK,
        PropKind::Mannequin => MANNEQUIN,
        _ => &[],
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    match kind {
        PropKind::Shelf => shelf(),
        PropKind::Checkout => checkout(),
        PropKind::FridgeCase => fridge_case(),
        PropKind::FreezerCase => freezer_case(open),
        PropKind::ProduceStand => produce_stand(),
        PropKind::ShoppingCart => shopping_cart(),
        PropKind::BasketStack => basket_stack(),
        PropKind::Gondola => gondola(),
        PropKind::Pallet => pallet(),
        PropKind::AisleSign => aisle_sign(),
        PropKind::DisplayCase => display_case(),
        PropKind::SecurityGate => security_gate(),
        PropKind::ClothesRack => clothes_rack(),
        PropKind::Mannequin => mannequin(),
        _ => Vec::new(),
    }
}

fn shelf() -> Vec<Piece> {
    let mut v = Vec::new();
    for x in [-0.49, 0.49] {
        v.push(bx(x, 0.0, 0.02, 0.4, 0.0, 1.8, 0));
    }
    v.push(bx(0.0, -0.19, 1.0, 0.02, 0.0, 1.8, 0));
    for k in 0..5 {
        let y = 0.08 + k as f32 * 0.42;
        v.push(bx(0.0, 0.0, 0.96, 0.38, y, y + 0.025, 0));
    }
    // marchandises colorées sur les tablettes
    for k in 0..4 {
        let y = 0.105 + k as f32 * 0.42;
        v.push(bx(-0.2 + (k % 2) as f32 * 0.4, 0.02, 0.4, 0.28, y, y + 0.25, 1 + k));
    }
    v
}

/// Caisse : le client longe l'avant (+z), le caissier est assis à l'arrière.
fn checkout() -> Vec<Piece> {
    vec![
        bx(0.0, 0.3, 2.0, 0.6, 0.0, 0.85, 0),
        bx(-0.35, 0.3, 1.2, 0.46, 0.85, 0.88, 1),
        bx(-0.35, 0.545, 1.24, 0.03, 0.85, 0.92, 2),
        bx(-0.35, 0.055, 1.24, 0.03, 0.85, 0.92, 2),
        // scanner : boîtier et sa vitre tournée vers les articles
        bx(0.45, 0.1, 0.34, 0.06, 0.85, 1.1, 4),
        bx(0.45, 0.135, 0.26, 0.01, 0.9, 1.05, 3),
        bx(0.45, 0.3, 0.34, 0.3, 0.85, 0.865, 3),
        // terminal de paiement côté client
        cyl(0.8, 0.5, 0.03, 0.85, 1.0, 4),
        bx(0.8, 0.5, 0.1, 0.16, 1.0, 1.04, 4),
        // écran tourné vers le caissier, tiroir-caisse sous le comptoir
        cyl(0.85, 0.15, 0.04, 0.85, 1.12, 4),
        bx(0.85, 0.15, 0.32, 0.04, 1.12, 1.34, 4),
        bx(0.85, 0.125, 0.28, 0.01, 1.14, 1.32, 5),
        bx(0.45, -0.01, 0.4, 0.02, 0.62, 0.76, 4),
        // mât et numéro de caisse
        cyl(-0.95, 0.08, 0.05, 0.85, 1.9, 2),
        bx(-0.95, 0.08, 0.22, 0.22, 1.9, 2.12, 7),
        // siège pivotant du caissier
        cyl(0.4, -0.35, 0.45, 0.0, 0.04, 4),
        cyl(0.4, -0.35, 0.05, 0.04, 0.6, 4),
        cyl(0.4, -0.35, 0.42, 0.6, 0.68, 6),
        bx(0.4, -0.58, 0.04, 0.03, 0.64, 0.78, 4),
        bx(0.4, -0.58, 0.38, 0.05, 0.75, 1.05, 6),
    ]
}

/// Meuble réfrigéré ouvert à l'avant : un bac en bas et trois tablettes de plus en plus courtes.
fn fridge_case() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 0.0, 2.0, 0.9, 0.0, 0.5, 0),
        bx(0.0, 0.43, 2.0, 0.04, 0.5, 0.68, 0),
        bx(0.0, 0.455, 1.96, 0.01, 0.15, 0.4, 3),
        bx(0.0, -0.43, 2.0, 0.04, 0.5, 2.0, 0),
        bx(0.0, -0.1, 2.0, 0.7, 1.9, 2.0, 0),
        bx(0.0, 0.2, 1.92, 0.06, 1.86, 1.9, 2),
        bx(0.0, -0.405, 1.92, 0.01, 0.5, 1.9, 1),
    ];
    for x in [-0.98, 0.98] {
        v.push(bx(x, 0.0, 0.04, 0.9, 0.0, 0.95, 0));
        v.push(bx(x, -0.1, 0.04, 0.7, 0.95, 2.0, 0));
    }
    for (y, d) in [(0.95, 0.55), (1.3, 0.5), (1.65, 0.42)] {
        v.push(bx(0.0, -0.41 + d / 2.0, 1.92, d, y - 0.02, y, 1));
    }
    for (k, (y, d)) in [(0.5, 0.8), (0.95, 0.55), (1.3, 0.5), (1.65, 0.42)].into_iter().enumerate() {
        for i in 0..4 {
            let h = 0.18 + 0.04 * ((i + k) % 2) as f32;
            v.push(bx(-0.72 + i as f32 * 0.48, -0.41 + d / 2.0, 0.44, d - 0.06, y, y + h, 4 + (i + k) % 4));
        }
    }
    v
}

/// Congélateur à trois portes vitrées qui s'ouvrent, produits visibles derrière le verre.
fn freezer_case(open: f32) -> Vec<Piece> {
    let mut v = carcass(2.1, 0.8, 2.1, 0, 1).to_vec();
    v.push(bx(0.0, 0.385, 2.1, 0.03, 0.0, 0.1, 0));
    v.push(bx(0.0, 0.385, 2.1, 0.03, 1.85, 2.1, 4));
    for x in [-0.35, 0.35] {
        v.push(bx(x, 0.385, 0.04, 0.03, 0.1, 1.85, 0));
    }
    for y in [0.55, 1.0, 1.45] {
        v.push(bx(0.0, -0.02, 2.04, 0.68, y - 0.02, y, 3));
    }
    for (i, x) in [-0.7, 0.0, 0.7].into_iter().enumerate() {
        for (k, y) in [0.08, 0.55, 1.0, 1.45].into_iter().enumerate() {
            v.push(bx(x, -0.05, 0.6, 0.55, y, y + 0.28, 5 + (i + k) % 3));
        }
        let [mut pane, handle] = door(open, (x - 0.35, 0.4), 0.7, (0.1, 1.85), 1.0, (0, 2));
        pane.shape = Shape::Glass;
        v.extend([pane, handle]);
    }
    v
}

/// Étal à deux gradins de cagettes inclinées vers le client (avant).
fn produce_stand() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 1.9, 1.0, 0.15, 0.18, 0)];
    for x in [-0.97, 0.97] {
        v.push(bx(x, 0.55, 0.05, 0.05, 0.0, 0.62, 0));
        v.push(bx(x, -0.55, 0.05, 0.05, 0.0, 1.08, 0));
        v.push(span(Shape::Cube, Vec3::new(x, 0.62, 0.55), Vec3::new(x, 1.08, -0.55), 0.05, 0));
    }
    for (k, x) in [-0.63, 0.0, 0.63].into_iter().enumerate() {
        produce_crate(&mut v, Vec3::new(x, 0.78, 0.27), 2 + k);
        produce_crate(&mut v, Vec3::new(x, 1.0, -0.27), 5 + k);
    }
    v
}

/// Cagette penchée vers l'avant, garnie d'un tas de fruits de la partie `fruit`.
fn produce_crate(v: &mut Vec<Piece>, c: Vec3, fruit: usize) {
    let q = Quat::from_rotation_x(0.35);
    let at = |x: f32, y: f32, z: f32| (c + q * Vec3::new(x, y, z)).to_array();
    v.push(Piece::new(Shape::Cube, c, Vec3::new(0.6, 0.16, 0.48), 1).turn(q));
    v.push(ball(at(0.0, 0.07, 0.0), [0.54, 0.14, 0.42], fruit).turn(q));
    for (x, z) in [(-0.16, 0.1), (0.14, -0.08), (0.02, 0.14), (-0.06, -0.12)] {
        v.push(ball(at(x, 0.12, z), [0.11; 3], fruit));
    }
}

/// Chariot : panier grillagé en tubes, plus étroit à l'avant (+z), poignée à l'arrière.
fn shopping_cart() -> Vec<Piece> {
    // point du panier : `f` de -1 (gauche) à 1 (droite), `t` de l'arrière à l'avant, `up` du fond au bord
    let p = |f: f32, t: f32, up: f32| {
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        let bottom = Vec3::new(f * lerp(0.25, 0.21), 0.5, lerp(-0.3, 0.45));
        let top = Vec3::new(f * lerp(0.28, 0.24), 0.95, lerp(-0.4, 0.52));
        bottom.lerp(top, up)
    };
    let tube = |a: Vec3, b: Vec3| span(Shape::Cylinder, a, b, 0.012, 0);
    let mut v = Vec::new();
    for f in [-1.0, 1.0] {
        v.extend((0..=6).map(|i| tube(p(f, i as f32 / 6.0, 0.0), p(f, i as f32 / 6.0, 1.0))));
        v.extend([0.0, 0.5, 1.0].map(|up| tube(p(f, 0.0, up), p(f, 1.0, up))));
    }
    for t in [0.0, 1.0] {
        v.extend([-0.5, 0.0, 0.5].map(|f| tube(p(f, t, 0.0), p(f, t, 1.0))));
        v.extend([0.0, 0.5, 1.0].map(|up| tube(p(-1.0, t, up), p(1.0, t, up))));
    }
    v.extend([-0.5, 0.0, 0.5].map(|f| tube(p(f, 0.0, 0.0), p(f, 1.0, 0.0))));
    // châssis, roulettes et poignée
    for s in [-1.0, 1.0] {
        let (back, front) = (Vec3::new(s * 0.22, 0.13, -0.42), Vec3::new(s * 0.18, 0.13, 0.43));
        v.push(span(Shape::Cylinder, back, front, 0.025, 0));
        v.push(span(Shape::Cylinder, back, Vec3::new(s * 0.27, 1.02, -0.5), 0.025, 0));
        v.push(span(Shape::Cylinder, front, p(s * 0.8, 1.0, 0.0), 0.02, 0));
        for c in [back, front] {
            v.push(bx(c.x, c.z, 0.02, 0.04, 0.06, 0.13, 0));
            v.push(Piece::new(Shape::Wheel, Vec3::new(c.x, 0.06, c.z), Vec3::new(0.03, 0.12, 0.12), 2));
        }
    }
    v.push(span(Shape::Cylinder, Vec3::new(-0.29, 1.02, -0.5), Vec3::new(0.29, 1.02, -0.5), 0.035, 1));
    v
}

/// Six paniers emboîtés sur leur socle, les anses du dernier relevées.
fn basket_stack() -> Vec<Piece> {
    let top = 0.3 + 5.0 * 0.04;
    let mut v = vec![bx(0.0, 0.0, 0.5, 0.38, 0.0, 0.04, 2), bx(0.0, 0.0, 0.4, 0.28, 0.04, top, 0)];
    for k in 0..6 {
        let y = 0.3 + k as f32 * 0.04;
        v.push(bx(0.0, 0.0, 0.46, 0.34, y - 0.015, y, 0));
    }
    for z in [-0.12, 0.12] {
        let p = |x: f32, y: f32| Vec3::new(x, y, z);
        let arc = [p(-0.16, top), p(-0.1, top + 0.12), p(0.1, top + 0.12), p(0.16, top)];
        v.extend(arc.windows(2).map(|w| span(Shape::Cylinder, w[0], w[1], 0.015, 1)));
    }
    v
}

/// Gondole double face : produits à l'avant et à l'arrière, panneau en tête au-dessus.
fn gondola() -> Vec<Piece> {
    let mut v = vec![
        bx(0.0, 0.0, 2.0, 0.05, 0.0, 1.8, 0),
        bx(0.0, 0.0, 2.0, 0.96, 0.0, 0.1, 0),
        bx(0.0, 0.0, 1.2, 0.04, 1.95, 2.3, 7),
    ];
    for x in [-0.4, 0.4] {
        v.push(cyl(x, 0.0, 0.03, 1.8, 1.95, 0));
    }
    for x in [-0.985, 0.985] {
        v.push(bx(x, 0.0, 0.03, 0.96, 0.1, 1.8, 0));
    }
    for (side, s) in [-1.0_f32, 1.0].into_iter().enumerate() {
        for y in [0.5, 0.9, 1.3] {
            v.push(bx(0.0, s * 0.255, 1.94, 0.43, y - 0.02, y, 1));
        }
        for (k, y) in [0.1, 0.5, 0.9, 1.3].into_iter().enumerate() {
            for i in 0..4 {
                let h = 0.22 + 0.06 * ((i + k) % 3) as f32;
                v.push(bx(-0.72 + i as f32 * 0.48, s * 0.25, 0.44, 0.34, y, y + h, 2 + (i + 2 * k + side) % 5));
            }
        }
    }
    v
}

/// Palette Europe (1,2 × 0,8 m) chargée de cartons sous film transparent.
fn pallet() -> Vec<Piece> {
    let mut v = Vec::new();
    for z in [-0.35, 0.0, 0.35] {
        v.push(bx(0.0, z, 1.2, 0.1, 0.0, 0.022, 0));
        for x in [-0.525, 0.0, 0.525] {
            v.push(bx(x, z, 0.145, 0.1, 0.022, 0.1, 0));
        }
    }
    for x in [-0.55, 0.0, 0.55] {
        v.push(bx(x, 0.0, 0.1, 0.8, 0.1, 0.122, 0));
    }
    for z in [-0.33, -0.165, 0.0, 0.165, 0.33] {
        v.push(bx(0.0, z, 1.2, 0.12, 0.122, 0.144, 0));
    }
    for k in 0..3 {
        let y = 0.144 + k as f32 * 0.38;
        for (x, z) in [(-0.3, -0.2), (0.3, -0.2), (-0.3, 0.2), (0.3, 0.2)] {
            v.push(bx(x, z, 0.58, 0.38, y, y + 0.375, 1));
        }
        for x in [-0.3, 0.3] {
            v.push(bx(x, 0.392, 0.2, 0.01, y + 0.2, y + 0.32, 2));
        }
    }
    v.push(Piece::new(Shape::Glass, Vec3::new(0.0, 0.722, 0.0), Vec3::new(1.2, 1.15, 0.8), 0));
    v
}

/// Panneau de rayon accroché par deux suspentes au plafond d'un étage de 3 m.
fn aisle_sign() -> Vec<Piece> {
    let mut v = vec![bx(0.0, 0.0, 1.2, 0.04, 2.2, 2.6, 0)];
    for z in [-0.022, 0.022] {
        v.push(bx(0.12, z, 0.8, 0.005, 2.36, 2.44, 1));
        v.push(bx(-0.45, z, 0.18, 0.005, 2.3, 2.5, 1));
    }
    for x in [-0.45, 0.45] {
        v.push(cyl(x, 0.0, 0.01, 2.6, 2.98, 2));
        v.push(bx(x, 0.0, 0.08, 0.08, 2.98, 3.0, 2));
    }
    v
}

/// Vitrine de boulangerie : meuble bas, vitre inclinée vers le client (+z), pains et gâteaux.
fn display_case() -> Vec<Piece> {
    let glass = |at: [f32; 3], size: [f32; 3]| Piece::new(Shape::Glass, Vec3::from(at), Vec3::from(size), 0);
    let mut front = span(Shape::Glass, Vec3::new(0.0, 0.87, 0.4), Vec3::new(0.0, 1.3, 0.15), 0.01, 0);
    front.size.x = 1.56;
    let mut v = vec![
        bx(0.0, 0.0, 1.6, 0.85, 0.0, 0.85, 0),
        bx(0.0, 0.0, 1.54, 0.79, 0.85, 0.87, 1),
        front,
        glass([0.0, 1.305, -0.12], [1.56, 0.01, 0.55]),
        glass([0.0, 1.085, -0.4], [1.56, 0.43, 0.01]),
        glass([0.0, 1.08, -0.12], [1.5, 0.01, 0.45]),
        bx(0.0, 0.15, 1.6, 0.03, 1.29, 1.32, 0),
        bx(0.0, 0.41, 1.6, 0.03, 0.85, 0.9, 0),
    ];
    for x in [-0.785, 0.785] {
        v.push(glass([x, 1.085, -0.125], [0.01, 0.43, 0.55]));
        v.push(bx(x, -0.4, 0.03, 0.03, 0.87, 1.32, 0));
    }
    // pains et viennoiseries sur le plateau, gâteaux et tartes sur la clayette
    for z in [0.25, 0.13, 0.01] {
        v.push(ball([-0.42, 0.905, z], [0.5, 0.07, 0.08], 2));
        v.push(ball([0.55, 0.9, z], [0.13, 0.06, 0.07], 3));
    }
    for z in [0.2, -0.1] {
        v.push(ball([0.12, 0.93, z], [0.24, 0.13, 0.24], 2));
    }
    for x in [-0.5, -0.18] {
        v.push(cyl(x, -0.12, 0.22, 1.085, 1.165, 4));
        v.push(cyl(x, -0.12, 0.22, 1.165, 1.18, 5));
    }
    for x in [0.2, 0.52] {
        v.push(cyl(x, -0.12, 0.24, 1.085, 1.11, 6));
    }
    v
}

/// Portique antivol : deux panneaux de part et d'autre d'un passage de 0,9 m (on passe selon z).
fn security_gate() -> Vec<Piece> {
    let mut v = Vec::new();
    for x in [-0.5, 0.5] {
        v.push(bx(x, 0.0, 0.1, 0.6, 0.0, 0.06, 1));
        for z in [-0.27, 0.27] {
            v.push(bx(x, z, 0.06, 0.06, 0.06, 1.55, 0));
        }
        v.push(bx(x, 0.0, 0.06, 0.6, 1.55, 1.6, 0));
        v.push(bx(x, 0.0, 0.06, 0.48, 0.75, 0.79, 0));
        v.push(Piece::new(Shape::Glass, Vec3::new(x, 0.8, 0.0), Vec3::new(0.02, 1.45, 0.48), 0));
        v.push(bx(x, 0.0, 0.065, 0.5, 1.6, 1.62, 2));
    }
    v
}

/// Portant chromé de 1,5 m, vêtements suspendus perpendiculairement à la barre.
fn clothes_rack() -> Vec<Piece> {
    let mut v = vec![span(Shape::Cylinder, Vec3::new(-0.74, 1.55, 0.0), Vec3::new(0.74, 1.55, 0.0), 0.03, 0)];
    for x in [-0.72, 0.72] {
        v.push(bx(x, 0.0, 0.04, 0.55, 0.0, 0.04, 0));
        v.push(cyl(x, 0.0, 0.035, 0.04, 1.55, 0));
    }
    for i in 0..9 {
        let x = -0.6 + i as f32 * 0.15;
        // une robe longue entre deux chemises
        let hem = if i % 3 == 1 { 0.55 } else { 0.85 };
        v.push(cyl(x, 0.0, 0.01, 1.5, 1.56, 1));
        v.push(bx(x, 0.0, 0.025, 0.4, 1.48, 1.5, 1));
        v.push(bx(x, 0.0, 0.06, 0.42, hem, 1.48, 2 + i % 4));
    }
    v
}

/// Mannequin de vitrine sur pied, face vers l'avant (+z), en robe.
fn mannequin() -> Vec<Piece> {
    let mut v = vec![
        cyl(0.0, 0.0, 0.4, 0.0, 0.03, 2),
        cyl(0.0, -0.06, 0.03, 0.03, 0.6, 2),
        Piece::new(Shape::Cone, Vec3::new(0.0, 0.83, 0.0), Vec3::new(0.5, 0.66, 0.4), 1),
        ball([0.0, 1.28, 0.0], [0.36, 0.45, 0.22], 1),
        cyl(0.0, 0.0, 0.08, 1.46, 1.58, 0),
        ball([0.0, 1.68, 0.0], [0.18, 0.24, 0.2], 0),
    ];
    for s in [-1.0, 1.0] {
        v.push(cyl(s * 0.07, 0.0, 0.1, 0.05, 0.55, 0));
        v.push(bx(s * 0.07, 0.04, 0.08, 0.18, 0.03, 0.08, 0));
        v.push(span(Shape::Cylinder, Vec3::new(s * 0.19, 1.44, 0.0), Vec3::new(s * 0.25, 1.06, 0.03), 0.07, 0));
        v.push(span(Shape::Cylinder, Vec3::new(s * 0.25, 1.06, 0.03), Vec3::new(s * 0.24, 0.84, 0.06), 0.06, 0));
    }
    v
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::{Category, LEVEL_H, Prop};

    fn local_box(kind: PropKind) -> (Vec3, Vec3) {
        Prop::new(kind, Vec2::ZERO, 0, 0.0).local_box()
    }

    #[test]
    fn store_kinds_stand_on_the_floor_around_their_origin() {
        for k in Category::Store.kinds() {
            let (lo, hi) = local_box(k);
            let (c, s) = ((lo + hi) / 2.0, hi - lo);
            assert!(lo.y > -1e-3, "{k:?} sous le sol : {lo}");
            assert!(c.x.abs() < 0.1 + 0.1 * s.x && c.z.abs() < 0.1 + 0.1 * s.z, "{k:?} décentré : {c}");
        }
    }
    #[test]
    fn only_the_freezer_and_its_glass_doors_open() {
        assert!(PropKind::FreezerCase.openable(), "portes vitrées");
        for k in [PropKind::FridgeCase, PropKind::Checkout, PropKind::Gondola, PropKind::DisplayCase] {
            assert!(!k.openable(), "{k:?}");
        }
        let doors = shapes(PropKind::FreezerCase, 1.0, LEVEL_H);
        assert_eq!(doors.iter().filter(|p| p.shape == Shape::Glass).count(), 3);
    }
    #[test]
    fn aisle_sign_hangs_from_a_three_metre_ceiling() {
        let (lo, hi) = local_box(PropKind::AisleSign);
        assert!(lo.y > 2.0 && (hi.y - LEVEL_H).abs() < 1e-3, "{lo} {hi}");
    }
    #[test]
    fn gondola_has_goods_on_both_faces() {
        let goods = |front: bool| {
            let pieces = shapes(PropKind::Gondola, 0.0, LEVEL_H);
            pieces.iter().filter(|p| (2..7).contains(&p.slot) && (p.at.z > 0.0) == front).count()
        };
        assert_eq!((goods(true), goods(false)), (16, 16));
    }
    #[test]
    fn produce_crates_lean_towards_the_customer() {
        let pieces = shapes(PropKind::ProduceStand, 0.0, LEVEL_H);
        let crates: Vec<_> = pieces.iter().filter(|p| p.slot == 1).collect();
        assert_eq!(crates.len(), 6);
        assert!(crates.iter().all(|p| (p.rot * Vec3::Y).z > 0.2), "dessus tourné vers l'avant");
    }
}
