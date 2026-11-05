//! Suggestions de placement : où poser un objet dans une pièce, d'après son annotation (dos au
//! mur, dégagé, au mur…). Ce sont des propositions classées, pas des règles : l'agent garde
//! `place_prop` et ses coordonnées, et peut s'en écarter.

use bevy::math::Vec2;

use super::annotations::Placement;
use super::check::{Solid, overlap};
use super::query::RoomOpening;
use super::query::on_ground;
use super::{Prop, PropKind, RoomInfo, RoomSide};

/// Une position proposée, avec ce qui l'a fait préférer.
#[derive(Clone, PartialEq, Debug)]
pub struct Suggestion {
    pub pos: Vec2,
    pub rot: f32,
    pub score: f32,
    pub why: Vec<String>,
}

/// Pas de balayage le long des murs et sur le sol.
const STEP: f32 = 0.1;
/// Jeu laissé entre un objet et le mur.
const GAP: f32 = 0.01;

impl super::House {
    /// Positions proposées pour `kind` dans `room`, de la meilleure à la moins bonne, sans deux
    /// positions à moins de 60 cm l'une de l'autre. `facing` : objet vers lequel tourner l'avant.
    pub fn suggest(&self, kind: PropKind, room: &RoomInfo, facing: Option<usize>, count: usize) -> Vec<Suggestion> {
        let placement = kind.annotation().map_or(Placement::Floor, |a| a.placement);
        let solids: Vec<Solid> = room.props.iter().filter_map(|&i| Solid::of(&self.props[i])).collect();
        let mut all: Vec<Suggestion> = (self.candidates(kind, room, placement, facing).into_iter())
            .filter(|(p, _, _)| self.fits(p, room, placement, &solids))
            .map(|(p, why, bonus)| self.rate(&p, room, facing, (vec![why], bonus)))
            .collect();
        all.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut out: Vec<Suggestion> = vec![];
        for s in all {
            if out.len() < count && out.iter().all(|o| o.pos.distance(s.pos) >= 0.6) {
                out.push(s);
            }
        }
        out
    }

    /// Positions à essayer selon la façon de poser l'objet.
    fn candidates(
        &self,
        kind: PropKind,
        room: &RoomInfo,
        placement: Placement,
        facing: Option<usize>,
    ) -> Vec<(Prop, String, f32)> {
        let at = |pos: Vec2, rot: f32| Prop::new(kind, pos, room.level, rot);
        match placement {
            Placement::AgainstWall | Placement::OnWall | Placement::Row => along_walls(room, &at),
            Placement::Floor => {
                let target = facing.and_then(|i| self.props.get(i)).map(|p| p.pos);
                on_floor(room, target, &at)
            }
            Placement::Ceiling => {
                vec![(at((room.inner.0 + room.inner.1) / 2.0, 0.0), "au centre du plafond".into(), 0.0)]
            }
            Placement::OnTop => (room.props.iter())
                .filter(|&&i| goes_with(kind, self.props[i].kind))
                .map(|&i| {
                    let o = &self.props[i];
                    (at(o.pos, o.rot), format!("sur Prop {i} ({:?})", o.kind), 0.0)
                })
                .collect(),
        }
    }

    /// La place est-elle possible ? Dans la pièce, sans heurter un meuble (même règle que
    /// `check`), hors du passage des portes et de l'espace devant les autres meubles ; un objet
    /// mural ne va ni sur une porte ni sur une fenêtre. Un objet posé sur un meuble le touche
    /// forcément.
    fn fits(&self, p: &Prop, room: &RoomInfo, placement: Placement, solids: &[Solid]) -> bool {
        let c = p.corners();
        let clash = Solid::of(p).is_some_and(|s| solids.iter().any(|o| s.clash(o)));
        c.iter().all(|q| inside(room.inner, *q))
            && match placement {
                Placement::OnTop => true,
                Placement::Ceiling => !clash,
                Placement::OnWall => !clash && !on_opening(&c, room, |_| true),
                _ => !clash && !self.in_the_way(&c, room, p.kind) && !blocks_door(&c, room),
            }
    }

