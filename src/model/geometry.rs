//! Conversion des murs, toits, piscines et routes en pièces géométriques ; sélection par rayon.

use bevy::math::{Quat, Vec2, Vec3};
use bevy::transform::components::Transform;

use super::{
    ElemRef, Finish, OpenFn, OpeningKind, Part, Pool, Road, Roof, RoofKind, Shape, Terrain, WHITE, Wall, cube, elev,
    local_box,
};

pub(super) fn wall_parts(w: &Wall, wi: usize, open: OpenFn, out: &mut Vec<Part>) {
    let len = w.len();
    if len < 1e-3 {
        return;
    }
    let frame = w.frame();
    let (h, th) = (w.height, w.thick);
    let owner = ElemRef::Wall(wi);
    // pièce `slot` de l'élément `owner` : le mur lui-même, ou une de ses ouvertures
    let mut push = |b: Option<Transform>, shape: Shape, owner: ElemRef, slot: usize| {
        let finish = match owner {
            ElemRef::Opening(_, oi) => w.openings[oi].finish(slot),
            _ => Finish { color: w.color, texture: w.texture },
        };
        if let Some(b) = b {
            let (color, texture) = (finish.color, finish.texture);
            out.push(Part { tf: frame.mul_transform(b), shape, color, texture, owner, slot });
        }
    };
    let mut order: Vec<usize> = (0..w.openings.len()).collect();
    order.sort_by(|&i, &j| w.openings[i].t.total_cmp(&w.openings[j].t));
    // les murs dépassent de th/2 à chaque bout pour que les angles se referment
    let mut x = -th / 2.0;
    for oi in order {
        let o = &w.openings[oi];
        let (x0, x1) = (o.t - o.width / 2.0, o.t + o.width / 2.0);
        let top = (o.sill + o.height).min(h);
        push(local_box(x, x0, 0.0, h, th), Shape::Cube, owner, 0);
        push(local_box(x0, x1, 0.0, o.sill, th), Shape::Cube, owner, 0);
        push(local_box(x0, x1, top, h, th), Shape::Cube, owner, 0);
        x = x1;

        let own = ElemRef::Opening(wi, oi);
        let f = 0.07;
        let fd = th + 0.04;
        // le cadre est la partie 1 d'une porte (le battant la 0), la partie 0 des autres
        let fr = usize::from(o.kind == OpeningKind::Door);
        push(local_box(x0, x0 + f, o.sill, top, fd), Shape::Cube, own, fr);
        push(local_box(x1 - f, x1, o.sill, top, fd), Shape::Cube, own, fr);
        push(local_box(x0, x1, top - f, top, fd), Shape::Cube, own, fr);
        match o.kind {
            OpeningKind::FrenchDoor => {
                // deux battants vitrés qui s'ouvrent vers l'intérieur
                let pw = (x1 - x0 - 2.0 * f) / 2.0;
                for (hx, side) in [(x0 + f, 1.0), (x1 - f, -1.0)] {
                    let hinge =
                        Transform::from_xyz(hx, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-side * open(own) * 1.6));
                    let mut leaf = |a: f32, b: f32, y0: f32, y1: f32, d: f32, shape: Shape| {
                        let (a, b) = if side > 0.0 { (a, b) } else { (-b, -a) };
                        push(local_box(a, b, y0, y1, d).map(|b| hinge.mul_transform(b)), shape, own, 0);
                    };
                    leaf(0.0, f, 0.0, top - f, 0.06, Shape::Cube);
                    leaf(pw - f, pw, 0.0, top - f, 0.06, Shape::Cube);
                    leaf(0.0, pw, top - 2.0 * f, top - f, 0.06, Shape::Cube);
                    leaf(0.0, pw, 0.0, 0.15, 0.06, Shape::Cube);
                    leaf(f, pw - f, 0.15, top - 2.0 * f, 0.02, Shape::Glass);
                }
            }
            OpeningKind::Door => {
                // le battant pivote autour de sa charnière (montant gauche)
                let hinge =
                    Transform::from_xyz(x0 + f, 0.0, 0.0).with_rotation(Quat::from_rotation_y(-open(own) * 1.6));
                let pw = x1 - x0 - 2.0 * f;
                push(local_box(0.0, pw, 0.0, top - f, 0.05).map(|b| hinge.mul_transform(b)), Shape::Cube, own, 0);
                let knob = local_box(pw - 0.13, pw - 0.06, 0.98, 1.04, 0.14).map(|b| hinge.mul_transform(b));
                push(knob, Shape::Cube, own, 2);
            }
            OpeningKind::Window | OpeningKind::Bay | OpeningKind::Fixed | OpeningKind::Grid => {
                push(local_box(x0, x1, o.sill, o.sill + f, fd), Shape::Cube, own, 0);
                if o.sill > 0.0 {
                    let sill = local_box(x0 - 0.05, x1 + 0.05, o.sill - 0.04, o.sill, th + 0.12);
                    push(sill, Shape::Cube, own, 1);
                }
                if o.kind == OpeningKind::Bay {
                    let m = o.t;
                    push(local_box(m - f / 2.0, m + f / 2.0, o.sill, top, fd), Shape::Cube, own, 0);
                }
                if o.kind == OpeningKind::Grid {
                    // petits bois : un montant et deux traverses
                    let (m, b) = (o.t, 0.015);
                    push(local_box(m - b, m + b, o.sill, top, 0.06), Shape::Cube, own, 0);
                    for k in [1.0, 2.0] {
                        let y = o.sill + f + (top - o.sill - 2.0 * f) * k / 3.0;
                        push(local_box(x0, x1, y - b, y + b, 0.06), Shape::Cube, own, 0);
                    }
                }
                push(local_box(x0 + f, x1 - f, o.sill + f, top - f, 0.02), Shape::Glass, own, 0);
            }
        }
    }
    push(local_box(x, len + th / 2.0, 0.0, h, th), Shape::Cube, owner, 0);
}

