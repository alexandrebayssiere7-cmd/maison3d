//! Primitives des objets posés, dans le repère de l'objet (avant vers +z, y vers le haut), un
//! fichier par famille de catégories. Chaque fichier donne, pour ses sortes, les pièces (`shapes`)
//! et les parties colorables (`slots`) auxquelles ces pièces renvoient.

use bevy::math::{Quat, Vec3};

use super::{Category, PropKind, Shape, Slot};

mod garage;
mod garden;
mod kitchen;
mod living;
mod office;
mod plants;
mod sport;
mod store;
mod street;
mod structure;
mod vehicles;

/// Primitive locale d'un objet : forme unitaire mise à l'échelle `size`, centrée en `at`,
/// tournée de `rot`, aux couleur et matière de la partie `slot` de l'objet.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Piece {
    pub(super) shape: Shape,
    pub(super) at: Vec3,
    pub(super) size: Vec3,
    pub(super) slot: usize,
    pub(super) rot: Quat,
}

impl Piece {
    fn new(shape: Shape, at: Vec3, size: Vec3, slot: usize) -> Self {
        Self { shape, at, size, slot, rot: Quat::IDENTITY }
    }
    /// Tourne la pièce sur elle-même de `a` radians autour de la verticale (+x vers -z).
    fn yaw(self, a: f32) -> Self {
        self.turn(Quat::from_rotation_y(a))
    }
    /// Tourne la pièce sur elle-même (penchée, couchée…).
    fn turn(mut self, q: Quat) -> Self {
        self.rot = q * self.rot;
        self
    }
}

/// Pièces de l'objet, la face avant vers +z ; `open` (0..1) ouvre ses portes, `height` est sa
/// dimension réglable (`PropKind::dim`).
pub(super) fn prop_shapes(kind: PropKind, open: f32, height: f32) -> Vec<Piece> {
    match kind.category() {
        Category::Living | Category::Bedroom => living::shapes(kind, open, height),
        Category::Kitchen | Category::Bathroom => kitchen::shapes(kind, open, height),
        Category::Store => store::shapes(kind, open, height),
        Category::Street | Category::Signage => street::shapes(kind, open, height),
        Category::Trees | Category::Plants => plants::shapes(kind, open, height),
        Category::Garden | Category::Leisure => garden::shapes(kind, open, height),
        Category::Structure => structure::shapes(kind, open, height),
        Category::Vehicles => vehicles::shapes(kind, open, height),
        Category::Office => office::shapes(kind, open, height),
        Category::Garage => garage::shapes(kind, open, height),
        Category::Sport | Category::Markings => sport::shapes(kind, open, height),
    }
}

/// Parties colorables de l'objet ; la partie 0 reçoit `color` et `texture` de l'objet. Les listes
/// sont écrites `const { &[Slot::plain(…), …] }` pour rester statiques.
pub fn prop_slots(kind: PropKind) -> &'static [Slot] {
    match kind.category() {
        Category::Living | Category::Bedroom => living::slots(kind),
        Category::Kitchen | Category::Bathroom => kitchen::slots(kind),
        Category::Store => store::slots(kind),
        Category::Street | Category::Signage => street::slots(kind),
        Category::Trees | Category::Plants => plants::slots(kind),
        Category::Garden | Category::Leisure => garden::slots(kind),
        Category::Structure => structure::slots(kind),
        Category::Vehicles => vehicles::slots(kind),
        Category::Office => office::slots(kind),
        Category::Garage => garage::slots(kind),
        Category::Sport | Category::Markings => sport::slots(kind),
    }
}

// ---------------------------------------------------------------- briques communes

/// Boîte posée entre y0 et y1, centrée en (x, z).
fn bx(x: f32, z: f32, sx: f32, sz: f32, y0: f32, y1: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cube, Vec3::new(x, f32::midpoint(y0, y1), z), Vec3::new(sx, y1 - y0, sz), slot)
}

/// Cylindre vertical de diamètre `d` posé entre y0 et y1, centré en (x, z).
fn cyl(x: f32, z: f32, d: f32, y0: f32, y1: f32, slot: usize) -> Piece {
    Piece::new(Shape::Cylinder, Vec3::new(x, f32::midpoint(y0, y1), z), Vec3::new(d, y1 - y0, d), slot)
}

/// Sphère (ou ellipsoïde) de diamètres `size` centrée en `at`.
fn ball(at: [f32; 3], size: [f32; 3], slot: usize) -> Piece {
    Piece::new(Shape::Sphere, Vec3::from(at), Vec3::from(size), slot)
}

