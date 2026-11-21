//! Relief du terrain : grille d'altitudes au mètre, pinceaux de sculpture et génération aléatoire.

use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Cases de 1 m par côté, centrées sur l'origine ; au-delà, le sol reste plat à 0.
pub const N: usize = 160;
pub const SIDE: usize = N + 1;
/// Demi-côté du terrain en mètres : il couvre x et z de -HALF à HALF.
pub const HALF: f32 = N as f32 / 2.0;

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Position le long de la route `a`→`b` (0 en `a`, 1 en `b`) des sommets les plus proches de
/// ses deux bouts.
fn anchor_params(a: Vec2, b: Vec2) -> (f32, f32) {
    let along = |p: Vec2| (p - a).dot(b - a) / a.distance_squared(b);
    (along(Terrain::anchor(a)), along(Terrain::anchor(b)))
}

/// Altitude d'une route à la position `k` : `ya` et `yb` exactement aux sommets de ses bouts
/// (en `ka` et `kb`), en ligne droite entre eux et jusqu'aux bouts, plate au-delà. Un tronçon
/// plus court qu'une maille a ses deux sommets confondus : il reste plat.
fn road_y(k: f32, (ka, kb): (f32, f32), (ya, yb): (f32, f32)) -> f32 {
    if kb - ka <= 1e-6 {
        return ya;
    }
    let s = (k.clamp(ka.min(0.0), kb.max(1.0)) - ka) / (kb - ka);
    // et non ya + (yb - ya) * s : exact aux deux sommets, que `House::regrade` relit
    ya * (1.0 - s) + yb * s
}

/// `p` est dans l'un des rectangles `skip` (le sol d'un bâtiment).
fn built(skip: &[(Vec2, Vec2)], p: Vec2) -> bool {
    skip.iter().any(|&(lo, hi)| p.cmpge(lo).all() && p.cmple(hi).all())
}

/// Pente maximale (m par m) du talus que `keep_flat` laisse autour d'un bâtiment : 45°.
const FLAT_SLOPE: f32 = 1.0;

/// Distance de `p` au rectangle `lo`–`hi` moins la bande plate de 3 m autour (négative dedans).
fn past_margin(p: Vec2, lo: Vec2, hi: Vec2) -> f32 {
    p.distance(p.clamp(lo, hi)) - 3.0
}