    /// Note d'une place possible. `bonus` : préférence du balayage (milieu d'un mur).
    fn rate(
        &self,
        p: &Prop,
        room: &RoomInfo,
        facing: Option<usize>,
        (mut why, bonus): (Vec<String>, f32),
    ) -> Suggestion {
        let c = p.corners();
        let ground = on_ground(p.kind);
        let mut score = bonus;
        let front = front(p);
        if let Some(t) = facing.and_then(|i| self.props.get(i)) {
            let d = front.dot((t.pos - p.pos).normalize_or_zero());
            score += 3.0 * d;
            // devant un meuble dos au mur (la chaise devant le bureau), pas sur son côté
            if t.kind.annotation().is_some_and(|a| a.placement == Placement::AgainstWall) {
                score += 2.0 * self::front(t).dot((p.pos - t.pos).normalize_or_zero());
            }
            why.push(format!("avant vers Prop {} ({:?}) à {:.0} %", facing.unwrap_or(0), t.kind, d.max(0.0) * 100.0));
        }
        if let Some((i, d)) = self.partner(p, room) {
            score += (3.0 - d).max(0.0);
            why.push(format!("à {d:.1} m de Prop {i} ({:?}), qui va avec", self.props[i].kind));
        }
        let clear = p.kind.annotation().map_or(0.0, |a| a.front);
        if ground && clear > 0.0 {
            let zone = ahead(&c, front, clear);
            if zone.iter().all(|q| inside(room.inner, *q)) && self.free(&zone, room, p.kind) {
                score += 1.0;
                why.push(format!("{clear} m libres devant"));
            } else {
                score -= 2.0;
                why.push(format!("moins de {clear} m libres devant"));
            }
        }
        if ground && p.kind.size().z > 0.9 && on_opening(&c, room, |o| !o.door) {
            score -= 1.5;
            why.push("masque une fenêtre".into());
        }
        // une lampe ou une plante va le long d'un mur, pas au milieu du passage
        if facing.is_none() && p.kind.annotation().is_some_and(|a| a.placement == Placement::Floor) && area(&c) < 0.5 {
            let (lo, hi) = room.inner;
            let d = (p.pos - lo).min(hi - p.pos).min_element();
            score -= 0.5 * d;
            why.push(format!("à {d:.1} m d'un mur"));
        }
        // + 0.0 : pas de -0 dans les commandes proposées
        Suggestion { pos: (p.pos * 100.0).round() / 100.0 + 0.0, rot: p.rot + 0.0, score, why }
    }

    /// Aucun meuble de la pièce ne chevauche `c`, sauf ceux qui vont avec `kind` (la chaise
    /// devant le bureau).
    fn free(&self, c: &[Vec2; 4], room: &RoomInfo, kind: PropKind) -> bool {
        (room.props.iter().map(|&i| &self.props[i]))
            .filter(|o| on_ground(o.kind) && !goes_with(kind, o.kind))
            .all(|o| !overlap(c, &o.corners()))
    }

    /// `c` empiète sur l'espace à laisser devant un meuble de la pièce : en entier pour un
    /// meuble sans rapport, sur 40 cm pour un meuble qui va avec et qui est dos au mur (la table
    /// basse devant le canapé, la chaise devant le bureau).
    fn in_the_way(&self, c: &[Vec2; 4], room: &RoomInfo, kind: PropKind) -> bool {
        (room.props.iter().map(|&i| &self.props[i])).filter(|o| on_ground(o.kind)).any(|o| {
            let Some(a) = o.kind.annotation() else { return false };
            let depth = match (goes_with(kind, o.kind), a.placement) {
                (false, _) => a.front,
                (true, Placement::AgainstWall) => a.front.min(0.4),
                (true, _) => 0.0,
            };
            depth > 0.0 && overlap(c, &ahead(&o.corners(), front(o), depth))
        })
    }