/// Barre de section `d` (boîte ou cylindre selon `shape`) tendue de `a` à `b` : montants
/// penchés, tubes, câbles, rampes.
fn span(shape: Shape, a: Vec3, b: Vec3, d: f32, slot: usize) -> Piece {
    let v = b - a;
    let len = v.length();
    let rot = if len > 1e-6 { Quat::from_rotation_arc(Vec3::Y, v / len) } else { Quat::IDENTITY };
    Piece::new(shape, (a + b) / 2.0, Vec3::new(d, len, d), slot).turn(rot)
}

/// Source de lumière ponctuelle en `at`, à la couleur de la partie `slot`.
fn light(at: [f32; 3], slot: usize) -> Piece {
    Piece::new(Shape::Light, Vec3::from(at), Vec3::ONE, slot)
}

/// Boules de feuillage (centre, taille).
fn clumps(c: &[([f32; 3], [f32; 3])], slot: usize) -> Vec<Piece> {
    c.iter().map(|&(p, s)| ball(p, s, slot)).collect()
}

/// Porte de largeur `w` accrochée en `hinge` = (x, z), entre les hauteurs `ys` ; `side` = 1 si
/// elle s'étend vers +x depuis la charnière. Parties `(battant, poignée)`.
fn door(open: f32, hinge: (f32, f32), w: f32, ys: (f32, f32), side: f32, slots: (usize, usize)) -> [Piece; 2] {
    let ((hx, hz), (y0, y1)) = (hinge, ys);
    let rot = -side * open * 1.7;
    let q = Quat::from_rotation_y(rot);
    let at = |x: f32, z: f32| Vec3::new(hx, f32::midpoint(y0, y1), hz) + q * Vec3::new(side * x, 0.0, z);
    let handle = Vec3::new(0.025, 0.2_f32.min(y1 - y0 - 0.1), 0.04);
    [
        Piece::new(Shape::Cube, at(w / 2.0, 0.02), Vec3::new(w - 0.01, y1 - y0, 0.03), slots.0).yaw(rot),
        Piece::new(Shape::Cube, at(w - 0.07, 0.06), handle, slots.1).yaw(rot),
    ]
}

/// Caisson ouvert à l'avant (pour voir l'intérieur quand les portes s'ouvrent) ; le fond intérieur
/// prend la partie `inside`.
fn carcass(w: f32, d: f32, h: f32, slot: usize, inside: usize) -> [Piece; 5] {
    let t = 0.025;
    [
        bx(0.0, -d / 2.0 + t / 2.0, w, t, 0.0, h, slot),
        bx(-w / 2.0 + t / 2.0, 0.0, t, d, 0.0, h, slot),
        bx(w / 2.0 - t / 2.0, 0.0, t, d, 0.0, h, slot),
        bx(0.0, 0.0, w, d, h - t, h, slot),
        bx(0.0, 0.0, w - 2.0 * t, d - t, 0.0, 0.08, inside),
    ]
}

/// Pièces fermées qui pivotent ensemble de `q` autour d'une charnière passant par `hinge` :
/// abattant de four (`from_rotation_x(a)`, le haut part vers l'avant), couvercle
/// (`from_rotation_x(-a)`, l'avant se lève), hublot ou porte vitrée (`from_rotation_y`).
fn swing<const N: usize>(pieces: [Piece; N], hinge: Vec3, q: Quat) -> [Piece; N] {
    pieces.map(|p| Piece { at: hinge + q * (p.at - hinge), ..p }.turn(q))
}

/// Pièces décalées de `d` (caisson posé plus haut ou plus en avant).
fn shift<const N: usize>(pieces: [Piece; N], d: Vec3) -> [Piece; N] {
    pieces.map(|p| Piece { at: p.at + d, ..p })
}

/// Tiroir de largeur `w` et de profondeur `d` dont la façade affleure en (x, z), entre les
/// hauteurs `ys`, sorti de `out` mètres vers l'avant. Parties `(façade et côtés, poignée, fond)`.
fn drawer(x: f32, z: f32, w: f32, d: f32, ys: (f32, f32), out: f32, slots: (usize, usize, usize)) -> [Piece; 5] {
    let ((y0, y1), z) = (ys, z + out);
    let (ym, mid) = (f32::midpoint(y0, y1), z - d / 2.0);
    let side = |sx: f32| bx(x + sx * (w / 2.0 - 0.035), mid, 0.012, d - 0.02, y0 + 0.01, y1 - 0.03, slots.0);
    [
        bx(x, z - 0.01, w - 0.01, 0.02, y0, y1, slots.0),
        bx(x, z + 0.01, (w * 0.35).min(0.16), 0.02, ym - 0.012, ym + 0.012, slots.1),
        bx(x, mid, w - 0.06, d - 0.02, y0 + 0.01, y0 + 0.02, slots.2),
        side(-1.0),
        side(1.0),
    ]
}