pub(super) fn roof_part(r: &Roof, owner: ElemRef) -> Part {
    let min = r.min.min(r.max) - Vec2::splat(r.overhang);
    let max = r.min.max(r.max) + Vec2::splat(r.overhang);
    let (c, size) = ((min + max) / 2.0, max - min);
    let base = elev(r.level);
    if r.kind == RoofKind::Flat {
        let tf = Transform::from_xyz(c.x, base + 0.15, c.y).with_scale(Vec3::new(size.x, 0.3, size.y));
        return cube(tf, r.color, r.texture, owner);
    }
    // le faîtage suit toujours le grand côté
    let (long, short, rot) =
        if size.x >= size.y { (size.x, size.y, 0.0) } else { (size.y, size.x, std::f32::consts::FRAC_PI_2) };
    Part {
        tf: Transform::from_xyz(c.x, base, c.y).with_rotation(Quat::from_rotation_y(rot)),
        shape: Shape::Roof { kind: r.kind, size: Vec3::new(long, r.height, short) },
        color: r.color,
        texture: r.texture,
        owner,
        slot: 0,
    }
}

/// Boîte entre deux coins au sol et deux altitudes.
fn boxed(a: Vec2, b: Vec2, y0: f32, y1: f32) -> Transform {
    let (c, s) = ((a + b) / 2.0, b - a);
    Transform::from_xyz(c.x, f32::midpoint(y0, y1), c.y).with_scale(Vec3::new(s.x, y1 - y0, s.y))
}

/// Bassin (fond, parois, marches), eau et margelle ; `top` = altitude du terrain au bord.
pub(super) fn pool_parts(p: &Pool, top: f32, owner: ElemRef, out: &mut Vec<Part>) {
    let (min, max) = (p.min, p.max);
    let bottom = top - p.depth;
    let (t, cop) = (0.2, 0.35);
    out.push(cube(boxed(min, max, bottom - 0.2, bottom), p.color, p.texture, owner));
    for (a, b) in [
        (min, Vec2::new(max.x, min.y + t)),
        (Vec2::new(min.x, max.y - t), max),
        (min, Vec2::new(min.x + t, max.y)),
        (Vec2::new(max.x - t, min.y), max),
    ] {
        out.push(cube(boxed(a, b, bottom, top), p.color, p.texture, owner));
    }
    let stone = [0.9, 0.87, 0.8];
    for (a, b) in [
        (min - cop, Vec2::new(max.x + cop, min.y)),
        (Vec2::new(min.x - cop, max.y), max + cop),
        (Vec2::new(min.x - cop, min.y), Vec2::new(min.x, max.y)),
        (Vec2::new(max.x, min.y), Vec2::new(max.x + cop, max.y)),
    ] {
        out.push(cube(boxed(a, b, top - 0.02, top + 0.06), stone, 0, owner));
    }
    // marches pour sortir du bassin, côté x min
    let n = (((max.x - min.x - 2.0 * t) / 0.35) as usize).min(4);
    let w = (max.y - min.y - 2.0 * t).min(1.4);
    for i in 0..n {
        let x = min.x + t + i as f32 * 0.35;
        let y = bottom + p.depth * (n - i) as f32 / (n + 1) as f32;
        out.push(cube(boxed(Vec2::new(x, min.y + t), Vec2::new(x + 0.35, min.y + t + w), bottom, y), WHITE, 0, owner));
    }
    let water = boxed(min + t, max - t, top - 0.17, top - 0.15);
    out.push(Part { tf: water, shape: Shape::Glass, color: p.color, texture: 0, owner, slot: 0 });
}

