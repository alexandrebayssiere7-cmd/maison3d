//! Types de portes et fenêtres, et leur dessin dans le trou que leur laisse le mur.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};
use bevy::transform::components::Transform;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DARK, METAL, Opening, Rgb, Shape, Slot, WHITE, local_box};

/// Type d'ouverture dans un mur ; dimensions par défaut dans `maison3d catalog`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub enum OpeningKind {
    /// Porte pleine en bois (0,9 × 2,1 m), qui s'ouvre pendant la visite.
    Door,
    /// Fenêtre (1,2 × 1,2 m, allège 0,9 m).
    Window,
    /// Baie vitrée (2,4 × 2,2 m, au sol), pour un séjour qui donne sur le jardin.
    Bay,
    /// Porte-fenêtre à deux battants (1,4 × 2,15 m, au sol), qui s'ouvre pendant la visite.
    FrenchDoor,
    /// Grande vitre fixe (2 × 2,4 m, allège 0,3 m), ne s'ouvre pas.
    Fixed,
    /// Fenêtre à petits carreaux, style ancien (1 × 1,3 m, allège 0,9 m).
    Grid,
    /// Baie coulissante (2,4 × 2,15 m, au sol) : deux vantaux vitrés, celui de droite glisse
    /// derrière celui de gauche pendant la visite.
    SlidingDoor,
    /// Porte japonaise (shoji, 1,8 × 2 m, au sol) : deux panneaux de bois quadrillés tendus de
    /// papier ; celui de droite coulisse derrière l'autre.
    Shoji,
    /// Porte à galandage (0,83 × 2,04 m, au sol) : le battant glisse vers la droite et disparaît
    /// dans l'épaisseur du mur ; laisser une largeur de mur pleine à sa droite.
    PocketDoor,
    /// Porte de grange coulissante (1 × 2,1 m, au sol) : vantail en applique côté intérieur du
    /// mur, suspendu à un rail visible ; il glisse vers la droite, laisser une largeur de mur libre.
    BarnDoor,
    /// Porte vitrée (0,9 × 2,15 m, au sol) : battant tout verre à cadre fin, qui pivote.
    GlassDoor,
    /// Porte double (1,5 × 2,15 m, au sol) : deux battants pleins qui pivotent, pour une entrée.
    DoubleDoor,
    /// Porte de garage basculante (2,4 × 2 m, au sol) : le tablier pivote vers le haut et
    /// rentre à l'horizontale sous le plafond, côté intérieur.
    GarageDoor,
    /// Porte automatique (1,8 × 2,2 m, au sol) : deux vantaux vitrés qui s'écartent dans le
    /// mur, caisson du moteur au-dessus ; entrée de magasin ou de bureau.
    AutoDoor,
    /// Fenêtre coulissante (1,4 × 1,15 m, allège 1 m) : deux châssis vitrés, celui de droite
    /// glisse derrière l'autre.
    SlidingWindow,
}