/// Quatre pieds de 5 cm aux coins (±hx, ±hz), hauts de h.
fn legs(hx: f32, hz: f32, h: f32, slot: usize) -> [Piece; 4] {
    [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sz)| bx(sx * hx, sz * hz, 0.05, 0.05, 0.0, h, slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_shapes_and_uses_each_of_its_slots() {
        for k in PropKind::ALL {
            let pieces = prop_shapes(k, 0.0, 3.0);
            assert!(!pieces.is_empty(), "{k:?} sans pièce");
            let slots = prop_slots(k);
            for (i, s) in slots.iter().enumerate() {
                assert!(!s.name.is_empty(), "{k:?} partie {i} sans nom");
                let known = s.texture.is_empty() || crate::textures::find(s.texture) != 0;
                assert!(known, "{k:?} : texture « {} » inconnue (`maison3d textures`)", s.texture);
                assert!(pieces.iter().any(|p| p.slot == i), "{k:?} : partie « {} » sans pièce", s.name);
            }
            assert!(pieces.iter().all(|p| p.slot < slots.len()), "{k:?} : pièce hors des parties");
            let names: std::collections::BTreeSet<_> = slots.iter().map(|s| s.name).collect();
            assert_eq!(names.len(), slots.len(), "{k:?} : noms de parties en double");
            for p in &pieces {
                assert!(p.size.is_finite() && p.at.is_finite() && p.rot.is_normalized(), "{k:?} : {p:?}");
            }
        }
    }
    #[test]
    fn doors_swing() {
        let closed = prop_shapes(PropKind::Wardrobe, 0.0, 3.0);
        let open = prop_shapes(PropKind::Wardrobe, 1.0, 3.0);
        assert!(closed.iter().zip(&open).any(|(a, b)| a.rot.angle_between(b.rot) > 1e-3), "les portes pivotent");
    }
    #[test]
    fn only_kinds_with_doors_open() {
        assert!(PropKind::Wardrobe.openable() && PropKind::Kitchen.openable() && PropKind::Fridge.openable());
        assert!(!PropKind::Sofa.openable() && !PropKind::Car.openable());
    }
    #[test]
    fn swing_turns_about_the_hinge_and_shift_moves() {
        let p = bx(0.0, 0.0, 0.6, 0.02, 0.0, 1.0, 0);
        let [q] = swing([p], Vec3::ZERO, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        assert!(q.at.abs_diff_eq(Vec3::new(0.0, 0.0, 0.5), 1e-5), "le centre tourne autour de la charnière");
        let [s] = shift([p], Vec3::Y);
        assert_eq!((s.at.y, s.rot), (1.5, p.rot));
    }
    #[test]
    fn drawer_slides_forward_by_its_travel() {
        let at = |out: f32| drawer(0.0, 0.2, 0.4, 0.3, (0.1, 0.3), out, (0, 1, 2));
        let (shut, out) = (at(0.0), at(0.25));
        for (a, b) in shut.iter().zip(&out) {
            assert!((b.at - a.at).abs_diff_eq(Vec3::Z * 0.25, 1e-6) && a.size == b.size, "{a:?} {b:?}");
        }
        assert!((shut[0].at.z + shut[0].size.z / 2.0 - 0.2).abs() < 1e-6, "façade affleurante");
    }
    #[test]
    fn span_follows_its_ends() {
        let bar = span(Shape::Cube, Vec3::ZERO, Vec3::new(3.0, 4.0, 0.0), 0.1, 0);
        assert!((bar.size.y - 5.0).abs() < 1e-5 && bar.at.abs_diff_eq(Vec3::new(1.5, 2.0, 0.0), 1e-5));
        assert!((bar.rot * Vec3::Y).abs_diff_eq(Vec3::new(0.6, 0.8, 0.0), 1e-5), "l'axe suit la barre");
        assert_eq!(span(Shape::Cylinder, Vec3::ONE, Vec3::ONE, 0.1, 0).rot, Quat::IDENTITY);
    }
}
