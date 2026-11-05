//! Ce qu'on vise et ce qu'on en fait : portes, voiture, sièges, lits, ou le sol.

use bevy::prelude::*;

use super::avatar::eyes;
use super::{REACH, Stance, VisitInput, Visitor};
use crate::editor::Editor;
use crate::model::{ElemRef, House, Prop, PropKind, elev, pick};

/// Orientation du personnage pour regarder dans la direction `d` (il regarde vers -Z).
pub(super) fn yaw_facing(d: Vec3) -> f32 {
    (-d.x).atan2(-d.z)
}

/// Orientation pour être allongé la tête vers `d` (la tête part vers +Z une fois couché).
pub(super) fn yaw_head_towards(d: Vec3) -> f32 {
    d.x.atan2(d.z)
}

/// Où s'asseoir (`lie` = faux) ou s'allonger sur un meuble : (posture, point d'appui, orientation).
pub(super) fn furniture_spot(p: &Prop, lie: bool, viewer: Vec3) -> Option<(Stance, Vec3, f32)> {
    let base =
        Transform::from_xyz(p.pos.x, elev(p.level), p.pos.y).with_rotation(Quat::from_rotation_y(p.rot.to_radians()));
    let local_viewer = base.compute_affine().inverse().transform_point3(viewer);
    let side = if local_viewer.x < 0.0 { -1.0 } else { 1.0 };
    let world = |x: f32, y: f32, z: f32| base.transform_point(Vec3::new(x, y, z));
    let dir = |x: f32, z: f32| base.rotation * Vec3::new(x, 0.0, z);
    let sit = |at: Vec3, facing: Vec3| Some((Stance::Sit, at, yaw_facing(facing)));
    let lie_down = |at: Vec3, head: Vec3| Some((Stance::Lie, at, yaw_head_towards(head)));
    match (p.kind, lie) {
        (PropKind::Chair, _) => sit(world(0.0, 0.48, 0.05), dir(0.0, 1.0)),
        (PropKind::Toilet, _) => sit(world(0.0, 0.42, 0.1), dir(0.0, 1.0)),
        (PropKind::Sofa, false) => {
            // la place la plus proche de celui qui s'assoit
            let x = [-0.55, 0.0, 0.55]
                .into_iter()
                .min_by(|a, b| (a - local_viewer.x).abs().total_cmp(&(b - local_viewer.x).abs()))?;
            sit(world(x, 0.45, 0.1), dir(0.0, 1.0))
        }
        (PropKind::Sofa, true) => lie_down(world(0.05, 0.45, 0.05), dir(-1.0, 0.0)),
        (PropKind::Bed, false) => sit(world(side * 0.7, 0.55, 0.2), dir(side, 0.0)),
        (PropKind::Bed, true) => lie_down(world(side * 0.38, 0.55, 0.15), dir(0.0, -1.0)),
        (PropKind::Bathtub, true) => lie_down(world(0.0, 0.12, 0.15), dir(0.0, -1.0)),
        (PropKind::Bathtub, false) => sit(world(0.0, 0.12, 0.3), dir(0.0, -1.0)),
        _ => None,
    }
}

/// Action principale (touche E) : s'allonger pour un lit / une baignoire, sinon s'asseoir.
fn prefers_lying(k: PropKind) -> bool {
    matches!(k, PropKind::Bed | PropKind::Bathtub)
}

impl Visitor {
    /// Ce qu'on vise, les actions possibles, et celle demandée.
    pub(super) fn interact(&mut self, ed: &mut Editor, inp: &VisitInput, cam: &Transform) {
        self.target = aim(ed, cam, eyes(self), self.pos);
        let prop = match self.target {
            Some(ElemRef::Prop(i)) => ed.house.props.get(i).cloned(),
            _ => None,
        };
        self.actions.clear();
        if self.stance == Stance::Stand {
            self.stand_interact(ed, inp, prop.as_ref());
        } else {
            self.actions.push("ZQSD, Espace ou E : se relever");
            if inp.act || inp.sit || inp.lie || inp.jump || inp.mv != Vec2::ZERO {
                self.stance = Stance::Stand;
                self.pos = self.stand_pos;
                self.vel = Vec3::ZERO;
            }
        }
        self.actions.push(if self.third { "V : vue 1re personne" } else { "V : vue 3e personne" });
    }

    fn stand_interact(&mut self, ed: &mut Editor, inp: &VisitInput, prop: Option<&Prop>) {
        let openable = self.target.is_some_and(|t| ed.house.openable(t));
        let open = self.target.and_then(|t| ed.doors.get(&t)).is_some_and(|d| d.1);
        let car_target = prop.is_some_and(|p| p.kind == PropKind::Car);
        let seat = Seat {
            sit: prop.and_then(|p| furniture_spot(p, false, self.pos)),
            lie: prop.and_then(|p| furniture_spot(p, true, self.pos)),
            prefers_lying: prop.is_some_and(|p| prefers_lying(p.kind)),
        };
        self.actions.extend(stand_actions(openable, open, car_target, &seat));
        if let Some(t) = self.target.filter(|_| inp.act && openable) {
            ed.toggle_door(t);
        } else if let (true, Some(ElemRef::Prop(i)), Some(p)) = (inp.act && car_target, self.target, prop) {
            self.board(i, p, ed);
        } else if let Some((stance, at, yaw)) = self.chosen_stance(inp, &seat) {
            self.stand_pos = self.pos;
            self.stance = stance;
            self.anchor = at;
            self.anchor_yaw = yaw;
            self.vel = Vec3::ZERO;
        }
    }

