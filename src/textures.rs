//! Textures procédurales (briques, parquets, carrelages, tuiles…) générées au démarrage.
//! Toutes se répètent sans couture ; `scale` est la taille réelle d'un motif, en mètres.

use std::f32::consts::TAU;

use crate::model::Rgb;

pub const SIZE: usize = 256;

pub struct Tex {
    pub name: &'static str,
    pub scale: f32,
    pub color: Box<dyn Fn(f32, f32) -> Rgb>,
}

/// Identifiant de texture d'un élément : 0 = aucune, sinon index dans `catalog()` + 1.
pub fn find(name: &str) -> usize {
    names().iter().position(|n| *n == name).map_or(0, |i| i + 1)
}

/// Nombre de matières : les indices valides vont de 0 (aucune) à `count()`.
pub fn count() -> usize {
    names().len()
}

fn names() -> &'static [&'static str] {
    // appelé à chaque reconstruction (écorce des troncs) : noms calculés une seule fois
    static NAMES: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    NAMES.get_or_init(|| catalog().iter().map(|t| t.name).collect())
}

fn tex(name: &'static str, scale: f32, f: impl Fn(f32, f32) -> Rgb + 'static) -> Tex {
    Tex { name, scale, color: Box::new(f) }
}

pub fn catalog() -> Vec<Tex> {
    let mut all = building();
    // ajoutées en fin de liste : les indices des mondes enregistrés ne bougent pas
    all.extend(furnishing());
    all
}

fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    [f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0]
}

