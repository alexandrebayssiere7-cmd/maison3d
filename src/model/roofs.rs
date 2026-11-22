//! Formes de toit et leurs triangles.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Forme de toit. Le faîtage suit toujours le grand côté de l'emprise.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub enum RoofKind {
    /// Deux pans, pignons sur les petits côtés : la maison classique.
    Gable,
    /// Quatre pans, sans pignon.
    Hip,
    /// Toit-terrasse plat de 30 cm (`height` ignorée).
    Flat,
    /// Un seul pan incliné (garage, extension).
    Shed,
    /// Quatre pans jusqu'à un sommet unique, pour une emprise presque carrée.
    Pyramid,
    /// Mansarde : pans brisés, raides en bas, doux en haut.
    Mansard,
    /// Comble de grange (gambrel, Mansart à l'américaine) : deux pans brisés, raides en bas,
    /// doux en haut, pignons sur les petits côtés.
    Gambrel,
    /// Saltbox : deux pans asymétriques, faîtage décalé vers le côté min (nord ou ouest).
    Saltbox,
    /// Papillon : deux pans en V qui descendent vers une noue centrale (1,2 m de haut par défaut).
    Butterfly,
    /// Sheds : dents de scie parallèles au faîtage, face verticale tournée vers le côté min
    /// (nord ou ouest) ; toit d'usine ou de supermarché (1,5 m de haut par défaut).
    Sawtooth,
    /// Voûte : demi-cylindre le long du grand côté.
    Barrel,
    /// Dôme : demi-ellipsoïde inscrit dans l'emprise, coins plats.
    Dome,
    /// Toit asiatique : quatre pans concaves, doux au bord et raides au faîtage, angles relevés.
    Pagoda,
    /// Conique : cône inscrit dans l'emprise, pour une tourelle (emprise carrée, 3 à 5 m de haut).
    Cone,
}

impl RoofKind {
    pub const ALL: [RoofKind; 14] = [
        Self::Gable,
        Self::Hip,
        Self::Flat,
        Self::Shed,
        Self::Pyramid,
        Self::Mansard,
        Self::Gambrel,
        Self::Saltbox,
        Self::Butterfly,
        Self::Sawtooth,
        Self::Barrel,
        Self::Dome,
        Self::Pagoda,
        Self::Cone,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Gable => "Deux pans",
            Self::Hip => "Quatre pans",
            Self::Flat => "Toit plat",
            Self::Shed => "Monopente",
            Self::Pyramid => "Pyramide",
            Self::Mansard => "Mansarde",
            Self::Gambrel => "Comble de grange",
            Self::Saltbox => "Saltbox",
            Self::Butterfly => "Papillon",
            Self::Sawtooth => "Sheds",
            Self::Barrel => "Voûte",
            Self::Dome => "Dôme",
            Self::Pagoda => "Toit asiatique",
            Self::Cone => "Conique",
        }
    }
}

type Tri = [Vec3; 3];

fn rect(a: f32, b: f32, y: f32) -> [Vec3; 4] {
    [Vec3::new(-a, y, -b), Vec3::new(a, y, -b), Vec3::new(a, y, b), Vec3::new(-a, y, b)]
}

/// Triangles d'un toit en pente, faîtage le long de X. `size` = (longueur, hauteur, largeur).
pub fn roof_tris(kind: RoofKind, size: Vec3) -> Vec<Tri> {
    let (a, h, b) = (size.x / 2.0, size.y, size.z / 2.0);
    let c = rect(a, b, 0.0);
    let mut t = vec![[c[0], c[2], c[1]], [c[0], c[3], c[2]]]; // dessous
    match kind {
        RoofKind::Gable => ridge(&mut t, c, h, 0.0),
        RoofKind::Hip => ridge(&mut t, c, h, b.min(a)),
        RoofKind::Pyramid => ridge(&mut t, c, h, a),
        RoofKind::Mansard => mansard(&mut t, c, h),
        RoofKind::Shed => extrude(&mut t, a, &[(-b, 0.0), (b, h), (b, 0.0)]),
        RoofKind::Flat => {}
        RoofKind::Gambrel => {
            let (zb, yb) = (0.6 * b, 0.65 * h);
            extrude(&mut t, a, &[(-b, 0.0), (-zb, yb), (0.0, h), (zb, yb), (b, 0.0)]);
        }
        RoofKind::Saltbox => extrude(&mut t, a, &[(-b, 0.0), (-b / 3.0, h), (b, 0.0)]),
        RoofKind::Butterfly => extrude(&mut t, a, &[(-b, 0.0), (-b, h), (0.0, 0.4 * h), (b, h), (b, 0.0)]),
        RoofKind::Sawtooth => extrude(&mut t, a, &sawtooth(b, h)),
        RoofKind::Barrel => {
            let arc: Vec<_> =
                (0..=16).map(|k| PI * k as f32 / 16.0).map(|f| (-b * f.cos(), h * f.sin().max(0.0))).collect();
            extrude(&mut t, a, &arc);
        }
        RoofKind::Dome => {
            let d = plinth(&mut t, a, b, h);
            let arc: Vec<_> =
                (0..=8).map(|k| FRAC_PI_2 * k as f32 / 8.0).map(|f| (f.cos(), d + (h - d) * f.sin())).collect();
            lathe(&mut t, (a, b), &arc);
        }
        RoofKind::Cone => {
            let d = plinth(&mut t, a, b, h);
            lathe(&mut t, (a, b), &[(1.0, d), (0.0, h)]);
        }
        RoofKind::Pagoda => pagoda(&mut t, (a, b), h),
    }
    // les faîtages réduits à un point laissent des triangles plats
    t.retain(|[p, q, r]| (*q - *p).cross(*r - *p).length_squared() > 1e-8);
    t
}