/// 0 dans le rectangle et jusqu'à 3 m autour, puis remonte vers 1 sur 15 m.
fn flat_weight(p: Vec2, lo: Vec2, hi: Vec2) -> f32 {
    smooth(past_margin(p, lo, hi) / 15.0)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Brush {
    Raise,
    Lower,
    Flatten,
    Smooth,
    Noise,
}

impl Brush {
    pub const ALL: [Brush; 5] = [Self::Raise, Self::Lower, Self::Flatten, Self::Smooth, Self::Noise];
    pub fn name(self) -> &'static str {
        match self {
            Self::Raise => "Monter",
            Self::Lower => "Creuser",
            Self::Flatten => "Aplanir",
            Self::Smooth => "Lisser",
            Self::Noise => "Aléatoire",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Terrain {
    /// Altitude des sommets, ligne par ligne selon z ; vide = terrain plat.
    pub h: Vec<f32>,
}

impl Terrain {
    pub fn vertex(i: usize, j: usize) -> Vec2 {
        Vec2::new(i as f32 - HALF, j as f32 - HALF)
    }

    pub fn at(&self, i: usize, j: usize) -> f32 {
        self.h.get(j * SIDE + i).copied().unwrap_or(0.0)
    }

    /// Altitude en `p`, interpolée sur les mêmes triangles que le maillage (diagonale 00–11).
    pub fn height(&self, p: Vec2) -> f32 {
        let q = p + HALF;
        if self.h.is_empty() || q.min_element() < 0.0 || q.max_element() >= N as f32 {
            return 0.0;
        }
        let (i, j) = (q.x as usize, q.y as usize);
        let (fx, fz) = (q.x.fract(), q.y.fract());
        let (h00, h10, h01, h11) = (self.at(i, j), self.at(i + 1, j), self.at(i, j + 1), self.at(i + 1, j + 1));
        if fx >= fz { h00 + fx * (h10 - h00) + fz * (h11 - h10) } else { h00 + fz * (h01 - h00) + fx * (h11 - h01) }
    }

    /// Remplace l'altitude des sommets du disque par `f(ancien terrain, i, j, poids)`.
    /// Le bord de la grille n'est jamais modifié : il se raccorde au sol plat qui l'entoure.
    fn each(&mut self, c: Vec2, r: f32, mut f: impl FnMut(&Terrain, usize, usize, f32) -> f32) {
        if self.h.is_empty() {
            self.h = vec![0.0; SIDE * SIDE];
        }
        let lo = |v: f32| (v - r + HALF).floor().max(1.0) as usize;
        let hi = |v: f32| (v + r + HALF).ceil().min((N - 1) as f32) as usize;
        let old = self.clone();
        for j in lo(c.y)..=hi(c.y) {
            for i in lo(c.x)..=hi(c.x) {
                let d = Self::vertex(i, j).distance(c);
                if d < r {
                    self.h[j * SIDE + i] = f(&old, i, j, (1.0 - (d / r).powi(2)).powi(2));
                }
            }
        }
    }

    /// Un coup de pinceau de rayon `r` ; `amount` en mètres (déjà multiplié par la durée).
    pub fn brush(&mut self, mode: Brush, c: Vec2, r: f32, amount: f32, target: f32, seed: u32) {
        self.each(c, r, |t, i, j, w| {
            let h = t.at(i, j);
            let k = (amount * w).min(1.0);
            match mode {
                Brush::Raise => h + amount * w,
                Brush::Lower => h - amount * w,
                Brush::Flatten => h + (target - h) * k,
                Brush::Smooth => {
                    h + ((t.at(i - 1, j) + t.at(i + 1, j) + t.at(i, j - 1) + t.at(i, j + 1)) / 4.0 - h) * k
                }
                Brush::Noise => h + amount * w * (2.0 * fbm(Self::vertex(i, j) / 5.0, seed) - 1.0) * 2.0,
            }
        });
    }

    /// Met à l'altitude `v` tous les sommets du rectangle (fond d'une piscine).
    pub fn flatten_rect(&mut self, min: Vec2, max: Vec2, v: f32) {
        if self.h.is_empty() && v == 0.0 {
            return;
        }
        let c = (min + max) / 2.0;
        self.each(c, (max - min).length() / 2.0 + 1.0, |t, i, j, _| {
            let p = Self::vertex(i, j);
            if p.cmpge(min).all() && p.cmple(max).all() { v } else { t.at(i, j) }
        });
    }

    /// Sommet de la grille le plus proche de `p` (dans la grille).
    pub fn anchor(p: Vec2) -> Vec2 {
        (p + HALF).round().clamp(Vec2::ZERO, Vec2::splat(N as f32)) - HALF
    }

    /// Altitude au sommet le plus proche de `p` : c'est là que `House::regrade` lit le bout d'une
    /// route ou le fond d'une piscine. Un seul sommet a un seul dernier écrivain, qui y relit
    /// exactement ce qu'il a écrit ; une moyenne de sommets voisins mêlerait plusieurs éléments.
    pub fn at_anchor(&self, p: Vec2) -> f32 {
        let q = Self::anchor(p) + HALF;
        self.at(q.x as usize, q.y as usize)
    }

    /// Altitude de la chaussée aux deux bouts `a` et `b` d'une route terrassée : celle de la
    /// droite que suit `grade`, pour dessiner la route exactement sur le terrain.
    pub fn road_ends(&self, a: Vec2, b: Vec2) -> (f32, f32) {
        let (ks, ys) = (anchor_params(a, b), (self.at_anchor(a), self.at_anchor(b)));
        (road_y(0.0, ks, ys), road_y(1.0, ks, ys))
    }

    /// Terrassement d'une route sur sa largeur plus un accotement de 1 m : droite qui passe par
    /// `ya` au sommet le plus proche de `a` et `yb` à celui de `b` (voir `road_y`). Les sommets
    /// dans un rectangle de `skip` (le sol d'un bâtiment) ne sont pas touchés.
    pub fn grade(&mut self, (a, ya): (Vec2, f32), (b, yb): (Vec2, f32), width: f32, skip: &[(Vec2, Vec2)]) {
        if self.h.is_empty() || a == b {
            return;
        }
        let len2 = a.distance_squared(b);
        let ks = anchor_params(a, b);
        // au moins 1 m : les sommets des bouts, à 0,71 m au plus, sont toujours terrassés
        let reach = width / 2.0 + 1.0;
        self.each((a + b) / 2.0, a.distance(b) / 2.0 + reach + 1.0, |t, i, j, _| {
            let p = Self::vertex(i, j);
            let k = (p - a).dot(b - a) / len2;
            let near = p.distance(a.lerp(b, k.clamp(0.0, 1.0))) <= reach;
            if near && !built(skip, p) { road_y(k, ks, (ya, yb)) } else { t.at(i, j) }
        });
    }

    /// Réécrit les deux sommets d'où `House::regrade` lit les bouts de la route, après les
    /// autres routes : chacune y retrouve sa propre altitude, pas l'accotement d'une voisine.
    /// Le bord de la grille et le sol des bâtiments ne sont pas touchés, comme dans `grade`.
    pub fn pin_road_ends(&mut self, (a, ya): (Vec2, f32), (b, yb): (Vec2, f32), skip: &[(Vec2, Vec2)]) {
        if self.h.is_empty() || a == b {
            return;
        }
        let ks = anchor_params(a, b);
        for (p, k) in [(Self::anchor(a), ks.0), (Self::anchor(b), ks.1)] {
            let q = p + HALF;
            let inner = q.cmpgt(Vec2::ZERO).all() && q.cmplt(Vec2::splat(N as f32)).all();
            if inner && !built(skip, p) {
                self.h[q.y as usize * SIDE + q.x as usize] = road_y(k, ks, (ya, yb));
            }
        }
    }

    /// Relief aléatoire : collines douces ou montagnes à crêtes, de taille `scale` (m).
    /// La zone `keep` (la maison) reste plate, et le terrain retombe à 0 sur les bords.
    pub fn generate(&mut self, seed: u32, amp: f32, scale: f32, mountains: bool, keep: Option<(Vec2, Vec2)>) {
        self.h = (0..SIDE * SIDE)
            .map(|k| {
                let p = Self::vertex(k % SIDE, k / SIDE);
                let n = fbm(p / scale, seed);
                let shape = if mountains { (1.0 - (2.0 * n - 1.0).abs()).powi(3) * 2.0 } else { 2.0 * n - 0.8 };
                let edge = smooth((HALF - p.abs().max_element()) / 15.0);
                let near = keep.map_or(1.0, |(lo, hi)| flat_weight(p, lo, hi));
                amp * shape * edge * near
            })
            .collect();
    }

    /// Ramène en douceur le relief à 0 autour du rectangle `lo`–`hi` (un bâtiment posé après le relief).
    /// Borne la hauteur par un talus en cône plutôt que la multiplier : 0 jusqu'à 3 m du
    /// rectangle, puis `FLAT_SLOPE` m de plus par mètre. Aplanir dix fois la même zone revient à
    /// l'aplanir une fois, dans n'importe quel ordre, et la pente reste au plus celle du relief
    /// ou du talus. Le relief plus bas que le talus est intact : une colline de 20 m est touchée
    /// jusqu'à 23 m du bâtiment.
    pub fn keep_flat(&mut self, lo: Vec2, hi: Vec2) {
        for (k, y) in self.h.iter_mut().enumerate() {
            let cap = FLAT_SLOPE * past_margin(Self::vertex(k % SIDE, k / SIDE), lo, hi).max(0.0);
            *y = y.clamp(-cap, cap);
        }
    }

    /// Premier point du terrain touché par un rayon.
    pub fn ray(&self, o: Vec3, d: Vec3) -> Option<Vec3> {
        if self.h.is_empty() {
            let t = -o.y / d.y;
            return (d.y.abs() > 1e-6 && t > 0.0).then(|| o + d * t);
        }
        let above = |t: f32| {
            let p = o + d * t;
            p.y - self.height(Vec2::new(p.x, p.z))
        };
        if above(0.0) < 0.0 {
            return None;
        }
        let step = 0.5;
        let mut t = 0.0;
        while t < 2000.0 {
            if above(t + step) <= 0.0 {
                let (mut a, mut b) = (t, t + step);
                for _ in 0..12 {
                    let m = f32::midpoint(a, b);
                    if above(m) > 0.0 { a = m } else { b = m }
                }
                return Some(o + d * b);
            }
            t += step;
        }
        None
    }
}

fn hash(i: i32, j: i32, s: u32) -> f32 {
    let mut h =
        (i as u32).wrapping_mul(374_761_393) ^ (j as u32).wrapping_mul(668_265_263) ^ s.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

fn vnoise(p: Vec2, s: u32) -> f32 {
    let (i, j) = (p.x.floor() as i32, p.y.floor() as i32);
    let f = p - p.floor();
    let u = f * f * (3.0 - 2.0 * f);
    let (a, b, c, d) = (hash(i, j, s), hash(i + 1, j, s), hash(i, j + 1, s), hash(i + 1, j + 1, s));
    let top = a + (b - a) * u.x;
    let bottom = c + (d - c) * u.x;
    top + (bottom - top) * u.y
}

/// Bruit fractal dans [0, 1].
pub fn fbm(p: Vec2, s: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut q) = (0.0, 1.0, 0.0, p);
    for o in 0..5 {
        sum += amp * vnoise(q, s.wrapping_add(o * 101));
        norm += amp;
        amp *= 0.5;
        q *= 2.0;
    }
    sum / norm
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Terrasse une route aux altitudes lues à ses bouts, comme `House::regrade`.
    fn road(t: &mut Terrain, a: Vec2, b: Vec2, width: f32, skip: &[(Vec2, Vec2)]) {
        let ends = ((a, t.at_anchor(a)), (b, t.at_anchor(b)));
        t.grade(ends.0, ends.1, width, skip);
    }

    #[test]
    fn brush_raises_center_and_keeps_border_flat() {
        let mut t = Terrain::default();
        assert_eq!(t.height(Vec2::new(3.0, 4.0)), 0.0);
        t.brush(Brush::Raise, Vec2::ZERO, 5.0, 2.0, 0.0, 0);
        assert!((t.height(Vec2::ZERO) - 2.0).abs() < 1e-4);
        assert!(t.height(Vec2::new(2.5, 0.0)) > 0.5 && t.height(Vec2::new(2.5, 0.0)) < 2.0);
        assert_eq!(t.height(Vec2::new(6.0, 0.0)), 0.0);
        t.brush(Brush::Raise, Vec2::splat(HALF), 5.0, 2.0, 0.0, 0);
        assert_eq!(t.at(N, N), 0.0, "le bord ne bouge pas");
        t.brush(Brush::Flatten, Vec2::ZERO, 5.0, 100.0, 0.5, 0);
        assert!((t.height(Vec2::ZERO) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn ray_hits_hill_before_plane() {
        let mut t = Terrain::default();
        let flat = t.ray(Vec3::new(0.0, 10.0, -10.0), Vec3::new(0.0, -1.0, 1.0).normalize()).unwrap();
        assert!(flat.distance(Vec3::ZERO) < 1e-3);
        t.brush(Brush::Raise, Vec2::ZERO, 6.0, 4.0, 0.0, 0);
        let hit = t.ray(Vec3::new(0.0, 10.0, -10.0), Vec3::new(0.0, -1.0, 1.0).normalize()).unwrap();
        assert!(hit.z < -0.5 && (hit.y - t.height(Vec2::new(hit.x, hit.z))).abs() < 0.01, "{hit}");
    }

    #[test]
    fn generation_keeps_house_flat_and_grading_follows_road() {
        let mut t = Terrain::default();
        t.generate(3, 10.0, 30.0, true, Some((Vec2::ZERO, Vec2::splat(10.0))));
        assert_eq!(t.height(Vec2::new(5.0, 5.0)), 0.0);
        assert!(t.h.iter().any(|h| h.abs() > 1.0), "du relief ailleurs");
        let (a, b) = (Vec2::new(-60.0, -40.0), Vec2::new(-20.0, -40.0));
        road(&mut t, a, b, 4.0, &[]);
        let mid = t.height(Vec2::new(-40.0, -40.0));
        assert!((mid - f32::midpoint(t.height(a), t.height(b))).abs() < 0.05);
    }

    #[test]
    fn keep_flat_is_idempotent_and_leaves_far_relief() {
        let mut t = Terrain::default();
        t.generate(3, 20.0, 30.0, false, None);
        let raw = t.clone();
        let (lo, hi) = (Vec2::new(-5.0, -5.0), Vec2::new(5.0, 5.0));
        t.keep_flat(lo, hi);
        assert_eq!(t.height(Vec2::ZERO), 0.0);
        assert_eq!(t.height(Vec2::new(7.0, 0.0)), 0.0, "plat jusqu'à 3 m du rectangle");
        let once = t.clone();
        for _ in 0..10 {
            t.keep_flat(lo, hi);
        }
        assert!(t == once, "aplanir dix fois = aplanir une fois");
        // loin du rectangle, le talus dépasse la colline : relief d'origine intact
        let far = Vec2::new(60.0, 0.0);
        assert_eq!(t.height(far), raw.height(far));
        // deux zones dans un ordre ou dans l'autre : même relief
        let other = (Vec2::new(10.0, -5.0), Vec2::new(20.0, 5.0));
        let mut ab = raw.clone();
        ab.keep_flat(lo, hi);
        ab.keep_flat(other.0, other.1);
        let mut ba = raw;
        ba.keep_flat(other.0, other.1);
        ba.keep_flat(lo, hi);
        assert!(ab == ba, "l'ordre ne compte pas");
    }

    #[test]
    fn keep_flat_never_steepens_the_relief() {
        // colline haute : un plafond trop raide creuserait un cratère autour de la maison
        let mut t = Terrain::default();
        t.generate(3, 40.0, 30.0, false, None);
        let raw = t.clone();
        t.keep_flat(Vec2::new(-5.0, -5.0), Vec2::new(5.0, 5.0));
        let mut worst = 0.0_f32;
        for j in 0..SIDE {
            for i in 0..N {
                let (before, after) = ((raw.at(i + 1, j) - raw.at(i, j)).abs(), (t.at(i + 1, j) - t.at(i, j)).abs());
                worst = worst.max(after - before.max(1.0));
            }
        }
        assert!(worst < 1e-4, "pente au-delà de 1 m par m et du relief d'origine : {worst}");
    }

    #[test]
    fn grading_again_changes_nothing_even_off_grid() {
        let mut t = Terrain::default();
        t.generate(3, 20.0, 30.0, false, None);
        let (a, b) = (Vec2::new(-40.3, -20.6), Vec2::new(-20.2, -19.7));
        road(&mut t, a, b, 4.0, &[]);
        let once = t.clone();
        for _ in 0..40 {
            road(&mut t, a, b, 4.0, &[]);
        }
        // avant correctif, les bouts glissaient de 30 cm l'un vers l'autre
        assert!(t == once, "le terrain sous la route bouge");
    }

    #[test]
    fn grading_leaves_buildings_and_never_extrapolates() {
        let mut t = Terrain::default();
        t.generate(3, 20.0, 30.0, false, None);
        let raw = t.clone();
        let (a, b) = (Vec2::new(-37.0, -5.0), Vec2::new(-37.0, 17.0));
        let house = (Vec2::new(-40.0, 17.0), Vec2::new(-34.0, 23.0));
        road(&mut t, a, b, 4.0, &[house]);
        assert_eq!(t.at(43, 99), raw.at(43, 99), "sommet de la maison intact");
        // sans maison : droite d'un bout à l'autre, plate au-delà, jamais plus bas ni plus haut
        let mut alone = raw.clone();
        road(&mut alone, a, b, 4.0, &[]);
        let (ya, yb) = alone.road_ends(a, b);
        assert_eq!((alone.height(Vec2::new(-37.0, 19.0)), alone.height(Vec2::new(-37.0, 20.0))), (yb, yb));
        assert_eq!((alone.height(a), alone.height(b), yb), (ya, yb, raw.at_anchor(b)), "chaussée sur le terrain");
        let (lo, hi) = (ya.min(yb), ya.max(yb));
        let graded = alone.h.iter().zip(&raw.h).filter(|(x, y)| (*x - *y).abs() > 0.0);
        assert!(graded.clone().count() > 50 && graded.into_iter().all(|(&y, _)| (lo..=hi).contains(&y)));
        // tronçon plus court qu'une maille : ses deux sommets se confondent, il reste plat
        let mut short = raw;
        let (c, d) = (Vec2::new(10.2, 10.0), Vec2::new(10.4, 10.1));
        road(&mut short, c, d, 2.0, &[]);
        assert_eq!(short.height(c), short.height(Vec2::new(11.0, 11.0)));
        assert_eq!(Terrain::anchor(Vec2::new(500.0, -10.4)), Vec2::new(HALF, -10.0), "borné à la grille");
    }

    #[test]
    fn pinning_road_ends_writes_their_two_vertices_only() {
        let mut t = Terrain::default();
        t.generate(3, 20.0, 30.0, false, None);
        let raw = t.clone();
        let (a, b) = (Vec2::new(-10.3, 4.2), Vec2::new(9.6, 5.1));
        t.pin_road_ends((a, 1.0), (b, 2.0), &[]);
        assert_eq!((t.at_anchor(a), t.at_anchor(b)), (1.0, 2.0));
        let changed = t.h.iter().zip(&raw.h).filter(|(x, y)| (*x - *y).abs() > 0.0).count();
        assert_eq!(changed, 2);
        // un bout dans une maison, l'autre sur le bord de la grille : rien n'est écrit
        let mut kept = raw.clone();
        let edge = Vec2::new(HALF, 5.0);
        kept.pin_road_ends((a, 1.0), (edge, 2.0), &[(Vec2::new(-12.0, 3.0), Vec2::new(-9.0, 6.0))]);
        assert!(kept == raw, "rien n'est écrit");
        // sans relief, rien à réécrire
        let mut flat = Terrain::default();
        flat.pin_road_ends((a, 1.0), (b, 2.0), &[]);
        assert_eq!(flat.h, Vec::<f32>::new());
    }
}
