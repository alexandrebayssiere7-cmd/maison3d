//! Tests des étiquettes : validation, `set`, cibles `{"tag": …}`, propagation.

use super::*;
use crate::model::demo_house;

fn json(s: &str) -> Command {
    serde_json::from_str(s).unwrap()
}

#[test]
fn bad_tags_are_refused_and_leave_the_house_intact() {
    let mut h = demo_house();
    let before = h.clone();
    let long = "x".repeat(41);
    for tag in ["", "   ", long.as_str()] {
        let mut p = Prop::new(PropKind::Sofa, Vec2::ONE, 0, 0.0);
        p.tag = Some(tag.into());
        assert_eq!(h.apply(Command::AddProp(p)), Err(EditError::BadTag(tag.into())));
        let mut w = h.walls[0].clone();
        w.tag = Some(tag.into());
        assert!(matches!(h.apply(Command::AddWall(w)), Err(EditError::BadTag(_))));
    }
    let cmd = format!(r#"{{"op":"set","target":{{"Road":0}},"fields":{{"tag":"{long}"}}}}"#);
    assert!(matches!(h.apply(json(&cmd)), Err(EditError::BadTag(_))));
    assert_eq!(h, before);
    let mut p = Prop::new(PropKind::Sofa, Vec2::ONE, 0, 0.0);
    p.tag = Some("x".repeat(40));
    assert!(h.apply(Command::AddProp(p)).is_ok(), "40 caractères : accepté");
}

#[test]
fn set_changes_then_clears_a_tag() {
    let mut h = demo_house();
    h.apply(json(r#"{"op":"set","target":{"Slab":0},"fields":{"tag":"rdc"}}"#)).unwrap();
    assert_eq!(h.slabs[0].tag.as_deref(), Some("rdc"));
    h.apply(json(r#"{"op":"set","target":{"Slab":0},"fields":{"tag":null}}"#)).unwrap();
    assert_eq!(h.slabs[0].tag, None);
}

/// Le monde de démo, plus 5 lampes dont les 1, 2 et 4 (dans l'ordre de pose) portent « haie ».
fn with_tagged_lamps() -> (House, usize) {
    let mut h = demo_house();
    let first = h.props.len();
    for k in 0..5 {
        let mut p = Prop::new(PropKind::Lamp, Vec2::new(k as f32, -30.0), 0, 0.0);
        p.tag = [1, 2, 4].contains(&k).then(|| "haie".into());
        h.apply(Command::AddProp(p)).unwrap();
    }
    (h, first)
}

fn haie() -> Target {
    Target::Tag { tag: "haie".into() }
}

#[test]
fn targets_read_from_json_by_index_or_by_tag() {
    let c = json(r#"{"op":"remove","target":{"tag":"haie"}}"#);
    assert_eq!(c, Command::Remove { target: haie() });
    let c = json(r#"{"op":"paint","target":{"Wall":0},"targets":[{"tag":"haie"}],"color":[0,0,0]}"#);
    let Command::Paint { target, targets, .. } = &c else { panic!("{c:?}") };
    assert_eq!((target, targets), (&Some(ElemRef::Wall(0).into()), &vec![haie()]));
    for c in [
        c.clone(),
        Command::Move { target: haie(), by: Vec2::X },
        Command::Set { target: haie(), fields: serde_json::Map::new() },
    ] {
        let text = serde_json::to_string(&c).unwrap();
        assert_eq!(serde_json::from_str::<Command>(&text).unwrap(), c, "{text}");
    }
}

#[test]
fn a_target_with_an_extra_key_is_refused() {
    for target in [r#"{"tag":"haie","Wall":0}"#, r#"{"Wall":0,"tag":"haie"}"#, r#"{"tga":"haie"}"#] {
        let text = format!(r#"{{"op":"remove","target":{target}}}"#);
        assert!(serde_json::from_str::<Command>(&text).is_err(), "{text}");
    }
}

#[test]
fn remove_by_tag_deletes_interleaved_elements_only() {
    let (mut h, first) = with_tagged_lamps();
    let keep = [h.props[first].clone(), h.props[first + 3].clone()];
    assert_eq!(h.apply(Command::Remove { target: haie() }), Ok(None));
    assert_eq!(h.props[first..], keep);
    assert!(h.props.iter().all(|p| p.tag.is_none()));
}

#[test]
fn remove_by_tag_spans_several_lists() {
    let mut h = demo_house();
    h.walls[1].tag = Some("aile".into());
    h.walls[3].tag = Some("aile".into());
    h.roads[0].tag = Some("aile".into());
    let (walls, roads) = (h.walls.len(), h.roads.len());
    let kept = h.walls[2].clone();
    h.apply(Command::Remove { target: Target::Tag { tag: "aile".into() } }).unwrap();
    assert_eq!((h.walls.len(), h.roads.len()), (walls - 2, roads - 1));
    assert_eq!(h.walls[1], kept, "le mur 2 a pris la place du 1");
}

#[test]
fn move_paint_and_set_by_tag_reach_every_tagged_element() {
    let (mut h, first) = with_tagged_lamps();
    let tagged = [first + 1, first + 2, first + 4];
    let before = h.clone();
    assert_eq!(h.apply(Command::Move { target: haie(), by: Vec2::Y }), Ok(Some(ElemRef::Prop(first + 4))));
    for i in tagged {
        assert_eq!(h.props[i].pos, before.props[i].pos + Vec2::Y);
    }
    assert_eq!(h.props[first + 3].pos, before.props[first + 3].pos, "sans étiquette : pas déplacée");

    let paint = json(r#"{"op":"paint","targets":[{"tag":"haie"}],"color":[0.5,0.5,0.5]}"#);
    assert_eq!(h.apply(paint), Ok(Some(ElemRef::Prop(first + 4))));
    assert!(tagged.iter().all(|&i| h.props[i].color.iter().all(|&c| (c - 0.5).abs() < f32::EPSILON)));
    assert_ne!(h.props[first].color, [0.5; 3]);

    h.apply(json(r#"{"op":"set","target":{"tag":"haie"},"fields":{"scale":2}}"#)).unwrap();
    assert!(tagged.iter().all(|&i| (h.props[i].scale - 2.0).abs() < f32::EPSILON));
    assert!((h.props[first + 3].scale - 1.0).abs() < f32::EPSILON);
}

#[test]
fn an_unknown_tag_is_refused_by_every_command() {
    let (mut h, _) = with_tagged_lamps();
    let before = h.clone();
    let none = || Target::Tag { tag: "Haie".into() };
    for c in [
        Command::Remove { target: none() },
        Command::Move { target: none(), by: Vec2::X },
        Command::Paint { target: Some(haie()), targets: vec![none()], part: 0, color: None, texture: Some(1) },
        Command::Set { target: none(), fields: serde_json::Map::new() },
    ] {
        assert_eq!(h.apply(c), Err(EditError::NoSuchTag("Haie".into())), "casse comprise");
    }
    assert_eq!(h, before);
}

#[test]
fn a_failure_on_one_tagged_element_leaves_the_house_intact() {
    let mut h = demo_house();
    h.walls[0].tag = Some("rdc".into());
    h.slabs[0].tag = Some("rdc".into());
    let before = h.clone();
    // `thick` n'existe que pour le mur : le sol refuse le champ, le mur déjà modifié est restauré
    let err = h.apply(json(r#"{"op":"set","target":{"tag":"rdc"},"fields":{"thick":0.3}}"#));
    assert!(matches!(err, Err(EditError::BadFields(_))), "{err:?}");
    assert_eq!(h, before);
    let (mut h, _) = with_tagged_lamps();
    let before = h.clone();
    // la dernière lampe (x = 4) sort du terrain, les deux premières avaient déjà bougé
    let err = h.apply(Command::Move { target: haie(), by: Vec2::new(77.0, 0.0) });
    assert!(matches!(err, Err(EditError::OutOfWorld(_))), "{err:?}");
    assert_eq!(h, before);
}
