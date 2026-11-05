//! Collisions du visiteur et de la voiture avec les boîtes de la scène, recul de la caméra.

use bevy::prelude::*;

use super::STEP;
use crate::model::{ElemRef, Part, Shape, ray_obb, roof_tris};

/// Ce qui arrête le visiteur (le volume englobant d'un toit en pente, lui, est surtout vide ;
/// on traverse l'eau, et la route inclinée est portée par le terrain terrassé dessous).
pub(super) fn solid(p: &Part) -> bool {
    !matches!(p.shape, Shape::Light | Shape::Roof { .. })
        && !matches!(p.owner, ElemRef::Road(_))
        && (p.shape != Shape::Glass || !matches!(p.owner, ElemRef::Pool(_)))
}

/// Boîte de collision d'une pièce. Les calculs ci-dessous ne gèrent que les rotations autour de
/// la verticale : une pièce penchée ou couchée (échelle, barre transversale) devient sa boîte
/// englobante droite.
pub(super) fn collider(p: &Part) -> Transform {
    let tf = p.bounds();
    if (tf.rotation * Vec3::Y).y > 1.0 - 1e-4 {
        return tf;
    }
    let m = Mat3::from_quat(tf.rotation);
    let size = m.x_axis.abs() * tf.scale.x + m.y_axis.abs() * tf.scale.y + m.z_axis.abs() * tf.scale.z;
    Transform::from_translation(tf.translation).with_scale(size)
}

/// Bas et haut d'une boîte droite (voir `collider`).
fn y_range(tf: &Transform) -> (f32, f32) {
    (tf.translation.y - tf.scale.y / 2.0, tf.translation.y + tf.scale.y / 2.0)
}

/// Repousse un cylindre (rayon, hauteur) hors d'une boîte qu'il traverse.
pub(super) fn push_out(pos: &mut Vec3, tf: &Transform, radius: f32, height: f32) {
    let (y0, y1) = y_range(tf);
    if y1 <= pos.y + STEP || y0 >= pos.y + height {
        return;
    }
    let l = tf.rotation.inverse() * (*pos - tf.translation);
    let h = tf.scale / 2.0;
    let (dx, dz) = (l.x - l.x.clamp(-h.x, h.x), l.z - l.z.clamp(-h.z, h.z));
    let d2 = dx * dx + dz * dz;
    let push = if d2 > 1e-8 {
        if d2 >= radius * radius {
            return;
        }
        let d = d2.sqrt();
        Vec2::new(dx, dz) / d * (radius - d)
    } else {
        // centre à l'intérieur : sortie par le côté le plus proche
        let (px, pz) = (h.x - l.x.abs() + radius, h.z - l.z.abs() + radius);
        if px < pz { Vec2::new(px * l.x.signum(), 0.0) } else { Vec2::new(0.0, pz * l.z.signum()) }
    };
    *pos += tf.rotation * Vec3::new(push.x, 0.0, push.y);
}

/// Hauteur du sol sous les pieds : dessus de la plus haute boîte franchissable, ou le terrain `ground`.
pub(super) fn floor_at(pos: Vec3, solids: &[Transform], ground: f32) -> f32 {
    let mut best = if pos.y >= ground - 0.05 { ground } else { -50.0 };
    for tf in solids {
        let top = y_range(tf).1;
        if top > pos.y + STEP || top <= best {
            continue;
        }
        let l = tf.rotation.inverse() * (pos - tf.translation);
        let h = tf.scale / 2.0;
        if l.x.abs() <= h.x && l.z.abs() <= h.z {
            best = top;
        }
    }
    best
}

/// Distance libre derrière le personnage pour la caméra 3e personne (murs, sols, toits, terrain).
pub(super) fn camera_clearance(parts: &[Part], from: Vec3, dir: Vec3, max: f32, ground: f32) -> f32 {
    let mut best = max;
    for p in parts.iter().filter(|p| p.shape != Shape::Light) {
        let hit = match p.shape {
            Shape::Roof { kind, size } => roof_tris(kind, size)
                .iter()
                .filter_map(|t| ray_triangle(from, dir, t.map(|v| p.tf.transform_point(v))))
                .reduce(f32::min),
            // une boîte qui contient déjà la tête (dossier, oreiller…) ne compte pas
            _ => ray_obb(from, dir, &p.bounds()).filter(|&t| t > 0.01),
        };
        if let Some(t) = hit {
            best = best.min(t);
        }
    }
    if dir.y < 0.0 && from.y > ground {
        best = best.min((from.y - ground) / -dir.y);
    }
    (best - 0.25).max(0.15)
}

/// Möller–Trumbore.
fn ray_triangle(o: Vec3, d: Vec3, [a, b, c]: [Vec3; 3]) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-7 {
        return None;
    }
    let s = o - a;
    let u = s.dot(p) / det;
    let q = s.cross(e1);
    let w = d.dot(q) / det;
    let t = e2.dot(q) / det;
    (u >= 0.0 && w >= 0.0 && u + w <= 1.0 && t > 0.0).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::demo_house;

    #[test]
    fn tilted_pieces_collide_as_their_upright_bounding_box() {
        let part = |rot: Quat| Part {
            tf: Transform::from_xyz(0.0, 2.4, 0.0).with_rotation(rot).with_scale(Vec3::new(0.1, 3.0, 0.1)),
            shape: Shape::Cylinder,
            color: [1.0; 3],
            texture: 0,
            owner: ElemRef::Prop(0),
            slot: 0,
        };
        // barre transversale couchée le long de x, à 2,4 m : on passe dessous
        let bar = collider(&part(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)));
        assert!(bar.rotation.is_near_identity() && bar.scale.abs_diff_eq(Vec3::new(3.0, 0.1, 0.1), 1e-5));
        let mut under = Vec3::new(1.0, 0.0, 0.0);
        push_out(&mut under, &bar, 0.3, 1.8);
        assert_eq!(under, Vec3::new(1.0, 0.0, 0.0), "rien ne bloque sous la barre");
        // une pièce qui ne tourne qu'autour de la verticale garde sa boîte exacte
        let yaw = Quat::from_rotation_y(0.7);
        assert_eq!(collider(&part(yaw)).rotation, yaw);
    }
    #[test]
    fn third_person_camera_stops_before_walls_and_roof() {
        let h = demo_house();
        let parts = h.parts_closed(None);
        // au centre du salon, caméra reculée vers le mur sud (z = 0) : bloquée avant 3,9 m
        let head = Vec3::new(3.0, 1.7, 3.9);
        let free = camera_clearance(&parts, head, -Vec3::Z, 10.0, 0.0);
        assert!(free < 3.8 && free > 3.0, "caméra traverse le mur : {free}");
        // vers le haut : le toit (base à 3 m) arrête la caméra
        let up = camera_clearance(&parts, head, Vec3::Y, 10.0, 0.0);
        assert!(up < 4.0, "caméra traverse le toit : {up}");
        // dehors, rien derrière : distance complète
        assert_eq!(camera_clearance(&parts, Vec3::new(30.0, 1.7, 30.0), Vec3::X, 4.0, 0.0), 3.75);
    }
}
