//! Salon ; la chambre est dans `bedroom`. Le tiroir (`drawer`) sert aussi au bureau.

use std::f32::consts::TAU;

use bevy::math::Vec3;

use super::{Piece, ball, bx, carcass, cyl, door, legs, light, span};
use crate::model::{DARK, METAL, PropKind, Shape, Slot, WHITE};

mod bedroom;

const WOOD: [f32; 3] = [0.55, 0.37, 0.2];
const LACQUER: [f32; 3] = [0.05, 0.05, 0.06];
const WARM_LIGHT: [f32; 3] = [1.0, 0.9, 0.7];

// listes longues hors du `match` pour que `slots` reste lisible
const COFFEE_TABLE: &[Slot] = &[
    Slot::textured("Plateau", "Bois clair"),
    Slot::plain("Pieds", DARK),
    Slot::textured("Tablette", "Bois clair"),
    Slot::plain("Magazines", [0.8, 0.3, 0.25]),
];
const TV_STAND: &[Slot] = &[
    Slot::textured("Meuble", "Bois foncé"),
    Slot::plain("Pieds", DARK),
    Slot::plain("Façades", [0.9, 0.9, 0.88]),
    Slot::plain("Téléviseur", LACQUER),
    Slot::plain("Écran", [0.04, 0.06, 0.1]),
    Slot::plain("Box", [0.2, 0.2, 0.22]),
];
const BOOKCASE: &[Slot] = &[
    Slot::textured("Bois", "Bois clair"),
    Slot::plain("Livres rouges", [0.7, 0.15, 0.12]),
    Slot::plain("Livres bleus", [0.15, 0.3, 0.6]),
    Slot::plain("Livres verts", [0.2, 0.5, 0.3]),
    Slot::plain("Livres jaunes", [0.9, 0.75, 0.3]),
];
const SIDEBOARD: &[Slot] = &[
    Slot::textured("Bois", "Bois foncé"),
    Slot::plain("Poignées", METAL),
    Slot::plain("Intérieur", [0.85, 0.82, 0.75]),
    Slot::textured("Plateau", "Marbre blanc"),
    Slot::plain("Vaisselle", WHITE),
];
const RUG: &[Slot] = &[
    Slot::plain("Fond", [0.62, 0.18, 0.16]),
    Slot::plain("Bordure", [0.2, 0.25, 0.45]),
    Slot::plain("Motif", [0.9, 0.8, 0.55]),
    Slot::plain("Franges", [0.92, 0.9, 0.82]),
];
const FIREPLACE: &[Slot] = &[
    Slot::textured("Pierre", "Pierre de taille"),
    Slot::plain("Foyer", [0.08, 0.07, 0.07]),
    Slot::textured("Bûches", "Écorce"),
    Slot::plain("Flammes et lumière", [1.0, 0.5, 0.12]),
];
const PIANO: &[Slot] = &[
    Slot::plain("Laque", LACQUER),
    Slot::plain("Touches blanches", WHITE),
    Slot::plain("Touches noires", [0.02, 0.02, 0.02]),
    Slot::plain("Pédales", [0.8, 0.65, 0.3]),
    Slot::plain("Partition", [0.97, 0.96, 0.9]),
];
const AQUARIUM: &[Slot] = &[
    Slot::textured("Meuble", "Bois foncé"),
    Slot::plain("Cadre", DARK),
    Slot::plain("Eau", [0.1, 0.38, 0.75]),
    Slot::textured("Sable", "Sable"),
    Slot::plain("Plantes", [0.2, 0.6, 0.25]),
    Slot::plain("Poissons", [1.0, 0.5, 0.1]),
];