/// Murs, sols, toitures et végétation.
fn building() -> Vec<Tex> {
    vec![
        // murs
        tex("Brique rouge", 1.0, bricks(14.0, 4.0, rgb(150, 62, 42), rgb(200, 195, 185), 0.18, 1)),
        tex("Brique orangée", 1.0, bricks(14.0, 4.0, rgb(190, 100, 60), rgb(215, 210, 200), 0.15, 2)),
        tex("Brique jaune", 1.0, bricks(14.0, 4.0, rgb(205, 170, 105), rgb(220, 215, 200), 0.12, 3)),
        tex("Brique peinte blanche", 1.0, bricks(14.0, 4.0, rgb(235, 233, 228), rgb(200, 198, 195), 0.04, 4)),
        tex("Brique anthracite", 1.0, bricks(14.0, 4.0, rgb(70, 68, 70), rgb(150, 150, 150), 0.2, 5)),
        tex("Brique ancienne", 1.0, bricks(14.0, 4.0, rgb(140, 75, 55), rgb(180, 170, 150), 0.4, 6)),
        tex("Pierre de taille", 2.0, bricks(6.0, 3.0, rgb(214, 200, 170), rgb(190, 180, 160), 0.1, 7)),
        tex("Moellons", 1.5, stones(5.0, rgb(180, 165, 140), rgb(120, 112, 100), 0.25, 8)),
        tex("Pierre grise", 1.5, stones(6.0, rgb(150, 150, 148), rgb(90, 90, 90), 0.2, 9)),
        tex("Schiste", 1.0, bricks(16.0, 3.0, rgb(80, 85, 92), rgb(45, 45, 50), 0.3, 10)),
        tex("Crépi blanc", 1.0, plain(rgb(240, 238, 232), 0.12, 24, 11)),
        tex("Crépi beige", 1.0, plain(rgb(228, 214, 188), 0.12, 24, 12)),
        tex("Crépi ocre", 1.0, plain(rgb(214, 170, 110), 0.12, 24, 13)),
        tex("Crépi rose", 1.0, plain(rgb(225, 180, 165), 0.12, 24, 14)),
        tex("Enduit gris", 1.0, plain(rgb(170, 170, 168), 0.15, 16, 15)),
        tex(
            "Béton brut",
            2.4,
            tiles(2.0, rgb(160, 158, 152), rgb(160, 158, 152), rgb(120, 118, 115), 0.004, false, 0.04, 16),
        ),
        tex("Bardage bois", 1.0, siding(8.0, rgb(160, 115, 75), false, 17)),
        tex("Bardage vertical", 1.0, siding(8.0, rgb(150, 105, 70), true, 18)),
        tex("Bardage gris", 1.0, siding(8.0, rgb(140, 140, 138), false, 19)),
        tex("Bardage rouge scandinave", 1.0, siding(8.0, rgb(140, 40, 35), true, 20)),
        tex("Plâtre", 1.0, plain(rgb(245, 244, 240), 0.05, 8, 21)),
        tex("Papier peint rayé", 0.5, stripes(6.0, rgb(230, 225, 210), rgb(200, 210, 200))),
        tex("Papier peint bleu", 0.5, stripes(10.0, rgb(180, 195, 220), rgb(160, 178, 210))),
        tex(
            "Faïence blanche",
            0.6,
            tiles(6.0, rgb(245, 245, 245), rgb(245, 245, 245), rgb(190, 190, 190), 0.004, false, 0.03, 24),
        ),
        tex("Carreaux métro", 0.6, bricks(8.0, 4.0, rgb(240, 240, 238), rgb(170, 170, 170), 0.03, 25)),
        // sols
        tex("Parquet chêne clair", 2.0, planks(14.0, 2.0, rgb(215, 180, 135), 0.12, 26)),
        tex("Parquet chêne", 2.0, planks(14.0, 2.0, rgb(180, 130, 80), 0.15, 27)),
        tex("Parquet noyer", 2.0, planks(14.0, 2.0, rgb(115, 75, 45), 0.15, 28)),
        tex("Parquet wengé", 2.0, planks(14.0, 2.0, rgb(65, 45, 35), 0.15, 29)),
        tex("Parquet gris vieilli", 2.0, planks(14.0, 2.0, rgb(165, 155, 140), 0.12, 30)),
        tex("Plancher brut", 2.0, planks(8.0, 1.0, rgb(170, 125, 80), 0.25, 31)),
        tex(
            "Carrelage blanc",
            1.2,
            tiles(2.0, rgb(235, 233, 228), rgb(235, 233, 228), rgb(180, 178, 172), 0.003, false, 0.03, 32),
        ),
        tex(
            "Carrelage gris",
            1.2,
            tiles(2.0, rgb(140, 140, 140), rgb(140, 140, 140), rgb(100, 100, 100), 0.003, false, 0.05, 33),
        ),
        tex(
            "Carrelage noir",
            1.2,
            tiles(2.0, rgb(40, 40, 42), rgb(40, 40, 42), rgb(90, 90, 90), 0.003, false, 0.05, 34),
        ),
        tex(
            "Damier noir et blanc",
            1.2,
            tiles(4.0, rgb(235, 235, 235), rgb(30, 30, 30), rgb(120, 120, 120), 0.002, true, 0.02, 35),
        ),
        tex("Tomettes", 1.0, tiles(5.0, rgb(175, 85, 55), rgb(175, 85, 55), rgb(200, 190, 175), 0.006, false, 0.2, 36)),
        tex("Marbre blanc", 2.0, marble(rgb(238, 236, 232), rgb(150, 150, 155), 2.0, 37)),
        tex("Marbre noir", 2.0, marble(rgb(30, 30, 33), rgb(200, 200, 200), 2.0, 38)),
        tex(
            "Mosaïque bleue",
            0.6,
            tiles(16.0, rgb(40, 110, 170), rgb(40, 110, 170), rgb(225, 225, 225), 0.003, false, 0.35, 39),
        ),
        tex("Moquette beige", 1.0, plain(rgb(200, 185, 160), 0.2, 96, 40)),
        tex("Moquette grise", 1.0, plain(rgb(120, 120, 125), 0.2, 96, 41)),
        tex("Béton ciré", 3.0, plain(rgb(175, 172, 165), 0.18, 4, 42)),
        // extérieur
        tex("Pavés", 1.0, bricks(8.0, 4.0, rgb(130, 128, 125), rgb(80, 78, 75), 0.25, 43)),
        tex(
            "Dalles de jardin",
            1.2,
            tiles(2.0, rgb(200, 192, 175), rgb(200, 192, 175), rgb(110, 105, 95), 0.01, false, 0.1, 44),
        ),
        tex("Gravier", 1.0, stones(28.0, rgb(175, 168, 155), rgb(90, 85, 80), 0.45, 45)),
        tex("Herbe", 1.0, grass()),
        tex("Sable", 1.0, plain(rgb(225, 205, 160), 0.2, 64, 47)),
        // toitures
        tex("Tuiles terre cuite", 1.0, roof_tiles(6.0, 4.0, rgb(175, 75, 50), 0.15, 48)),
        tex("Tuiles brunes", 1.0, roof_tiles(6.0, 4.0, rgb(110, 65, 45), 0.15, 49)),
        tex("Tuiles noires", 1.0, roof_tiles(6.0, 4.0, rgb(55, 55, 58), 0.1, 50)),
        tex("Ardoise", 1.0, bricks(6.0, 4.0, rgb(65, 70, 80), rgb(35, 37, 42), 0.15, 51)),
        tex("Bardeaux bois", 1.0, bricks(6.0, 5.0, rgb(140, 100, 65), rgb(70, 50, 35), 0.25, 52)),
        tex("Tôle ondulée", 1.0, corrugated(12.0, rgb(170, 172, 175))),
        tex("Bac acier noir", 1.0, corrugated(3.0, rgb(55, 57, 60))),
        tex("Zinc", 1.0, siding(2.0, rgb(150, 155, 160), true, 56)),
        // végétation (échelle 0,5 / 0,25 : un nombre entier de motifs autour des sphères et cylindres)
        tex("Feuillage", 0.5, leaves(rgb(40, 95, 35), rgb(95, 150, 55), 36.0, 57)),
        tex("Feuillage clair", 0.5, leaves(rgb(70, 125, 45), rgb(140, 180, 70), 36.0, 58)),
        tex("Feuillage d'automne", 0.5, leaves(rgb(150, 60, 25), rgb(230, 160, 50), 36.0, 59)),
        tex("Aiguilles de pin", 0.5, leaves(rgb(20, 60, 35), rgb(55, 105, 60), 56.0, 60)),
        tex("Écorce", 0.25, bark(rgb(105, 80, 60), 61)),
        tex("Écorce de bouleau", 0.25, birch()),
    ]
}

