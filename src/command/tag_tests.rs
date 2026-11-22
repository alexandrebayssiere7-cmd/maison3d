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