pub(super) fn slots(kind: PropKind) -> &'static [Slot] {
    match kind {
        PropKind::Table => const { &[Slot::plain("Plateau", WOOD), Slot::plain("Pieds", WOOD)] },
        PropKind::Chair => const { &[Slot::plain("Assise et dossier", WOOD), Slot::plain("Pieds", WOOD)] },
        PropKind::Sofa => const { &[Slot::plain("Tissu", [0.3, 0.42, 0.62]), Slot::plain("Pieds", DARK)] },
        PropKind::Lamp => const { &[Slot::plain("Abat-jour et lumière", WARM_LIGHT), Slot::plain("Pied", DARK)] },
        PropKind::Armchair => {
            const {
                &[
                    Slot::textured("Tissu", "Tissu beige"),
                    Slot::plain("Coussin", [0.8, 0.74, 0.64]),
                    Slot::plain("Pieds", WOOD),
                ]
            }
        }
        PropKind::CoffeeTable => COFFEE_TABLE,
        PropKind::RoundTable => const { &[Slot::textured("Plateau", "Bois foncé"), Slot::plain("Pied", DARK)] },
        PropKind::CornerSofa => {
            const {
                &[
                    Slot::textured("Tissu", "Tissu gris"),
                    Slot::plain("Coussins", [0.55, 0.56, 0.6]),
                    Slot::plain("Pieds", DARK),
                ]
            }
        }
        PropKind::TvStand => TV_STAND,
        PropKind::Bookcase => BOOKCASE,
        PropKind::Sideboard => SIDEBOARD,
        PropKind::Rug => RUG,
        PropKind::Fireplace => FIREPLACE,
        PropKind::Piano => PIANO,
        PropKind::CeilingLamp => {
            const {
                &[
                    Slot::plain("Abat-jour et lumière", WARM_LIGHT),
                    Slot::plain("Monture", [0.75, 0.6, 0.3]),
                    Slot::plain("Câble", DARK),
                ]
            }
        }
        PropKind::Aquarium => AQUARIUM,
        PropKind::Ottoman => {
            const {
                &[
                    Slot::textured("Tissu", "Velours bleu"),
                    Slot::textured("Dessus", "Velours bleu"),
                    Slot::plain("Pieds", WOOD),
                ]
            }
        }
        _ => bedroom::slots(kind),
    }
}

pub(super) fn shapes(kind: PropKind, open: f32, _height: f32) -> Vec<Piece> {
    let mut v = Vec::new();
    match kind {
        PropKind::Table => {
            v.push(bx(0.0, 0.0, 1.6, 0.9, 0.72, 0.77, 0));
            v.extend(legs(0.72, 0.38, 0.72, 1));
        }
        PropKind::Chair => {
            v.push(bx(0.0, 0.0, 0.45, 0.45, 0.43, 0.48, 0));
            v.push(bx(0.0, -0.2, 0.45, 0.05, 0.48, 0.95, 0));
            v.extend(legs(0.19, 0.19, 0.43, 1));
        }
        PropKind::Sofa => {
            v.push(bx(0.0, 0.0, 2.0, 0.9, 0.1, 0.45, 0));
            v.push(bx(0.0, -0.35, 2.0, 0.2, 0.45, 0.85, 0));
            v.push(bx(-0.9, 0.0, 0.2, 0.9, 0.45, 0.65, 0));
            v.push(bx(0.9, 0.0, 0.2, 0.9, 0.45, 0.65, 0));
            v.extend(legs(0.9, 0.38, 0.1, 1));
        }
        PropKind::Lamp => {
            v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.3, 0.04, 0.3), 1));
            v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 0.8, 0.0), Vec3::new(0.04, 1.6, 0.04), 1));
            v.push(Piece::new(Shape::Sphere, Vec3::new(0.0, 1.7, 0.0), Vec3::new(0.4, 0.3, 0.4), 0));
            v.push(light([0.0, 1.5, 0.0], 0));
        }
        PropKind::Armchair => armchair(&mut v),
        PropKind::CoffeeTable => coffee_table(&mut v),
        PropKind::RoundTable => {
            v.push(cyl(0.0, 0.0, 1.1, 0.72, 0.76, 0));
            v.push(cyl(0.0, 0.0, 0.1, 0.04, 0.72, 1));
            v.push(cyl(0.0, 0.0, 0.55, 0.0, 0.04, 1));
        }
        PropKind::CornerSofa => corner_sofa(&mut v),
        PropKind::TvStand => tv_stand(&mut v),
        PropKind::Bookcase => bookcase(&mut v),
        PropKind::Sideboard => sideboard(&mut v, open),
        PropKind::Rug => rug(&mut v),
        PropKind::Fireplace => fireplace(&mut v),
        PropKind::Piano => piano(&mut v),
        PropKind::CeilingLamp => ceiling_lamp(&mut v),
        PropKind::Aquarium => aquarium(&mut v),
        PropKind::Ottoman => {
            v.push(cyl(0.0, 0.0, 0.5, 0.04, 0.4, 0));
            v.push(cyl(0.0, 0.0, 0.47, 0.4, 0.44, 1));
            v.extend(legs(0.15, 0.15, 0.04, 2));
        }
        _ => return bedroom::shapes(kind, open),
    }
    v
}