/// Mobilier, terrains de sport et rues.
fn furnishing() -> Vec<Tex> {
    vec![
        // mobilier
        tex("Tissu gris", 0.5, plain(rgb(150, 150, 155), 0.12, 128, 62)),
        tex("Tissu beige", 0.5, plain(rgb(205, 190, 165), 0.12, 128, 63)),
        tex("Velours bleu", 0.5, plain(rgb(50, 70, 120), 0.18, 64, 64)),
        tex("Cuir brun", 0.5, plain(rgb(110, 65, 40), 0.14, 32, 65)),
        tex("Cuir noir", 0.5, plain(rgb(38, 35, 35), 0.12, 32, 66)),
        tex("Bois clair", 1.0, planks(8.0, 1.0, rgb(210, 175, 130), 0.1, 67)),
        tex("Bois foncé", 1.0, planks(8.0, 1.0, rgb(95, 62, 40), 0.12, 68)),
        tex("Inox brossé", 0.25, stripes(48.0, rgb(196, 198, 201), rgb(178, 180, 184))),
        tex("Chrome", 1.0, plain(rgb(215, 218, 222), 0.04, 8, 70)),
        tex("Papier de riz", 0.5, plain(rgb(245, 240, 222), 0.06, 32, 71)),
        tex("Tatami", 0.5, stripes(40.0, rgb(200, 190, 130), rgb(184, 174, 114))),
        // sport et rue
        tex("Gazon synthétique", 10.0, stripes(2.0, rgb(70, 140, 60), rgb(60, 125, 52))),
        tex("Terre battue", 1.0, plain(rgb(190, 95, 55), 0.15, 64, 74)),
        tex("Résine bleue", 1.0, plain(rgb(45, 90, 150), 0.06, 32, 75)),
        tex("Résine verte", 1.0, plain(rgb(55, 120, 80), 0.06, 32, 76)),
        tex("Parquet de sport", 2.0, planks(20.0, 3.0, rgb(210, 165, 105), 0.08, 77)),
        tex("Tartan", 1.0, plain(rgb(170, 60, 45), 0.12, 96, 78)),
        tex("Asphalte", 1.0, plain(rgb(70, 70, 72), 0.18, 128, 79)),
    ]
}