    /// Posture demandée : E sur un meuble, C pour s'asseoir, X pour s'allonger (par terre à défaut).
    fn chosen_stance(&self, inp: &VisitInput, seat: &Seat) -> Option<(Stance, Vec3, f32)> {
        if inp.act {
            if seat.prefers_lying { seat.lie.or(seat.sit) } else { seat.sit.or(seat.lie) }
        } else if inp.sit && self.grounded {
            seat.sit.or(Some((Stance::SitFloor, self.pos, self.body_yaw)))
        } else if inp.lie && self.grounded {
            seat.lie.or(Some((Stance::Lie, self.pos, yaw_head_towards(self.axes().0))))
        } else {
            None
        }
    }
}

/// Places offertes par le meuble visé.
struct Seat {
    sit: Option<(Stance, Vec3, f32)>,
    lie: Option<(Stance, Vec3, f32)>,
    prefers_lying: bool,
}

/// L'élément visé au centre de l'écran, s'il est à portée et qu'on peut s'en servir.
fn aim(ed: &Editor, cam: &Transform, eye: Vec3, viewer: Vec3) -> Option<ElemRef> {
    let (origin, ray) = (cam.translation, cam.rotation * Vec3::NEG_Z);
    pick(&ed.parts, origin, ray)
        .filter(|&(_, dist)| (origin + ray * dist).distance(eye) < REACH)
        .map(|h| h.0)
        .filter(|&e| usable(&ed.house, e, viewer))
}

/// Porte ou placard à ouvrir, voiture à conduire, meuble où s'asseoir ou s'allonger.
fn usable(h: &House, e: ElemRef, viewer: Vec3) -> bool {
    h.openable(e)
        || matches!(e, ElemRef::Prop(i) if h.props.get(i).is_some_and(|p| p.kind == PropKind::Car
            || furniture_spot(p, false, viewer).is_some()
            || furniture_spot(p, true, viewer).is_some()))
}

/// Actions affichées sous le viseur quand on est debout.
fn stand_actions(openable: bool, open: bool, car: bool, seat: &Seat) -> Vec<&'static str> {
    let (sit, lie, primary_lie) = (seat.sit.is_some(), seat.lie.is_some(), seat.prefers_lying);
    let mut v = vec![];
    if car {
        v.push("E : conduire");
    }
    if openable {
        v.push(if open { "E : fermer" } else { "E : ouvrir" });
    }
    if !openable && (sit || lie) {
        v.push(if primary_lie { "E : s'allonger" } else { "E : s'asseoir" });
    }
    v.push(match (sit, primary_lie) {
        (true, true) => "C : s'asseoir au bord",
        (true, false) => "C : s'asseoir ici",
        (false, _) => "C : s'asseoir par terre",
    });
    v.push(match (lie, primary_lie) {
        (true, false) => "X : s'allonger dessus",
        (true, true) => "X : s'allonger",
        (false, _) => "X : s'allonger par terre",
    });
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::demo_house;

    #[test]
    fn furniture_spots_face_the_right_way() {
        let chair = Prop::new(PropKind::Chair, Vec2::ZERO, 0, 0.0);
        let (stance, at, yaw) = furniture_spot(&chair, false, Vec3::new(0.0, 0.0, 2.0)).unwrap();
        assert_eq!(stance, Stance::Sit);
        assert!((at.y - 0.48).abs() < 1e-4);
        // assis, on regarde vers l'avant de la chaise (+z)
        let facing = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
        assert!(facing.z > 0.99, "{facing}");
        let bed = Prop { kind: PropKind::Bed, ..chair.clone() };
        let (stance, _, yaw) = furniture_spot(&bed, true, Vec3::new(2.0, 0.0, 0.0)).unwrap();
        assert_eq!(stance, Stance::Lie);
        // allongé, la tête va vers la tête de lit (-z)
        assert!((Quat::from_rotation_y(yaw) * Vec3::Z).z < -0.99);
        assert!(furniture_spot(&Prop { kind: PropKind::Table, ..chair }, false, Vec3::ZERO).is_none());
    }
    #[test]
    fn standing_actions_depend_on_the_target() {
        let none = Seat { sit: None, lie: None, prefers_lying: false };
        assert_eq!(
            stand_actions(true, true, false, &none),
            ["E : fermer", "C : s'asseoir par terre", "X : s'allonger par terre"]
        );
        assert_eq!(stand_actions(true, false, false, &none)[0], "E : ouvrir");
        let spot = Some((Stance::Sit, Vec3::ZERO, 0.0));
        let bed = Seat { sit: spot, lie: spot, prefers_lying: true };
        assert_eq!(
            stand_actions(false, false, false, &bed),
            ["E : s'allonger", "C : s'asseoir au bord", "X : s'allonger"]
        );
        let sofa = Seat { prefers_lying: false, ..bed };
        assert_eq!(
            stand_actions(false, false, false, &sofa),
            ["E : s'asseoir", "C : s'asseoir ici", "X : s'allonger dessus"]
        );
        assert_eq!(stand_actions(false, false, true, &none)[0], "E : conduire");
        let h = demo_house();
        assert!(usable(&h, ElemRef::Opening(0, 0), Vec3::ZERO), "porte d entrée");
        assert!(!usable(&h, ElemRef::Wall(0), Vec3::ZERO));
    }
}