fn armchair(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 0.85, 0.82, 0.12, 0.42, 0));
    v.push(bx(0.0, 0.08, 0.55, 0.62, 0.42, 0.52, 1));
    v.push(bx(0.0, -0.32, 0.85, 0.18, 0.42, 0.9, 0));
    for x in [-0.35, 0.35] {
        v.push(bx(x, 0.0, 0.15, 0.82, 0.42, 0.66, 0));
    }
    v.extend(legs(0.36, 0.35, 0.12, 2));
}

fn coffee_table(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.1, 0.6, 0.38, 0.42, 0));
    v.push(bx(0.0, 0.0, 1.0, 0.5, 0.1, 0.12, 2));
    v.push(bx(0.2, 0.03, 0.3, 0.22, 0.12, 0.14, 3));
    v.extend(legs(0.5, 0.25, 0.38, 1));
}

/// Assise le long du dossier (-z), retour en méridienne vers l'avant côté +x.
fn corner_sofa(v: &mut Vec<Piece>) {
    v.push(bx(0.0, -0.45, 2.6, 0.9, 0.1, 0.4, 0));
    v.push(bx(0.75, 0.45, 1.1, 0.9, 0.1, 0.4, 0));
    v.push(bx(0.0, -0.8, 2.6, 0.2, 0.4, 0.85, 0));
    v.push(bx(1.2, 0.1, 0.2, 1.6, 0.4, 0.85, 0));
    v.push(bx(-1.2, -0.35, 0.2, 0.7, 0.4, 0.62, 0));
    let seats = [(-0.73, -0.35, 0.74, 0.7), (0.01, -0.35, 0.74, 0.7), (0.75, -0.35, 0.7, 0.7), (0.75, 0.4, 0.7, 0.9)];
    for (x, z, w, d) in seats {
        v.push(bx(x, z, w, d, 0.4, 0.5, 1));
    }
    let backs =
        [(-0.73, -0.63, 0.66, 0.14), (0.01, -0.63, 0.66, 0.14), (1.03, -0.3, 0.14, 0.66), (1.03, 0.4, 0.14, 0.66)];
    for (x, z, w, d) in backs {
        v.push(bx(x, z, w, d, 0.5, 0.8, 1));
    }
    for (x, z) in [(-1.25, -0.85), (1.25, -0.85), (-1.25, -0.05), (0.25, 0.85), (1.25, 0.85)] {
        v.push(bx(x, z, 0.05, 0.05, 0.0, 0.1, 2));
    }
}

fn tv_stand(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.6, 0.42, 0.06, 0.5, 0));
    v.extend(legs(0.75, 0.17, 0.06, 1));
    for x in [-0.52, 0.52] {
        v.push(bx(x, 0.215, 0.5, 0.012, 0.09, 0.47, 2));
    }
    v.push(bx(-0.6, 0.02, 0.3, 0.2, 0.5, 0.56, 5));
    // téléviseur sur son pied, écran vers l'avant
    v.push(bx(0.0, -0.05, 0.4, 0.22, 0.5, 0.52, 3));
    v.push(bx(0.0, -0.08, 0.06, 0.04, 0.52, 0.64, 3));
    v.push(bx(0.0, -0.05, 1.24, 0.05, 0.6, 1.32, 3));
    v.push(bx(0.0, -0.022, 1.2, 0.006, 0.62, 1.3, 4));
}

fn bookcase(v: &mut Vec<Piece>) {
    const W: [f32; 6] = [0.12, 0.16, 0.1, 0.18, 0.14, 0.12];
    for x in [-0.49, 0.49] {
        v.push(bx(x, 0.0, 0.02, 0.32, 0.0, 2.0, 0));
    }
    v.push(bx(0.0, -0.15, 1.0, 0.02, 0.0, 2.0, 0));
    v.push(bx(0.0, 0.0, 1.0, 0.32, 1.97, 2.0, 0));
    // cinq rayons de livres aux largeurs, hauteurs et couleurs mêlées
    for r in 0..5 {
        let y = 0.04 + r as f32 * 0.38;
        v.push(bx(0.0, 0.0, 0.96, 0.3, y, y + 0.025, 0));
        let mut x = -0.46;
        for b in 0..6 {
            let (w, h) = (W[(b + r) % 6], 0.22 + 0.04 * ((b + 2 * r) % 3) as f32);
            v.push(bx(x + w / 2.0, 0.0, w - 0.005, 0.22, y + 0.025, y + 0.025 + h, 1 + (b + r) % 4));
            x += w;
        }
    }
}