impl OpeningKind {
    /// Dans l'ordre des variantes (`k as usize` indexe les réglages de l'éditeur).
    pub const ALL: [OpeningKind; 15] = [
        Self::Door,
        Self::Window,
        Self::Bay,
        Self::FrenchDoor,
        Self::Fixed,
        Self::Grid,
        Self::SlidingDoor,
        Self::Shoji,
        Self::PocketDoor,
        Self::BarnDoor,
        Self::GlassDoor,
        Self::DoubleDoor,
        Self::GarageDoor,
        Self::AutoDoor,
        Self::SlidingWindow,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Door => "Porte",
            Self::Window => "Fenêtre",
            Self::Bay => "Baie vitrée",
            Self::FrenchDoor => "Porte-fenêtre",
            Self::Fixed => "Vitre fixe",
            Self::Grid => "Fenêtre à carreaux",
            Self::SlidingDoor => "Baie coulissante",
            Self::Shoji => "Porte japonaise",
            Self::PocketDoor => "Porte à galandage",
            Self::BarnDoor => "Porte de grange coulissante",
            Self::GlassDoor => "Porte vitrée",
            Self::DoubleDoor => "Porte double",
            Self::GarageDoor => "Porte de garage",
            Self::AutoDoor => "Porte automatique",
            Self::SlidingWindow => "Fenêtre coulissante",
        }
    }
    /// (largeur, hauteur, allège)
    pub fn default_dims(self) -> (f32, f32, f32) {
        match self {
            Self::Door => (0.9, 2.1, 0.0),
            Self::Window => (1.2, 1.2, 0.9),
            Self::Bay => (2.4, 2.2, 0.0),
            Self::FrenchDoor => (1.4, 2.15, 0.0),
            Self::Fixed => (2.0, 2.4, 0.3),
            Self::Grid => (1.0, 1.3, 0.9),
            Self::SlidingDoor => (2.4, 2.15, 0.0),
            Self::Shoji => (1.8, 2.0, 0.0),
            Self::PocketDoor => (0.83, 2.04, 0.0),
            Self::BarnDoor => (1.0, 2.1, 0.0),
            Self::GlassDoor => (0.9, 2.15, 0.0),
            Self::DoubleDoor => (1.5, 2.15, 0.0),
            Self::GarageDoor => (2.4, 2.0, 0.0),
            Self::AutoDoor => (1.8, 2.2, 0.0),
            Self::SlidingWindow => (1.4, 1.15, 1.0),
        }
    }
    /// Parties colorables ; la partie 0 reçoit `color` et `texture` de l'ouverture. Le verre ne se peint pas.
    pub fn slots(self) -> &'static [Slot] {
        const FRAME: Slot = Slot::plain("Cadre", WHITE);
        const SILL: Slot = Slot::plain("Appui", WHITE);
        const KNOB: Slot = Slot::plain("Poignée", METAL);
        const ALU: Slot = Slot::plain("Cadre", [0.3, 0.31, 0.33]);
        match self {
            Self::Door => const { &[Slot::plain("Battant", [0.5, 0.33, 0.2]), FRAME, KNOB] },
            Self::FrenchDoor => &[FRAME],
            Self::Window | Self::Bay | Self::Fixed | Self::Grid | Self::SlidingWindow => &[FRAME, SILL],
            Self::SlidingDoor | Self::GlassDoor => &[ALU, KNOB],
            Self::Shoji => {
                const { &[Slot::plain("Cadre", [0.8, 0.64, 0.44]), Slot::textured("Papier", "Papier de riz")] }
            }
            Self::PocketDoor => const { &[Slot::plain("Battant", [0.9, 0.88, 0.84]), FRAME, KNOB] },
            Self::BarnDoor => {
                const { &[Slot::plain("Battant", [0.55, 0.36, 0.22]), FRAME, Slot::plain("Ferrures", DARK)] }
            }
            Self::DoubleDoor => {
                const { &[Slot::plain("Battants", [0.42, 0.27, 0.16]), FRAME, Slot::plain("Poignées", [0.8, 0.65, 0.3])] }
            }
            Self::GarageDoor => {
                const { &[Slot::plain("Tablier", [0.86, 0.87, 0.87]), FRAME, Slot::plain("Poignée", DARK)] }
            }
            Self::AutoDoor => const { &[ALU, Slot::plain("Caisson", [0.22, 0.23, 0.25])] },
        }
    }
    pub fn default_color(self) -> Rgb {
        self.slots()[0].color
    }
}

/// Largeur du cadre dormant.
const F: f32 = 0.07;

/// Trou d'une ouverture dans le repère du mur (x le long du mur, z vers l'intérieur) et
/// pinceau qui y pose ses pièces : (boîte, forme, partie colorable).
struct Hole<'a> {
    x0: f32,
    x1: f32,
    sill: f32,
    top: f32,
    th: f32,
    out: &'a mut dyn FnMut(Option<Transform>, Shape, usize),
}

/// Pièces de l'ouverture `o` qui occupe `x0..x1` et monte jusqu'à `top` dans un mur
/// d'épaisseur `th` ; `open` (0 → 1) l'ouvre.
pub(super) fn opening_parts(
    o: &Opening,
    (x0, x1, top): (f32, f32, f32),
    th: f32,
    open: f32,
    out: &mut dyn FnMut(Option<Transform>, Shape, usize),
) {
    let mut h = Hole { x0, x1, sill: o.sill, top, th, out };
    let fr = o.kind.slots().iter().position(|s| s.name == "Cadre").unwrap_or(0);
    let fd = th + 0.04;
    h.cube(local_box(x0, x0 + F, o.sill, top, fd), fr);
    h.cube(local_box(x1 - F, x1, o.sill, top, fd), fr);
    h.cube(local_box(x0, x1, top - F, top, fd), fr);
    match o.kind {
        OpeningKind::Door => h.door(open),
        OpeningKind::DoubleDoor => h.double_door(open),
        OpeningKind::GlassDoor => h.glass_door(open),
        OpeningKind::FrenchDoor => h.french_door(open),
        OpeningKind::Window | OpeningKind::Bay | OpeningKind::Fixed | OpeningKind::Grid => h.window(o.kind),
        OpeningKind::SlidingWindow => h.sliding_window(open),
        OpeningKind::SlidingDoor => h.sliding_door(open),
        OpeningKind::Shoji => h.shoji(open),
        OpeningKind::PocketDoor => h.pocket_door(open),
        OpeningKind::BarnDoor => h.barn_door(open),
        OpeningKind::GarageDoor => h.garage_door(open),
        OpeningKind::AutoDoor => h.auto_door(open),
    }
}