/// Pans montant de la base `c` jusqu'à un faîtage le long de X, raccourci de `inset` à chaque bout.
fn ridge(t: &mut Vec<Tri>, c: [Vec3; 4], top: f32, inset: f32) {
    let a = c[1].x;
    let r = [Vec3::new(-a + inset, top, 0.0), Vec3::new(a - inset, top, 0.0)];
    t.extend([
        [c[0], c[1], r[1]],
        [c[0], r[1], r[0]], // pan avant
        [c[2], c[3], r[0]],
        [c[2], r[0], r[1]], // pan arrière
        [c[3], c[0], r[0]],
        [c[1], c[2], r[1]], // pignons / croupes
    ]);
}

/// Brisis raide jusqu'aux deux tiers, puis terrasson à faible pente.
fn mansard(t: &mut Vec<Tri>, c: [Vec3; 4], h: f32) {
    let (a, b) = (c[1].x, c[2].z);
    let k = 0.2 * a.min(b);
    let top = rect(a - k, b - k, 0.65 * h);
    for i in 0..4 {
        let j = (i + 1) % 4;
        t.extend([[c[i], c[j], top[j]], [c[i], top[j], top[i]]]);
    }
    ridge(t, top, h, (b - k).min(a - k));
}

/// Profil `(z, y)` de travers, de z = -b à z = b en finissant à y = 0, tiré le long de X
/// entre -a et a ; chaque bout est fermé par un pignon qui descend jusqu'à y = 0.
fn extrude(t: &mut Vec<Tri>, a: f32, profile: &[(f32, f32)]) {
    let p = |x: f32, (z, y): (f32, f32)| Vec3::new(x, y, z);
    for w in profile.windows(2) {
        let (s, e) = (w[0], w[1]);
        t.extend([[p(-a, s), p(a, s), p(a, e)], [p(-a, s), p(a, e), p(-a, e)]]);
        for x in [-a, a] {
            let (s0, e0) = (p(x, (s.0, 0.0)), p(x, (e.0, 0.0)));
            t.extend([[s0, e0, p(x, e)], [s0, p(x, e), p(x, s)]]);
        }
    }
}

/// Dents de scie d'environ 2,5 m : face verticale côté -z, pan qui redescend vers +z, sur un
/// socle de 30 % de la hauteur.
fn sawtooth(b: f32, h: f32) -> Vec<(f32, f32)> {
    let n = ((2.0 * b / 2.5).ceil() as usize).clamp(2, 12);
    let (v, w) = (0.3 * h, 2.0 * b / n as f32);
    let mut p = vec![(-b, 0.0), (-b, h)];
    for i in 1..n {
        let z = -b + w * i as f32;
        p.extend([(z, v), (z, h)]);
    }
    p.extend([(b, v), (b, 0.0)]);
    p
}

/// Dalle mince sous un toit rond : ses coins restent plats sans se confondre avec le haut des
/// murs. Renvoie son épaisseur.
fn plinth(t: &mut Vec<Tri>, a: f32, b: f32, h: f32) -> f32 {
    let d = (0.1 * h).min(0.12);
    extrude(t, a, &[(-b, 0.0), (-b, d), (b, d), (b, 0.0)]);
    d
}

/// Profil `(rayon relatif, y)` de (1, 0) jusqu'à l'axe, tourné autour de Y sur une ellipse de
/// demi-axes `(a, b)`.
fn lathe(t: &mut Vec<Tri>, (a, b): (f32, f32), profile: &[(f32, f32)]) {
    const N: usize = 32;
    let p = |(r, y): (f32, f32), i: usize| {
        let f = TAU * i as f32 / N as f32;
        Vec3::new(a * r * f.cos(), y, b * r * f.sin())
    };
    for w in profile.windows(2) {
        for i in 0..N {
            let q = [p(w[0], i), p(w[0], i + 1), p(w[1], i + 1), p(w[1], i)];
            t.extend([[q[0], q[1], q[2]], [q[0], q[2], q[3]]]);
        }
    }
}