fn sideboard(v: &mut Vec<Piece>, open: f32) {
    v.extend(carcass(1.8, 0.45, 0.82, 0, 2));
    v.push(bx(0.0, 0.0, 1.84, 0.48, 0.82, 0.86, 3));
    v.push(bx(0.0, 0.0, 1.75, 0.4, 0.42, 0.44, 0));
    for x in [-0.6, 0.0, 0.6] {
        v.push(cyl(x, 0.0, 0.26, 0.44, 0.52, 4));
    }
    for (hx, side) in [(-0.9, 1.0), (-0.3, 1.0), (0.9, -1.0)] {
        v.extend(door(open, (hx, 0.225), 0.6, (0.09, 0.8), side, (0, 1)));
    }
}

/// Moins de 5 cm d'épaisseur : passe sous les meubles sans alerte.
fn rug(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.8, 1.2, 0.0, 0.033, 0));
    v.push(bx(0.0, 0.0, 2.0, 1.4, 0.0, 0.03, 1));
    v.push(Piece::new(Shape::Cylinder, Vec3::new(0.0, 0.018, 0.0), Vec3::new(0.9, 0.036, 0.6), 2));
    for x in [-1.03, 1.03] {
        v.push(bx(x, 0.0, 0.06, 1.3, 0.0, 0.01, 3));
    }
}

/// Coffre de pierre dos au mur, foyer ouvert à l'avant où brûle un feu.
fn fireplace(v: &mut Vec<Piece>) {
    let z = -0.075;
    for x in [-0.55, 0.55] {
        v.push(bx(x, z, 0.3, 0.45, 0.0, 0.95, 0));
    }
    v.push(bx(0.0, z, 0.8, 0.45, 0.75, 0.95, 0));
    v.push(bx(0.0, z + 0.02, 1.5, 0.5, 0.95, 1.02, 0));
    v.push(bx(0.0, 0.25, 1.5, 0.2, 0.0, 0.04, 0));
    v.push(bx(0.0, -0.27, 0.8, 0.06, 0.0, 0.75, 1));
    v.push(bx(0.0, z, 0.8, 0.45, 0.0, 0.06, 1));
    v.push(span(Shape::Cylinder, Vec3::new(-0.25, 0.1, -0.05), Vec3::new(0.25, 0.1, 0.0), 0.09, 2));
    v.push(span(Shape::Cylinder, Vec3::new(-0.18, 0.17, 0.06), Vec3::new(0.2, 0.15, -0.12), 0.08, 2));
    for (x, h) in [(-0.12, 0.28), (0.0, 0.38), (0.13, 0.25)] {
        v.push(Piece::new(Shape::Cone, Vec3::new(x, 0.14 + h / 2.0, -0.04), Vec3::new(0.14, h, 0.1), 3));
    }
    v.push(light([0.0, 0.35, 0.05], 3));
}

fn piano(v: &mut Vec<Piece>) {
    v.push(bx(0.0, -0.13, 1.5, 0.34, 0.0, 1.25, 0));
    v.push(bx(0.0, 0.14, 1.38, 0.2, 0.6, 0.7, 0));
    for x in [-0.72, 0.72] {
        v.push(bx(x, 0.1, 0.06, 0.28, 0.55, 0.82, 0));
        v.push(bx(x, 0.2, 0.06, 0.06, 0.0, 0.55, 0));
    }
    v.push(bx(0.0, 0.15, 1.38, 0.18, 0.7, 0.73, 1));
    // touches noires, deux puis trois par octave, sur sept octaves
    let key = 1.32 / 49.0;
    for o in 0..7 {
        for k in [1, 2, 4, 5, 6] {
            v.push(bx(-0.66 + (o * 7 + k) as f32 * key, 0.11, 0.015, 0.1, 0.73, 0.745, 2));
        }
    }
    for x in [-0.1, 0.0, 0.1] {
        v.push(bx(x, 0.1, 0.035, 0.1, 0.05, 0.08, 3));
    }
    v.push(bx(0.0, 0.045, 0.32, 0.01, 0.85, 1.1, 4));
}

/// Lustre à cinq bras accroché sous la dalle de l'étage au-dessus (plafond à 2,8 m).
fn ceiling_lamp(v: &mut Vec<Piece>) {
    const TOP: f32 = 2.8;
    v.push(cyl(0.0, 0.0, 0.14, TOP - 0.04, TOP, 1));
    v.push(cyl(0.0, 0.0, 0.012, 2.1, TOP - 0.04, 2));
    v.push(ball([0.0, 2.08, 0.0], [0.09, 0.09, 0.09], 1));
    for k in 0..5 {
        let (s, c) = (k as f32 * TAU / 5.0).sin_cos();
        let end = Vec3::new(0.3 * s, 2.14, 0.3 * c);
        v.push(span(Shape::Cylinder, Vec3::new(0.0, 2.08, 0.0), end, 0.015, 1));
        v.push(Piece::new(Shape::Cone, end + Vec3::Y * 0.06, Vec3::splat(0.14), 0));
    }
    v.push(light([0.0, 1.95, 0.0], 0));
}

