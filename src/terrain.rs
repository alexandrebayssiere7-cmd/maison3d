//! Relief du terrain : grille d'altitudes au mètre, pinceaux de sculpture et génération aléatoire.

use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Cases de 1 m par côté, centrées sur l'origine ; au-delà, le sol reste plat à 0.
pub const N: usize = 160;
pub const SIDE: usize = N + 1;
const HALF: f32 = N as f32 / 2.0;

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
                Brush::Smooth => h + ((t.at(i - 1, j) + t.at(i + 1, j) + t.at(i, j - 1) + t.at(i, j + 1)) / 4.0 - h) * k,
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

    /// Terrassement d'une route : pente régulière de `a` à `b` sur sa largeur plus un accotement.
    pub fn grade(&mut self, a: Vec2, b: Vec2, width: f32) {
        if self.h.is_empty() || a == b {
            return;
        }
        let (ya, yb) = (self.height(a), self.height(b));
        let len2 = a.distance_squared(b);
        self.each((a + b) / 2.0, a.distance(b) / 2.0 + width / 2.0 + 2.0, |t, i, j, _| {
            let p = Self::vertex(i, j);
            let k = ((p - a).dot(b - a) / len2).clamp(0.0, 1.0);
            if p.distance(a.lerp(b, k)) <= width / 2.0 + 1.0 { ya + (yb - ya) * k } else { t.at(i, j) }
        });
    }

    /// Relief aléatoire : collines douces ou montagnes à crêtes, de taille `scale` (m).
    /// La zone `keep` (la maison) reste plate, et le terrain retombe à 0 sur les bords.
    pub fn generate(&mut self, seed: u32, amp: f32, scale: f32, mountains: bool, keep: Option<(Vec2, Vec2)>) {
        let smooth = |x: f32| {
            let x = x.clamp(0.0, 1.0);
            x * x * (3.0 - 2.0 * x)
        };
        self.h = (0..SIDE * SIDE)
            .map(|k| {
                let p = Self::vertex(k % SIDE, k / SIDE);
                let n = fbm(p / scale, seed);
                let shape = if mountains { (1.0 - (2.0 * n - 1.0).abs()).powi(3) * 2.0 } else { 2.0 * n - 0.8 };
                let edge = smooth((HALF - p.abs().max_element()) / 15.0);
                let near = keep.map_or(1.0, |(lo, hi)| smooth((p.distance(p.clamp(lo, hi)) - 3.0) / 15.0));
                amp * shape * edge * near
            })
            .collect();
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
                    let m = (a + b) / 2.0;
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
    let mut h = (i as u32).wrapping_mul(374_761_393) ^ (j as u32).wrapping_mul(668_265_263) ^ s.wrapping_mul(2_246_822_519);
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
        t.grade(a, b, 4.0);
        let mid = t.height(Vec2::new(-40.0, -40.0));
        assert!((mid - (t.height(a) + t.height(b)) / 2.0).abs() < 0.05);
    }
}