/// Ruban incliné d'un bout à l'autre, au ras du terrain.
pub(super) fn road_part(r: &Road, terrain: &Terrain, owner: ElemRef) -> Option<Part> {
    let d = r.b - r.a;
    let len = d.length();
    if len < 1e-3 {
        return None;
    }
    let (ya, yb) = (terrain.height(r.a), terrain.height(r.b));
    let c = (r.a + r.b) / 2.0;
    let rot = Quat::from_rotation_y(-d.y.atan2(d.x)) * Quat::from_rotation_z((yb - ya).atan2(len));
    // déborde d'une demi-largeur à chaque bout : les virages se referment
    let long = Vec2::new(len, yb - ya).length() + r.width;
    let tf = Transform::from_xyz(c.x, f32::midpoint(ya, yb) + 0.02, c.y)
        .with_rotation(rot)
        .with_scale(Vec3::new(long, 0.08, r.width));
    Some(cube(tf, r.color, r.texture, owner))
}

/// Triangles d'un toit en pente, faîtage le long de X. `size` = (longueur, hauteur, largeur).
pub fn roof_tris(kind: RoofKind, size: Vec3) -> Vec<[Vec3; 3]> {
    let (a, h, b) = (size.x / 2.0, size.y, size.z / 2.0);
    let rect =
        |a: f32, b: f32, y: f32| [Vec3::new(-a, y, -b), Vec3::new(a, y, -b), Vec3::new(a, y, b), Vec3::new(-a, y, b)];
    let c = rect(a, b, 0.0);
    let mut t = vec![[c[0], c[2], c[1]], [c[0], c[3], c[2]]]; // dessous
    // pans montant de la base `c` jusqu'à un faîtage le long de X, raccourci de `inset` à chaque bout
    let ridge = |t: &mut Vec<[Vec3; 3]>, c: [Vec3; 4], top: f32, inset: f32| {
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
    };
    match kind {
        RoofKind::Gable => ridge(&mut t, c, h, 0.0),
        RoofKind::Hip => ridge(&mut t, c, h, b.min(a)),
        RoofKind::Pyramid => ridge(&mut t, c, h, a),
        RoofKind::Mansard => {
            // brisis raide jusqu'aux deux tiers, puis terrasson à faible pente
            let k = 0.2 * a.min(b);
            let top = rect(a - k, b - k, 0.65 * h);
            for i in 0..4 {
                let j = (i + 1) % 4;
                t.extend([[c[i], c[j], top[j]], [c[i], top[j], top[i]]]);
            }
            ridge(&mut t, top, h, (b - k).min(a - k));
        }
        RoofKind::Shed => {
            let (t0, t1) = (Vec3::new(-a, h, b), Vec3::new(a, h, b));
            t.extend([
                [c[0], c[1], t1],
                [c[0], t1, t0],
                [c[3], c[2], t1],
                [c[3], t1, t0],
                [c[0], t0, c[3]],
                [c[1], c[2], t1],
            ]);
        }
        RoofKind::Flat => {}
    }
    // les faîtages réduits à un point laissent des triangles plats
    t.retain(|[p, q, r]| (*q - *p).cross(*r - *p).length_squared() > 1e-8);
    t
}

// ---------------------------------------------------------------- sélection par rayon

/// Distance d'intersection entre un rayon et un cube unitaire transformé.
pub fn ray_obb(origin: Vec3, dir: Vec3, tf: &Transform) -> Option<f32> {
    let inv = tf.compute_affine().inverse();
    let (o, d) = (inv.transform_point3(origin), inv.transform_vector3(dir));
    let (mut t0, mut t1) = (f32::NEG_INFINITY, f32::INFINITY);
    for i in 0..3 {
        if d[i].abs() < 1e-8 {
            if o[i].abs() > 0.5 {
                return None;
            }
            continue;
        }
        let (a, b) = ((-0.5 - o[i]) / d[i], (0.5 - o[i]) / d[i]);
        t0 = t0.max(a.min(b));
        t1 = t1.min(a.max(b));
    }
    (t0 <= t1 && t1 > 0.0).then_some(t0.max(0.0))
}