/// Toit en croupe dont les pans se creusent (y = h·s² de l'égout au faîtage) et dont les angles
/// se relèvent ; un bandeau ferme l'égout relevé jusqu'à y = 0.
fn pagoda(t: &mut Vec<Tri>, (a, b): (f32, f32), h: f32) {
    const RINGS: usize = 6;
    const SIDE: usize = 6;
    let inset = a.min(b);
    // anneau k : rectangle rétréci parcouru en 4 × SIDE points depuis le coin (-a, -b)
    let ring = |k: usize| -> Vec<Vec3> {
        let s = k as f32 / RINGS as f32;
        let c = rect(a - s * inset, b - s * inset, h * s * s);
        (0..4 * SIDE)
            .map(|i| {
                let u = (i % SIDE) as f32 / SIDE as f32;
                let p = c[i / SIDE].lerp(c[(i / SIDE + 1) % 4], u);
                // 1 aux angles, 0 au milieu des côtés, et de moins en moins vers le faîtage
                let corner = (2.0 * u - 1.0).powi(2) * (1.0 - s).powi(2);
                p + Vec3::Y * 0.2 * h * corner
            })
            .collect()
    };
    let n = 4 * SIDE;
    for k in 0..RINGS {
        let (lo, hi) = (ring(k), ring(k + 1));
        for i in 0..n {
            let j = (i + 1) % n;
            t.extend([[lo[i], lo[j], hi[j]], [lo[i], hi[j], hi[i]]]);
        }
    }
    let eave = ring(0);
    for i in 0..n {
        let (p, q) = (eave[i], eave[(i + 1) % n]);
        t.extend([[p.with_y(0.0), q.with_y(0.0), q], [p.with_y(0.0), q, p]]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: Vec3 = Vec3::new(10.0, 3.0, 6.0);
    fn top_at(k: RoofKind, keep: impl Fn(Vec3) -> bool) -> f32 {
        roof_tris(k, SIZE).iter().flatten().copied().filter(|v| keep(*v)).map(|v| v.y).fold(f32::MIN, f32::max)
    }

    #[test]
    fn new_roofs_have_no_flat_triangles() {
        for k in RoofKind::ALL.into_iter().filter(|k| *k != RoofKind::Flat) {
            let tris = roof_tris(k, SIZE);
            assert!(tris.len() >= 6, "{k:?}");
            let top = tris.iter().flatten().map(|v| v.y).fold(0.0, f32::max);
            assert!((top - 3.0).abs() < 1e-4, "{k:?} culmine à {top}");
            let inside = |v: &Vec3| v.abs().cmple(Vec3::new(5.0, 3.0, 3.0) + 1e-4).all() && v.y >= 0.0;
            assert!(tris.iter().flatten().all(inside), "{k:?} déborde de son emprise");
        }
    }

    #[test]
    fn roof_shapes_have_their_silhouette() {
        let near = |x: f32, y: f32| (x - y).abs() < 1e-3;
        // papillon : bords hauts, noue basse au milieu
        assert!(near(top_at(RoofKind::Butterfly, |v| near(v.z, 3.0)), 3.0));
        assert!(top_at(RoofKind::Butterfly, |v| near(v.z, 0.0)) < 1.5);
        // saltbox : faîtage décalé vers -z
        assert!(near(top_at(RoofKind::Saltbox, |v| near(v.z, -1.0)), 3.0));
        // sheds : plusieurs faces verticales à pleine hauteur
        let mut teeth: Vec<i32> = roof_tris(RoofKind::Sawtooth, SIZE)
            .iter()
            .flatten()
            .filter(|v| near(v.y, 3.0))
            .map(|v| (v.z * 100.0).round() as i32)
            .collect();
        teeth.sort_unstable();
        teeth.dedup();
        assert!(teeth.len() >= 3, "{teeth:?}");
        // toit asiatique : angles relevés, pans creusés sous la droite égout-faîtage
        assert!(top_at(RoofKind::Pagoda, |v| near(v.x, 5.0) && near(v.z, 3.0)) > 0.3);
        assert!(top_at(RoofKind::Pagoda, |v| near(v.x, 0.0) && near(v.z, -1.5)) < 1.0);
        // voûte, dôme et cône : sommet au centre, rien aux angles du haut
        for k in [RoofKind::Barrel, RoofKind::Dome, RoofKind::Cone] {
            assert!(near(top_at(k, |v| near(v.z, 0.0)), 3.0), "{k:?}");
        }
        assert!(top_at(RoofKind::Dome, |v| near(v.x, 5.0)) < 0.15, "le dôme ne monte pas au bord");
    }

    #[test]
    fn low_roofs_get_a_lower_default_height() {
        use crate::command::Command;
        let mut h = super::super::House::default();
        for (i, (kind, height)) in
            [(RoofKind::Butterfly, 1.2), (RoofKind::Sawtooth, 1.5), (RoofKind::Dome, 2.5)].into_iter().enumerate()
        {
            let min = bevy::math::Vec2::new(10.0 * i as f32, 0.0);
            h.apply(Command::PlaceRoof { min, max: min + 6.0, level: 1, kind, texture: None, tag: None }).unwrap();
            assert_eq!(h.roofs[i].height, height, "{kind:?}");
        }
    }
}