/// Repère d'un battant de largeur `w` (x de 0 à w) dont la charnière est en `hx` ; `side` vaut
/// 1 s'il s'étend vers +x, -1 vers -x. Il s'ouvre vers +z.
fn hinge(hx: f32, side: f32, w: f32, open: f32) -> Transform {
    let pivot = Transform::from_xyz(hx, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-side * open * 1.6));
    pivot.mul_transform(Transform::from_xyz(if side > 0.0 { 0.0 } else { -w }, 0.0, 0.0))
}

fn at(x: f32, z: f32) -> Transform {
    Transform::from_xyz(x, 0.0, z)
}

impl Hole<'_> {
    /// Boîte `b` placée par `base`.
    fn put(&mut self, base: Transform, b: Option<Transform>, shape: Shape, slot: usize) {
        (self.out)(b.map(|b| base.mul_transform(b)), shape, slot);
    }
    fn cube(&mut self, b: Option<Transform>, slot: usize) {
        self.put(Transform::IDENTITY, b, Shape::Cube, slot);
    }
    /// Bornes intérieures du dormant, le long du mur.
    fn inner(&self) -> (f32, f32) {
        (self.x0 + F, self.x1 - F)
    }

    /// Vantail vitré de largeur `w` (x de 0 à w, y de y0 à y1) placé par `base` : montants et
    /// traverses de largeur `bar`, traverse basse de hauteur `foot`, verre au milieu.
    fn glazed(&mut self, base: Transform, w: f32, (y0, y1): (f32, f32), (bar, foot): (f32, f32), slot: usize) {
        for b in [
            local_box(0.0, bar, y0, y1, 0.05),
            local_box(w - bar, w, y0, y1, 0.05),
            local_box(0.0, w, y1 - bar, y1, 0.05),
            local_box(0.0, w, y0, y0 + foot, 0.05),
        ] {
            self.put(base, b, Shape::Cube, slot);
        }
        self.put(base, local_box(bar, w - bar, y0 + foot, y1 - bar, 0.02), Shape::Glass, 0);
    }

    /// Deux vantaux de largeur `w` sur deux rails : celui de gauche fixe, celui de droite glisse
    /// derrière lui. Renvoie le repère du vantail mobile.
    fn slide_pair(&mut self, open: f32, gap: f32, leaf: &mut dyn FnMut(&mut Self, Transform, f32)) -> Transform {
        let (xa, xb) = self.inner();
        let w = f32::midpoint(xb - xa, 0.05);
        leaf(self, at(xa, -gap), w);
        let moving = at(xb - w - open * (w - 0.05), gap);
        leaf(self, moving, w);
        moving
    }

    fn door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let pw = xb - xa;
        let base = hinge(xa, 1.0, pw, open);
        self.put(base, local_box(0.0, pw, 0.0, self.top - F, 0.05), Shape::Cube, 0);
        self.put(base, local_box(pw - 0.13, pw - 0.06, 0.98, 1.04, 0.14), Shape::Cube, 2);
    }

    fn double_door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let pw = (xb - xa) / 2.0;
        let y1 = self.top - F;
        for (hx, side, knob) in [(xa, 1.0, pw - 0.1), (xb, -1.0, 0.04)] {
            let base = hinge(hx, side, pw, open);
            self.put(base, local_box(0.0, pw, 0.0, y1, 0.05), Shape::Cube, 0);
            // panneaux moulurés
            self.put(base, local_box(0.1, pw - 0.1, 0.15, 0.9, 0.07), Shape::Cube, 0);
            self.put(base, local_box(0.1, pw - 0.1, 1.15, y1 - 0.15, 0.07), Shape::Cube, 0);
            self.put(base, local_box(knob, knob + 0.06, 0.98, 1.04, 0.14), Shape::Cube, 2);
        }
    }

    fn glass_door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let pw = xb - xa;
        let base = hinge(xa, 1.0, pw, open);
        self.glazed(base, pw, (0.0, self.top - F), (0.04, 0.08), 0);
        // barre de tirage verticale
        self.put(base, local_box(pw - 0.13, pw - 0.1, 0.75, 1.35, 0.14), Shape::Cube, 1);
    }

    fn french_door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let pw = (xb - xa) / 2.0;
        for (hx, side) in [(xa, 1.0), (xb, -1.0)] {
            self.glazed(hinge(hx, side, pw, open), pw, (0.0, self.top - F), (F, 0.15), 0);
        }
    }

    /// Traverse basse du dormant et appui extérieur.
    fn sill_frame(&mut self) {
        let (x0, x1, s) = (self.x0, self.x1, self.sill);
        self.cube(local_box(x0, x1, s, s + F, self.th + 0.04), 0);
        if s > 0.0 {
            self.cube(local_box(x0 - 0.05, x1 + 0.05, s - 0.04, s, self.th + 0.12), 1);
        }
    }

    fn window(&mut self, kind: OpeningKind) {
        self.sill_frame();
        let (x0, x1, s, top) = (self.x0, self.x1, self.sill, self.top);
        let m = f32::midpoint(x0, x1);
        if kind == OpeningKind::Bay {
            self.cube(local_box(m - F / 2.0, m + F / 2.0, s, top, self.th + 0.04), 0);
        }
        if kind == OpeningKind::Grid {
            // petits bois : un montant et deux traverses
            let b = 0.015;
            self.cube(local_box(m - b, m + b, s, top, 0.06), 0);
            for k in [1.0, 2.0] {
                let y = s + F + (top - s - 2.0 * F) * k / 3.0;
                self.cube(local_box(x0, x1, y - b, y + b, 0.06), 0);
            }
        }
        self.put(Transform::IDENTITY, local_box(x0 + F, x1 - F, s + F, top - F, 0.02), Shape::Glass, 0);
    }

    fn sliding_window(&mut self, open: f32) {
        self.sill_frame();
        let y = (self.sill + F, self.top - F);
        self.slide_pair(open, 0.03, &mut |h, base, w| h.glazed(base, w, y, (0.05, 0.05), 0));
    }

    fn sliding_door(&mut self, open: f32) {
        // rail bas
        self.cube(local_box(self.x0, self.x1, 0.0, 0.03, self.th + 0.04), 0);
        let y = (0.03, self.top - F);
        let moving = self.slide_pair(open, 0.03, &mut |h, base, w| h.glazed(base, w, y, (0.06, 0.1), 0));
        self.put(moving, local_box(0.07, 0.09, 0.85, 1.35, 0.11), Shape::Cube, 1);
    }

    fn shoji(&mut self, open: f32) {
        let y1 = self.top - F;
        self.slide_pair(open, 0.025, &mut |h, base, w| {
            let (bar, foot) = (0.035, 0.12);
            for b in [
                local_box(0.0, bar, 0.0, y1, 0.035),
                local_box(w - bar, w, 0.0, y1, 0.035),
                local_box(0.0, w, y1 - bar, y1, 0.035),
                local_box(0.0, w, 0.0, foot, 0.035),
            ] {
                h.put(base, b, Shape::Cube, 0);
            }
            // quadrillage (kumiko) de part et d'autre du papier
            for k in 1..3 {
                let x = w * k as f32 / 3.0;
                h.put(base, local_box(x - 0.008, x + 0.008, foot, y1 - bar, 0.02), Shape::Cube, 0);
            }
            for k in 1..6 {
                let y = foot + (y1 - bar - foot) * k as f32 / 6.0;
                h.put(base, local_box(bar, w - bar, y - 0.008, y + 0.008, 0.02), Shape::Cube, 0);
            }
            h.put(base, local_box(bar, w - bar, foot, y1 - bar, 0.004), Shape::Cube, 1);
        });
    }

    fn pocket_door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let pw = xb - xa;
        // le battant rentre dans le mur à droite, son chant reste au ras du dormant
        let base = at(xa + open * pw, 0.0);
        self.put(base, local_box(0.0, pw, 0.0, self.top - F, 0.04), Shape::Cube, 0);
        // poignée cuvette près du chant
        self.put(base, local_box(0.05, 0.08, 0.95, 1.12, 0.05), Shape::Cube, 2);
    }

    fn barn_door(&mut self, open: f32) {
        let w = self.x1 - self.x0 + 0.1;
        let (z, h) = (self.th / 2.0 + 0.05, self.top + 0.05);
        let base = at(self.x0 - 0.05 + open * w, z);
        self.put(base, local_box(0.0, w, 0.01, h, 0.04), Shape::Cube, 0);
        // traverses et écharpe en Z
        self.put(base, local_box(0.0, w, h - 0.16, h - 0.04, 0.06), Shape::Cube, 0);
        self.put(base, local_box(0.0, w, 0.06, 0.18, 0.06), Shape::Cube, 0);
        let (a, b) = (Vec3::new(0.1, 0.18, 0.0), Vec3::new(w - 0.1, h - 0.16, 0.0));
        let d = b - a;
        let brace = Transform::from_translation((a + b) / 2.0)
            .with_rotation(Quat::from_rotation_z(d.y.atan2(d.x)))
            .with_scale(Vec3::new(d.length(), 0.11, 0.06));
        self.put(base, Some(brace), Shape::Cube, 0);
        // rail fixé au mur sur deux largeurs, galets et poignée
        let rail = local_box(self.x0 - 0.1, self.x1 + w + 0.05, h + 0.04, h + 0.09, 0.07);
        self.put(at(0.0, self.th / 2.0 + 0.035), rail, Shape::Cube, 2);
        for x in [0.12, w - 0.16] {
            self.put(base, local_box(x, x + 0.04, h - 0.2, h + 0.08, 0.06), Shape::Cube, 2);
        }
        self.put(base, local_box(0.1, 0.13, 0.85, 1.3, 0.1), Shape::Cube, 2);
    }

    fn garage_door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let hy = self.top - F;
        // tablier suspendu à un axe sous le linteau, qui bascule vers l'intérieur
        let base = Transform::from_xyz(0.0, hy, 0.0).with_rotation(Quat::from_rotation_x(-open * FRAC_PI_2));
        self.put(base, local_box(xa, xb, -hy, 0.0, 0.05), Shape::Cube, 0);
        for k in 1..6 {
            let y = -hy * k as f32 / 6.0;
            self.put(base, local_box(xa + 0.05, xb - 0.05, y - 0.02, y + 0.02, 0.07), Shape::Cube, 0);
        }
        let m = f32::midpoint(xa, xb);
        self.put(base, local_box(m - 0.12, m + 0.12, -hy + 0.25, -hy + 0.3, 0.1), Shape::Cube, 2);
    }

    fn auto_door(&mut self, open: f32) {
        let (xa, xb) = self.inner();
        let w = (xb - xa) / 2.0;
        let y = (0.0, self.top - F);
        self.glazed(at(xa - open * w, 0.0), w, y, (0.04, 0.08), 0);
        self.glazed(at(xa + w + open * w, 0.0), w, y, (0.04, 0.08), 0);
        // caisson du moteur côté intérieur, sur la course des vantaux
        let caisson = local_box(self.x0 - w, self.x1 + w, self.top, self.top + 0.22, 0.16);
        self.put(at(0.0, self.th / 2.0 + 0.08), caisson, Shape::Cube, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ElemRef, House, Part, WHITE, Wall};
    use super::*;
    use bevy::math::Vec2;

    const TH: f32 = 0.2;
    const SLIDING: [OpeningKind; 6] = [
        OpeningKind::SlidingDoor,
        OpeningKind::Shoji,
        OpeningKind::PocketDoor,
        OpeningKind::BarnDoor,
        OpeningKind::AutoDoor,
        OpeningKind::SlidingWindow,
    ];

    /// Mur de 6 m le long de x (repère du mur = repère du monde) avec une ouverture centrée en 3 m.
    fn house(kind: OpeningKind) -> House {
        let wall = Wall {
            a: Vec2::ZERO,
            b: Vec2::new(6.0, 0.0),
            level: 0,
            height: 3.0,
            thick: TH,
            color: WHITE,
            openings: vec![Opening::new(kind, 3.0)],
            texture: 0,
        };
        House { walls: vec![wall], ..House::default() }
    }
    fn parts(h: &House, open: f32) -> Vec<Part> {
        h.parts(None, &|_| open).into_iter().filter(|p| p.owner == ElemRef::Opening(0, 0)).collect()
    }
    /// Pièces qui bougent entre fermé et ouvert : (fermée, ouverte).
    fn moving(kind: OpeningKind) -> Vec<(Transform, Transform)> {
        let h = house(kind);
        parts(&h, 0.0).iter().zip(parts(&h, 1.0)).filter(|(a, b)| a.tf != b.tf).map(|(a, b)| (a.tf, b.tf)).collect()
    }

    #[test]
    fn every_opening_leaves_a_hole_and_uses_its_parts() {
        assert!(OpeningKind::ALL.iter().enumerate().all(|(i, k)| *k as usize == i), "ALL suit l ordre des variantes");
        for k in OpeningKind::ALL {
            let h = house(k);
            let (_, height, sill) = k.default_dims();
            let probe = Vec3::new(3.0, sill + height / 2.0, 0.0);
            for p in h.parts_closed(None).iter().filter(|p| p.owner == ElemRef::Wall(0)) {
                let local = p.tf.compute_affine().inverse().transform_point3(probe);
                assert!(local.abs().max_element() > 0.5, "{k:?} : mur dans le trou");
            }
            let used: Vec<usize> = parts(&h, 0.0).iter().map(|p| p.slot).collect();
            // l'appui n'existe que sous une allège
            for slot in (0..k.slots().len()).filter(|&i| sill > 0.0 || k.slots()[i].name != "Appui") {
                assert!(used.contains(&slot), "{k:?} : partie {slot} jamais dessinée");
            }
            assert!(used.iter().all(|&s| s < k.slots().len()), "{k:?} : partie inconnue");
        }
    }

    #[test]
    fn openable_openings_are_exactly_those_that_move() {
        for k in OpeningKind::ALL {
            let h = house(k);
            assert_eq!(h.openable(ElemRef::Opening(0, 0)), !moving(k).is_empty(), "{k:?}");
        }
        assert!(!house(OpeningKind::Door).openable(ElemRef::Opening(0, 1)), "ouverture absente");
    }

    #[test]
    fn sliding_openings_slide_along_the_wall() {
        for k in SLIDING {
            let moved = moving(k);
            assert!(!moved.is_empty(), "{k:?}");
            for (a, b) in moved {
                assert_eq!(a.rotation, b.rotation, "{k:?} : glisse sans tourner");
                assert!((a.translation - b.translation).with_x(0.0).length() < 1e-5, "{k:?} : reste dans son plan");
                assert!((a.translation.x - b.translation.x).abs() > 0.3, "{k:?} : glisse vraiment");
                // le vantail d'une porte de grange est en applique, les autres dans l'épaisseur
                let inside = a.translation.z.abs() + a.scale.z / 2.0 <= TH / 2.0 + 1e-4;
                assert_eq!(inside, k != OpeningKind::BarnDoor, "{k:?}");
            }
        }
    }

    #[test]
    fn pocket_door_disappears_into_the_wall() {
        let (x1, f) = (3.0 + OpeningKind::PocketDoor.default_dims().0 / 2.0, F);
        for (_, open) in moving(OpeningKind::PocketDoor) {
            assert!(open.translation.x - open.scale.x / 2.0 >= x1 - f - 1e-4, "dans le mur à droite : {open:?}");
        }
    }

    #[test]
    fn swinging_doors_pivot_and_garage_door_ends_up_flat_inside() {
        for k in [OpeningKind::Door, OpeningKind::DoubleDoor, OpeningKind::GlassDoor, OpeningKind::GarageDoor] {
            assert!(moving(k).iter().all(|(a, b)| a.rotation != b.rotation), "{k:?} pivote");
        }
        let (_, height, _) = OpeningKind::GarageDoor.default_dims();
        let panel = moving(OpeningKind::GarageDoor)
            .into_iter()
            .max_by(|a, b| a.0.scale.y.total_cmp(&b.0.scale.y))
            .expect("tablier")
            .1;
        let down = panel.rotation * Vec3::NEG_Y;
        assert!(down.y.abs() < 1e-3 && down.z > 0.99, "horizontal, vers l intérieur : {down:?}");
        assert!(panel.translation.z > 0.5 && panel.translation.y > height - 0.15, "rentré sous le linteau : {panel:?}");
    }
}