/// Pixels RGBA (sRGB) d'une texture, avec toute la chaîne de mipmaps à la suite.
pub fn pixels_with_mips(t: &Tex) -> (Vec<u8>, u32) {
    let mut level: Vec<[f32; 3]> = (0..SIZE * SIZE)
        .map(|i| {
            let (x, y) = (i % SIZE, i / SIZE);
            (t.color)((x as f32 + 0.5) / SIZE as f32, (y as f32 + 0.5) / SIZE as f32)
        })
        .collect();
    let (mut out, mut n, mut count) = (Vec::new(), SIZE, 0);
    loop {
        for c in &level {
            out.extend(c.map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8));
            out.push(255);
        }
        count += 1;
        if n == 1 {
            return (out, count);
        }
        let m = n / 2;
        level = (0..m * m)
            .map(|i| {
                let (x, y) = (i % m * 2, i / m * 2);
                let px = |dx, dy| level[(y + dy) * n + x + dx];
                let mut s = [0.0; 3];
                for p in [px(0, 0), px(1, 0), px(0, 1), px(1, 1)] {
                    for k in 0..3 {
                        s[k] += p[k] / 4.0;
                    }
                }
                s
            })
            .collect();
        n = m;
    }
}

// ---------------------------------------------------------------- bruit

fn hash(x: i32, y: i32, s: u32) -> f32 {
    let mut h =
        (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ s.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xff_ffff) as f32 / 16_777_216.0
}

/// Bruit de valeur lissé, périodique (px, py) pour que la texture se répète sans couture.
fn vnoise(x: f32, y: f32, px: i32, py: i32, s: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - xi as f32, y - yi as f32);
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let h = |i: i32, j: i32| hash((xi + i).rem_euclid(px), (yi + j).rem_euclid(py), s);
    let a = h(0, 0) + (h(1, 0) - h(0, 0)) * ux;
    let b = h(0, 1) + (h(1, 1) - h(0, 1)) * ux;
    a + (b - a) * uy
}

/// Bruit fractal 0..1 sur [0,1)², fréquence de base `f`.
fn fbm(u: f32, v: f32, f: i32, s: u32) -> f32 {
    let (mut amp, mut freq, mut sum, mut norm) = (0.5, f, 0.0, 0.0);
    for o in 0..4 {
        sum += amp * vnoise(u * freq as f32, v * freq as f32, freq, freq, s + o);
        norm += amp;
        amp *= 0.5;
        freq *= 2;
    }
    sum / norm
}

fn mul(c: Rgb, k: f32) -> Rgb {
    c.map(|v| v * k)
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

/// Variation de teinte ±var par élément (brique, lame…).
fn vary(c: Rgb, var: f32, h: f32) -> Rgb {
    mul(c, 1.0 + var * (h - 0.5) * 2.0)
}

// ---------------------------------------------------------------- motifs

/// Appareillage de briques décalé d'une demi-brique ; `rows` doit être pair.
fn bricks(rows: f32, cols: f32, brick: Rgb, mortar: Rgb, var: f32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let r = (v * rows).floor();
        let x = u * cols + if r as i32 % 2 == 1 { 0.5 } else { 0.0 };
        let c = (x.floor() as i32).rem_euclid(cols as i32);
        let (fu, fv) = (x.fract(), (v * rows).fract());
        let n = 0.85 + 0.3 * fbm(u, v, 16, s);
        let joint = 0.012;
        if fu < joint * cols || fv < joint * rows {
            mul(mortar, n)
        } else {
            mul(vary(brick, var, hash(c, r as i32, s)), n)
        }
    }
}

fn tiles(
    n: f32,
    a: Rgb,
    b: Rgb,
    joint_c: Rgb,
    joint: f32,
    checker: bool,
    var: f32,
    s: u32,
) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let (i, j) = ((u * n).floor() as i32, (v * n).floor() as i32);
        let (fu, fv) = ((u * n).fract(), (v * n).fract());
        if fu < joint * n || fv < joint * n {
            return joint_c;
        }
        let base = if checker && (i + j) % 2 == 1 { b } else { a };
        mul(vary(base, var, hash(i, j, s)), 0.94 + 0.12 * fbm(u, v, 8, s))
    }
}