/// Élément le plus proche touché par le rayon, avec la distance.
pub fn pick(parts: &[Part], origin: Vec3, dir: Vec3) -> Option<(ElemRef, f32)> {
    pick_part(parts, origin, dir).map(|(p, d)| (p.owner, d))
}

/// Pièce la plus proche touchée par le rayon (pour peindre la partie visée d'un objet), avec la distance.
pub fn pick_part(parts: &[Part], origin: Vec3, dir: Vec3) -> Option<(&Part, f32)> {
    parts
        .iter()
        .filter(|p| p.shape != Shape::Light)
        .filter_map(|p| Some((p, ray_obb(origin, dir, &p.bounds())?)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    fn wall(len: f32) -> Wall {
        Wall {
            a: Vec2::ZERO,
            b: Vec2::new(len, 0.0),
            level: 0,
            height: 3.0,
            thick: 0.2,
            color: WHITE,
            openings: vec![],
            texture: 0,
        }
    }
    #[test]
    fn new_roofs_have_no_flat_triangles() {
        for k in RoofKind::ALL.into_iter().filter(|k| *k != RoofKind::Flat) {
            let tris = roof_tris(k, Vec3::new(10.0, 3.0, 6.0));
            assert!(tris.len() >= 6, "{k:?}");
            let top = tris.iter().flatten().map(|v| v.y).fold(0.0, f32::max);
            assert!((top - 3.0).abs() < 1e-4, "{k:?} culmine à {top}");
        }
    }
    #[test]
    fn door_leaves_a_hole_in_the_wall() {
        let mut w = wall(4.0);
        w.openings.push(Opening {
            kind: OpeningKind::Door,
            t: 2.0,
            width: 1.0,
            height: 2.1,
            sill: 0.0,
            color: WHITE,
            texture: 0,
            finishes: vec![],
        });
        let mut parts = vec![];
        wall_parts(&w, 0, &|_| 0.0, &mut parts);
        // un point au milieu de la porte, à 1 m de haut, ne doit toucher aucun morceau de mur
        let probe = Vec3::new(2.0, 1.0, 0.0);
        for p in parts.iter().filter(|p| p.owner == ElemRef::Wall(0)) {
            let local = p.tf.compute_affine().inverse().transform_point3(probe);
            assert!(local.abs().max_element() > 0.5, "mur dans la porte : {:?}", p.tf);
        }
        assert_eq!(parts.iter().filter(|p| p.owner == ElemRef::Wall(0)).count(), 3);
        assert!(parts.iter().any(|p| p.owner == ElemRef::Opening(0, 0)));
    }
    #[test]
    fn ray_picks_nearest_element() {
        let h = demo_house();
        let parts = h.parts_closed(Some(0));
        // depuis le sud, vers le nord, à 2 m de haut sur x = 5 : le mur sud (mur 0) est le premier touché
        let hit = pick(&parts, Vec3::new(5.0, 2.0, -20.0), Vec3::Z);
        assert_eq!(hit.map(|h| h.0), Some(ElemRef::Wall(0)));
        assert!(pick(&parts, Vec3::new(500.0, 2.0, 0.0), Vec3::Y).is_none());
    }
    #[test]
    fn doors_swing_open() {
        let h = demo_house();
        let door = ElemRef::Opening(0, 0);
        assert!(h.openable(door) && !h.openable(ElemRef::Opening(0, 1)), "porte oui, fenêtre non");
        let panel = |open: f32| {
            let parts = h.parts(Some(0), &|e| if e == door { open } else { 0.0 });
            // le battant : haut et fin (5 cm)
            parts
                .into_iter()
                .find(|p| p.owner == door && p.tf.scale.y > 1.5 && (p.tf.scale.z - 0.05).abs() < 1e-4)
                .unwrap()
                .tf
        };
        let (closed, open) = (panel(0.0), panel(1.0));
        assert!((closed.translation.z - 0.0).abs() < 0.01, "fermée : dans le plan du mur");
        assert!(open.translation.z > 0.3, "ouverte : sortie du plan du mur");
        let wardrobe = h.props.iter().position(|p| p.kind == PropKind::Wardrobe).unwrap();
        assert!(h.openable(ElemRef::Prop(wardrobe)));
    }
}