/// Bac vitré sur son meuble : le verre enveloppe l'eau, le fond bleu lui donne sa couleur.
fn aquarium(v: &mut Vec<Piece>) {
    v.push(bx(0.0, 0.0, 1.2, 0.45, 0.0, 0.75, 0));
    v.push(bx(0.0, 0.0, 1.2, 0.45, 0.75, 0.78, 1));
    v.push(bx(0.0, 0.0, 1.2, 0.45, 1.25, 1.3, 1));
    v.push(bx(0.0, 0.0, 1.16, 0.41, 0.78, 0.83, 3));
    v.push(bx(0.0, -0.2, 1.16, 0.01, 0.83, 1.25, 2));
    for (x, h) in [(-0.45, 0.3), (-0.36, 0.22), (0.42, 0.26)] {
        v.push(Piece::new(Shape::Cone, Vec3::new(x, 0.83 + h / 2.0, -0.12), Vec3::new(0.08, h, 0.06), 4));
    }
    for (x, y, z) in [(-0.15, 1.0, 0.05), (0.2, 1.08, -0.05), (0.05, 0.93, 0.08), (0.35, 1.0, 0.02)] {
        v.push(ball([x, y, z], [0.1, 0.05, 0.03], 5));
    }
    v.push(Piece::new(Shape::Glass, Vec3::new(0.0, 1.015, 0.0), Vec3::new(1.2, 0.47, 0.45), 0));
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::*;
    use crate::model::{Category, House, LEVEL_H, Prop};

    fn salon_and_bedroom() -> impl Iterator<Item = PropKind> {
        Category::Living.kinds().chain(Category::Bedroom.kinds())
    }

    #[test]
    fn only_real_doors_drawers_and_lids_open() {
        let opening =
            [PropKind::Sideboard, PropKind::Wardrobe, PropKind::Nightstand, PropKind::Dresser, PropKind::ToyChest];
        for k in salon_and_bedroom() {
            assert_eq!(k.openable(), opening.contains(&k), "{k:?}");
        }
    }
    #[test]
    fn only_the_rug_is_flat() {
        assert!(salon_and_bedroom().filter(|k| k.flat()).eq([PropKind::Rug]));
    }
    #[test]
    fn rug_under_a_table_raises_no_warning() {
        let mut h = House::default();
        let mut run = |cmd: &str| h.apply(serde_json::from_str(cmd).unwrap()).unwrap();
        run(r#"{"op":"room","min":[0,0],"max":[5,4],"level":0}"#);
        run(r#"{"op":"place_prop","kind":"Table","pos":[2.5,2],"level":0}"#);
        run(r#"{"op":"place_prop","kind":"Rug","pos":[2.5,2],"level":0}"#);
        assert_eq!(h.props.len(), 2);
        assert!(h.warnings().is_empty(), "{:?}", h.warnings());
        // garde-fou : un vrai meuble au même endroit, lui, chevauche la table
        h.apply(serde_json::from_str(r#"{"op":"place_prop","kind":"Armchair","pos":[2.5,2],"level":0}"#).unwrap())
            .unwrap();
        assert_eq!(h.warnings().len(), 1, "{:?}", h.warnings());
    }
    #[test]
    fn fire_and_ceiling_lamp_light_the_room() {
        let lit = |k: PropKind| {
            shapes(k, 0.0, LEVEL_H).into_iter().find(|p| p.shape == Shape::Light).map(|p| k.slots()[p.slot].color)
        };
        let fire = lit(PropKind::Fireplace).expect("le feu éclaire");
        assert!(fire[0] > fire[1] && fire[1] > fire[2], "lumière orangée : {fire:?}");
        assert!(lit(PropKind::CeilingLamp).is_some());
    }
    #[test]
    fn ceiling_lamp_hangs_from_the_ceiling_above_heads() {
        let (lo, hi) = Prop::new(PropKind::CeilingLamp, Vec2::ZERO, 0, 0.0).local_box();
        assert!((hi.y - (LEVEL_H - 0.2)).abs() < 1e-4, "sous la dalle du dessus : {hi}");
        assert!(lo.y > 2.0, "on passe dessous : {lo}");
    }
}