/// Lames de parquet : `rows` lames dans la hauteur, `segs` longueurs par ligne, joints décalés.
fn planks(rows: f32, segs: f32, wood: Rgb, var: f32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let r = (v * rows).floor() as i32;
        let x = u * segs + hash(r, 0, s);
        let seg = (x.floor() as i32).rem_euclid(segs as i32);
        let (fu, fv) = (x.fract(), (v * rows).fract());
        if fv < 0.04 || fu < 0.004 * segs {
            return mul(wood, 0.45);
        }
        let rows_i = rows as i32;
        let streak = vnoise(u * 3.0, v * rows * 10.0, 3, rows_i * 10, s + seg as u32);
        let fine = vnoise(u * 48.0, v * rows * 24.0, 48, rows_i * 24, s + 7);
        mul(vary(wood, var, hash(seg, r, s + 1)), 0.75 + 0.35 * streak + 0.1 * fine)
    }
}

/// Bardage à clin : planches avec une ombre sous chaque recouvrement.
fn siding(rows: f32, wood: Rgb, vertical: bool, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let (u, v) = if vertical { (v, u) } else { (u, v) };
        let fv = (v * rows).fract();
        let shade = if fv > 0.92 { 0.5 } else { 0.8 + 0.25 * fv };
        let grain = vnoise(u * 6.0, v * rows * 16.0, 6, rows as i32 * 16, s);
        mul(wood, shade * (0.9 + 0.2 * grain))
    }
}

/// Pierres irrégulières (cellules de Voronoï périodiques).
fn stones(cells: f32, stone: Rgb, joint: Rgb, var: f32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let n = cells as i32;
        let (x, y) = (u * cells, v * cells);
        let (cx, cy) = (x.floor() as i32, y.floor() as i32);
        let (mut d1, mut d2, mut id) = (f32::MAX, f32::MAX, (0, 0));
        for dx in -1..=1 {
            for dy in -1..=1 {
                let (gx, gy) = (cx + dx, cy + dy);
                let (wx, wy) = (gx.rem_euclid(n), gy.rem_euclid(n));
                let px = gx as f32 + 0.15 + 0.7 * hash(wx, wy, s);
                let py = gy as f32 + 0.15 + 0.7 * hash(wx, wy, s + 1);
                let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
                if d < d1 {
                    (d2, d1, id) = (d1, d, (wx, wy));
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        if d2 - d1 < 0.08 {
            return joint;
        }
        let edge = ((d2 - d1) * 4.0).min(1.0);
        mul(vary(stone, var, hash(id.0, id.1, s + 2)), (0.7 + 0.3 * edge) * (0.85 + 0.3 * fbm(u, v, 16, s)))
    }
}

/// Tuiles en écailles, rangées décalées ; `rows` doit être pair.
fn roof_tiles(rows: f32, cols: f32, c: Rgb, var: f32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let r = (v * rows).floor() as i32;
        let x = u * cols + if r % 2 == 1 { 0.5 } else { 0.0 };
        let col = (x.floor() as i32).rem_euclid(cols as i32);
        let (fx, fv) = (x.fract() - 0.5, (v * rows).fract());
        let round = 1.0 - (2.0 * fx).powi(2);
        if fx.abs() > 0.47 || fv > 0.8 + 0.2 * round {
            return mul(c, 0.4);
        }
        mul(vary(c, var, hash(col, r, s)), (0.6 + 0.45 * fv) * (0.8 + 0.2 * round))
    }
}

fn corrugated(waves: f32, c: Rgb) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| mul(c, (0.72 + 0.28 * (0.5 + 0.5 * (TAU * u * waves).sin())) * (0.95 + 0.1 * fbm(u, v, 8, 3)))
}

fn marble(base: Rgb, vein: Rgb, k: f32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let t = ((u + v) * k * TAU + 7.0 * fbm(u, v, 4, s)).sin();
        mix(base, vein, (1.0 - t.abs()).powf(10.0) * 0.8)
    }
}

fn stripes(n: f32, a: Rgb, b: Rgb) -> impl Fn(f32, f32) -> Rgb {
    move |u, _| if (u * n).fract() < 0.5 { a } else { b }
}

fn plain(base: Rgb, amount: f32, freq: i32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| mul(base, 1.0 + amount * (fbm(u, v, freq, s) - 0.5) * 2.0)
}