    /// Meuble de la pièce le plus proche qui va avec `p`, et sa distance.
    fn partner(&self, p: &Prop, room: &RoomInfo) -> Option<(usize, f32)> {
        (room.props.iter().copied())
            .filter(|&i| goes_with(p.kind, self.props[i].kind))
            .map(|i| (i, self.props[i].pos.distance(p.pos)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}

/// `a` va-t-il avec `b`, dans un sens ou dans l'autre ?
fn goes_with(a: PropKind, b: PropKind) -> bool {
    let with = |x: PropKind, y: PropKind| x.annotation().is_some_and(|n| n.goes_with.contains(&y));
    with(a, b) || with(b, a)
}

/// Dos contre chaque mur nord, sud, est et ouest, avant vers la pièce, tous les 10 cm.
/// Petit bonus vers le milieu du mur, pour départager des places sinon égales.
fn along_walls(room: &RoomInfo, at: &dyn Fn(Vec2, f32) -> Prop) -> Vec<(Prop, String, f32)> {
    let (lo, hi) = room.inner;
    let mut out = vec![];
    for w in &room.walls {
        // avant vers la pièce : [sin rot, cos rot]
        let (rot, inward) = match w.side {
            RoomSide::North => (0.0, Vec2::Y),
            RoomSide::South => (180.0, -Vec2::Y),
            RoomSide::West => (90.0, Vec2::X),
            RoomSide::East => (270.0, -Vec2::X),
            RoomSide::Inside => continue,
        };
        let Some(face) = w.face else { continue };
        let (a, b) = if inward.x == 0.0 { (lo.x, hi.x) } else { (lo.y, hi.y) };
        let mut t = a;
        while t <= b {
            let p = at(if inward.x == 0.0 { Vec2::new(t, face) } else { Vec2::new(face, t) }, rot);
            let back = p.corners().iter().map(|c| c.dot(inward)).fold(f32::MAX, f32::min);
            let pos = p.pos + inward * (face * (inward.x + inward.y) + GAP - back);
            let centered = 0.3 * (1.0 - (2.0 * t - a - b).abs() / (b - a).max(0.1));
            out.push((at(pos, rot), format!("contre le mur {} (Wall {})", w.side.name(), w.wall), centered));
            t += STEP;
        }
    }
    out
}

/// Grille sur le sol de la pièce ; avant vers `target` s'il est donné, sinon droit ou de côté.
fn on_floor(room: &RoomInfo, target: Option<Vec2>, at: &dyn Fn(Vec2, f32) -> Prop) -> Vec<(Prop, String, f32)> {
    let (lo, hi) = room.inner;
    let mut out = vec![];
    let mut z = lo.y;
    while z <= hi.y {
        let mut x = lo.x;
        while x <= hi.x {
            let pos = Vec2::new(x, z);
            let rots = match target {
                // front = [sin rot, cos rot], à 15° près
                Some(t) => vec![((t - pos).x.atan2((t - pos).y).to_degrees() / 15.0).round().rem_euclid(24.0) * 15.0],
                None => vec![0.0, 90.0],
            };
            out.extend(rots.into_iter().map(|r| (at(pos, r), "dégagé".to_owned(), 0.0)));
            x += STEP * 2.0;
        }
        z += STEP * 2.0;
    }
    out
}

fn front(p: &Prop) -> Vec2 {
    let (s, c) = p.rot.to_radians().sin_cos();
    Vec2::new(s, c)
}

/// Rectangle de profondeur `depth` devant l'objet (coins 2 et 3 : son avant).
fn ahead(c: &[Vec2; 4], front: Vec2, depth: f32) -> [Vec2; 4] {
    [c[3], c[2], c[2] + front * depth, c[3] + front * depth]
}

fn inside((lo, hi): (Vec2, Vec2), p: Vec2) -> bool {
    p.cmpge(lo - 0.02).all() && p.cmple(hi + 0.02).all()
}

/// Rectangle centré sur `c`, de demi-côtés `h`.
fn rect(c: Vec2, h: Vec2) -> [Vec2; 4] {
    [c - h, c + Vec2::new(h.x, -h.y), c + h, c + Vec2::new(-h.x, h.y)]
}

/// Rectangle le long du mur de l'ouverture : `along` de part et d'autre de son centre, `across`
/// de part et d'autre de l'axe du mur.
fn opening_zone(o: &RoomOpening, along: f32, across: f32) -> [Vec2; 4] {
    let h = if matches!(o.side, RoomSide::North | RoomSide::South) {
        Vec2::new(along, across)
    } else {
        Vec2::new(across, along)
    };
    rect(o.pos, h)
}

/// Empiète sur le passage d'une porte : sa largeur (plus 10 cm) sur sa largeur en profondeur.
fn blocks_door(c: &[Vec2; 4], room: &RoomInfo) -> bool {
    room.openings.iter().filter(|o| o.door).any(|o| overlap(c, &opening_zone(o, o.width / 2.0 + 0.1, o.width)))
}

/// À moins de 40 cm d'une ouverture retenue par `which`, en face d'elle.
fn on_opening(c: &[Vec2; 4], room: &RoomInfo, which: impl Fn(&RoomOpening) -> bool) -> bool {
    room.openings.iter().filter(|o| which(o)).any(|o| overlap(c, &opening_zone(o, o.width / 2.0, 0.4)))
}

/// Aire au sol de l'emprise `c`.
fn area(c: &[Vec2; 4]) -> f32 {
    c[0].distance(c[1]) * c[1].distance(c[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{House, Opening, OpeningKind, WHITE, room_walls};

    /// Salon 6 × 4 : porte au sud en x = 3, fenêtre au nord en x = 1.5.
    fn living() -> House {
        let mut h =
            House { walls: room_walls(Vec2::ZERO, Vec2::new(6.0, 4.0), 0, 3.0, 0.2, WHITE), ..House::default() };
        h.walls[0].openings.push(Opening::new(OpeningKind::Window, 1.5));
        h.walls[2].openings.push(Opening::new(OpeningKind::Door, 3.0));
        h
    }

    fn suggest(h: &House, kind: PropKind, facing: Option<usize>) -> Vec<Suggestion> {
        let room = h.room_at(0, Vec2::splat(2.0)).unwrap();
        h.suggest(kind, &room, facing, 5)
    }

    #[test]
    fn against_wall_backs_onto_a_face_and_never_blocks_the_door() {
        let h = living();
        let room = h.room_at(0, Vec2::splat(2.0)).unwrap();
        let all = suggest(&h, PropKind::Sofa, None);
        assert_eq!(all.len(), 5);
        for s in &all {
            let c = Prop::new(PropKind::Sofa, s.pos, 0, s.rot).corners();
            assert!(c.iter().all(|q| inside(room.inner, *q)), "{s:?}");
            assert!(!blocks_door(&c, &room), "{s:?}");
            assert!([0.0, 90.0, 180.0, 270.0].contains(&s.rot));
            // dos à 1 cm de la face du mur
            let back =
                c.iter().map(|q| q.dot(front(&Prop::new(PropKind::Sofa, s.pos, 0, s.rot)))).fold(f32::MAX, f32::min);
            let face = match s.rot as i32 {
                0 | 90 => 0.1,
                180 => -3.9,
                _ => -5.9,
            };
            assert!((back - face - GAP).abs() < 0.02, "{s:?} dos {back}");
            assert!(s.why[0].starts_with("contre le mur"));
        }
        assert!(all.windows(2).all(|w| w[0].score >= w[1].score));
        assert!(all.iter().all(|a| all.iter().filter(|b| b.pos.distance(a.pos) < 0.6).count() == 1));
    }

    #[test]
    fn facing_and_partners_rank_first() {
        let mut h = living();
        h.props.push(Prop::new(PropKind::CoffeeTable, Vec2::new(4.5, 2.0), 0, 0.0));
        let best = &suggest(&h, PropKind::Sofa, Some(0))[0];
        let to_table = (h.props[0].pos - best.pos).normalize();
        assert!(front(&Prop::new(PropKind::Sofa, best.pos, 0, best.rot)).dot(to_table) > 0.9, "{best:?}");
        assert!(best.why.iter().any(|w| w.contains("qui va avec")), "{best:?}");
        // la chaise se met devant le bureau, pas sur son côté
        let mut h = living();
        h.props.push(Prop::new(PropKind::Desk, Vec2::new(5.5, 2.0), 0, 270.0));
        let chair = &suggest(&h, PropKind::OfficeChair, Some(0))[0];
        assert!(chair.pos.x < 5.0 && (chair.pos.y - 2.0).abs() < 0.3, "{chair:?}");
        // la table basse se tourne vers le canapé posé
        let mut h = living();
        h.props.push(Prop::new(PropKind::CoffeeTable, Vec2::new(4.5, 2.0), 0, 0.0));
        h.props.push(Prop::new(PropKind::Sofa, Vec2::new(5.45, 2.0), 0, 270.0));
        let table = &suggest(&h, PropKind::CoffeeTable, Some(1))[0];
        let table_prop = Prop::new(PropKind::CoffeeTable, table.pos, 0, table.rot);
        assert!(front(&table_prop).dot((h.props[1].pos - table.pos).normalize()) > 0.95);
        assert_eq!(table.rot % 15.0, 0.0);
        // et laisse 40 cm pour les jambes devant le canapé
        assert!(!overlap(&table_prop.corners(), &ahead(&h.props[1].corners(), front(&h.props[1]), 0.4)), "{table:?}");
    }

    #[test]
    fn tall_furniture_avoids_windows_and_needs_room_in_front() {
        let h = living();
        let best = &suggest(&h, PropKind::Wardrobe, None)[0];
        assert!(!best.why.iter().any(|w| w.contains("fenêtre")), "{best:?}");
        assert!(best.why.iter().any(|w| w.contains("libres devant")), "{best:?}");
        // un réduit de 1 m sur 1 m n'a pas la place d'un lit
        let tiny = House { walls: room_walls(Vec2::ZERO, Vec2::ONE, 0, 3.0, 0.2, WHITE), ..House::default() };
        assert_eq!(suggest_in(&tiny, PropKind::Bed), []);
    }

    fn suggest_in(h: &House, kind: PropKind) -> Vec<Suggestion> {
        let room = h.room_at(0, Vec2::splat(0.5)).unwrap();
        h.suggest(kind, &room, None, 5)
    }

    #[test]
    fn ceiling_on_top_and_occupied_spots() {
        let mut h = living();
        assert_eq!(suggest(&h, PropKind::CeilingLamp, None)[0].pos, Vec2::new(3.0, 2.0));
        assert_eq!(suggest(&h, PropKind::Bonsai, None), []);
        h.props.push(Prop::new(PropKind::Sideboard, Vec2::new(1.0, 1.0), 0, 0.0));
        let on = suggest(&h, PropKind::Bonsai, None);
        assert_eq!((on[0].pos, &on[0].why[0][..9]), (Vec2::ONE, "sur Prop "));
        // aucun canapé proposé par-dessus le buffet
        let sideboard = h.props[0].corners();
        assert!(
            suggest(&h, PropKind::Sofa, None)
                .iter()
                .all(|s| !overlap(&Prop::new(PropKind::Sofa, s.pos, 0, s.rot).corners(), &sideboard))
        );
    }

    #[test]
    fn wall_items_avoid_openings_and_furniture_below() {
        let mut h = living();
        // baignoire contre le mur ouest : rien de mural au-dessus d'elle
        h.props.push(Prop::new(PropKind::Bathtub, Vec2::new(0.5, 2.0), 0, 90.0));
        let solids: Vec<Solid> = h.props.iter().filter_map(Solid::of).collect();
        for kind in [PropKind::Mirror, PropKind::TowelRadiator] {
            let all = suggest(&h, kind, None);
            assert!(!all.is_empty(), "{kind:?}");
            let room = h.room_at(0, Vec2::splat(2.0)).unwrap();
            for s in all {
                let c = Prop::new(kind, s.pos, 0, s.rot).corners();
                assert!(!on_opening(&c, &room, |_| true), "{kind:?} sur une ouverture : {s:?}");
                let me = Solid::of(&Prop::new(kind, s.pos, 0, s.rot)).unwrap();
                assert!(solids.iter().all(|o| !me.clash(o)), "{kind:?} heurte la baignoire : {s:?}");
            }
        }
    }

    #[test]
    fn small_floor_items_stay_near_a_wall_and_never_show_minus_zero() {
        let h = living();
        let lamp = &suggest(&h, PropKind::Lamp, None)[0];
        let (lo, hi) = h.room_at(0, Vec2::splat(2.0)).unwrap().inner;
        assert!((lamp.pos - lo).min(hi - lamp.pos).min_element() < 0.5, "{lamp:?}");
        assert!(
            suggest(&h, PropKind::Sofa, None).iter().all(|s| s.rot.is_sign_positive() && s.pos.x.is_sign_positive())
        );
    }
}