/// Feuilles : petites cellules de teintes variées, éclairées au centre, ombres entre elles,
/// et grosses masses plus sombres pour donner du relief au feuillage.
fn leaves(dark: Rgb, light: Rgb, cells: f32, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let n = cells as i32;
        let (x, y) = (u * cells, v * cells);
        let (cx, cy) = (x.floor() as i32, y.floor() as i32);
        let (mut d1, mut d2, mut id) = (f32::MAX, f32::MAX, (0, 0));
        for dx in -1..=1 {
            for dy in -1..=1 {
                let (gx, gy) = (cx + dx, cy + dy);
                let (wx, wy) = (gx.rem_euclid(n), gy.rem_euclid(n));
                let px = gx as f32 + hash(wx, wy, s);
                let py = gy as f32 + hash(wx, wy, s + 1);
                let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
                if d < d1 {
                    (d2, d1, id) = (d1, d, (wx, wy));
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        let leaf = mix(dark, light, hash(id.0, id.1, s + 2));
        let lit = 1.05 - 0.55 * d1.min(1.0);
        let gap = ((d2 - d1) * 6.0).min(1.0);
        let masses = 0.55 + 0.65 * fbm(u, v, 4, s + 3);
        mul(leaf, lit * (0.35 + 0.65 * gap) * masses)
    }
}

/// Écorce : sillons verticaux irréguliers.
fn bark(c: Rgb, s: u32) -> impl Fn(f32, f32) -> Rgb {
    move |u, v| {
        let ridges = vnoise(u * 24.0, v * 3.0, 24, 3, s);
        let fine = fbm(u, v, 16, s + 1);
        let groove = if ridges < 0.35 { 0.45 } else { 0.8 + 0.4 * (ridges - 0.35) };
        mul(c, groove * (0.8 + 0.4 * fine))
    }
}

fn birch() -> impl Fn(f32, f32) -> Rgb {
    |u, v| {
        let mark = vnoise(u * 6.0, v * 40.0, 6, 40, 62);
        let base = mul([0.92, 0.9, 0.86], 0.9 + 0.15 * fbm(u, v, 8, 63));
        if mark > 0.72 { mul([0.15, 0.13, 0.12], 1.0 + mark * 0.3) } else { base }
    }
}

fn grass() -> impl Fn(f32, f32) -> Rgb {
    |u, v| {
        let blades = vnoise(u * 160.0, v * 40.0, 160, 40, 46);
        let patches = fbm(u, v, 6, 47);
        mul(mix([0.25, 0.45, 0.18], [0.42, 0.58, 0.25], patches), 0.75 + 0.45 * blades)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_about_fifty_unique_textures() {
        let c = catalog();
        assert!(c.len() >= 50, "{} textures", c.len());
        let mut names: Vec<_> = c.iter().map(|t| t.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), c.len(), "noms en double");
        assert_eq!(find("Brique rouge"), 1);
        // les mondes enregistrent des indices : une nouvelle matière s'ajoute à la fin
        assert_eq!((find("Écorce de bouleau"), find("Tissu gris")), (61, 62));
        assert_eq!(find("inconnue"), 0);
    }

    #[test]
    fn textures_tile_seamlessly() {
        // le saut entre dernière et première colonne (ou ligne) ne doit pas être plus marqué
        // que le pire saut entre deux colonnes voisines à l'intérieur (un joint, par exemple)
        let px = |i: usize| (i as f32 + 0.5) / SIZE as f32;
        let diff = |a: Rgb, b: Rgb| (0..3).map(|k| (a[k] - b[k]).abs()).sum::<f32>();
        for t in catalog() {
            let f = &t.color;
            let jump = |horizontal: bool, i: usize| -> f32 {
                let j = (i + 1) % SIZE;
                (0..SIZE)
                    .step_by(4)
                    .map(|k| {
                        if horizontal {
                            diff(f(px(i), px(k)), f(px(j), px(k)))
                        } else {
                            diff(f(px(k), px(i)), f(px(k), px(j)))
                        }
                    })
                    .sum()
            };
            for horizontal in [true, false] {
                let worst_inside = (0..SIZE - 1).map(|i| jump(horizontal, i)).fold(0.0, f32::max);
                let seam = jump(horizontal, SIZE - 1);
                assert!(seam <= worst_inside * 1.2 + 0.5, "{} : couture visible ({seam} > {worst_inside})", t.name);
            }
        }
    }

    #[test]
    fn mip_chain_size() {
        let (px, levels) = pixels_with_mips(&catalog()[0]);
        assert_eq!(levels, 9);
        assert_eq!(px.len(), (0..9).map(|l| (SIZE >> l).pow(2) * 4).sum::<usize>());
    }
}
