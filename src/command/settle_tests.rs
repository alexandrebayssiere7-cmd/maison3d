//! Le relief suit un élément ajouté, déplacé, copié ou modifié (`House::settle`).

use super::*;

/// Relief marqué, sans bâtiment pour l'aplanir.
fn hilly() -> House {
    let mut h = House::default();
    h.apply(Command::GenerateTerrain { seed: 3, amp: 20.0, scale: 30.0, mountains: false }).unwrap();
    h
}

fn road(a: Vec2, b: Vec2) -> Road {
    Road { a, b, width: 4.0, color: WHITE_TEX, texture: 0 }
}

/// Écart entre le relief au milieu de la route et sa chaussée, droite qui relie ses deux bouts
/// sur le relief (comme `road_top`).
fn road_gap(h: &House, r: &Road) -> f32 {
    let (ya, yb) = (h.terrain.height(r.a), h.terrain.height(r.b));
    (h.terrain.height((r.a + r.b) / 2.0) - f32::midpoint(ya, yb)).abs()
}

const BY: Vec2 = Vec2::new(30.0, 0.0);

fn moved(r: &Road) -> Road {
    road(r.a + BY, r.b + BY)
}

#[test]
fn moved_road_is_graded_at_its_new_place() {
    let mut h = hilly();
    let r = road(Vec2::new(-40.0, -20.0), Vec2::new(-20.0, -20.0));
    assert!(road_gap(&h, &moved(&r)) > 0.5, "le relief doit croiser la route déplacée");
    h.apply(Command::AddRoad(r.clone())).unwrap();
    h.apply(Command::Move { target: ElemRef::Road(0), by: BY }).unwrap();
    assert!(road_gap(&h, &h.roads[0]) < 0.05, "écart {}", road_gap(&h, &h.roads[0]));
}

#[test]
fn duplicated_road_is_graded() {
    let mut h = hilly();
    let r = road(Vec2::new(-40.0, -20.0), Vec2::new(-20.0, -20.0));
    assert!(road_gap(&h, &moved(&r)) > 0.5);
    h.apply(Command::AddRoad(r)).unwrap();
    let copy = h.apply(Command::Duplicate { target: ElemRef::Road(0), by: BY }).unwrap();
    assert_eq!(copy, Some(ElemRef::Road(1)));
    assert!(road_gap(&h, &h.roads[1]) < 0.05, "écart {}", road_gap(&h, &h.roads[1]));
}

#[test]
fn moved_ground_floor_wall_flattens_the_relief_under_it() {
    let mut h = hilly();
    let w = Wall {
        a: Vec2::new(-70.0, 20.0),
        b: Vec2::new(-64.0, 20.0),
        level: 0,
        height: LEVEL_H,
        thick: 0.2,
        color: WHITE_TEX,
        openings: vec![],
        texture: 0,
    };
    let mid = (w.a + w.b) / 2.0 + BY;
    assert!(h.terrain.height(mid).abs() > 0.5, "le relief doit monter sous le mur déplacé");
    h.apply(Command::AddWall(w.clone())).unwrap();
    h.apply(Command::Move { target: ElemRef::Wall(0), by: BY }).unwrap();
    assert_eq!(h.terrain.height(mid), 0.0);
    // un mur d'étage, une ouverture ou un objet déplacés ne touchent pas au relief
    let before = h.terrain.h.clone();
    h.apply(Command::AddWall(Wall { level: 1, ..w })).unwrap();
    h.apply(Command::PlaceOpening { wall: 1, kind: OpeningKind::Window, t: 3.0 }).unwrap();
    h.apply(Command::PlaceProp { kind: PropKind::Sofa, pos: Vec2::ZERO, level: 0, rot: 0.0, scale: None }).unwrap();
    for target in [ElemRef::Wall(1), ElemRef::Opening(1, 0), ElemRef::Prop(0)] {
        h.apply(Command::Move { target, by: Vec2::new(0.5, 0.0) }).unwrap();
    }
    assert_eq!(h.terrain.h, before);
}

#[test]
fn moved_slab_and_pool_settle_too() {
    let mut h = hilly();
    let spot = Vec2::splat(-7.0);
    assert!(h.terrain.height(spot).abs() > 0.5);
    h.apply(Command::AddSlab(Slab {
        min: Vec2::splat(-40.0),
        max: Vec2::splat(-34.0),
        level: 0,
        color: WHITE_TEX,
        texture: 0,
    }))
    .unwrap();
    h.apply(Command::Move { target: ElemRef::Slab(0), by: Vec2::splat(30.0) }).unwrap();
    assert_eq!(h.terrain.height(spot), 0.0);
    let p = Pool { min: Vec2::new(30.0, -40.0), max: Vec2::new(34.0, -37.0), depth: 1.5, color: WHITE_TEX, texture: 0 };
    let by = Vec2::new(0.0, 20.0);
    assert!((h.terrain.height(p.min + by) - h.terrain.height(p.max + by)).abs() > 0.1, "en pente avant la copie");
    h.apply(Command::AddPool(p)).unwrap();
    h.apply(Command::Duplicate { target: ElemRef::Pool(0), by }).unwrap();
    let q = &h.pools[1];
    assert!((h.terrain.height(q.min) - h.terrain.height(q.max)).abs() < 1e-4, "le fond de la copie est plat");
}

#[test]
fn set_flattens_under_a_ground_floor_wall() {
    let mut h = hilly();
    let w = Wall {
        a: Vec2::new(-40.0, 20.0),
        b: Vec2::new(-34.0, 20.0),
        level: 1,
        height: LEVEL_H,
        thick: 0.2,
        color: WHITE_TEX,
        openings: vec![],
        texture: 0,
    };
    let mid = (w.a + w.b) / 2.0;
    h.apply(Command::AddWall(w)).unwrap();
    assert!(h.terrain.height(mid).abs() > 0.5, "un mur d'étage laisse le relief");
    h.apply(Command::Set { target: ElemRef::Wall(0), fields: fields(serde_json::json!({"level": 0})) }).unwrap();
    assert_eq!(h.terrain.height(mid), 0.0);
}

fn fields(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    let serde_json::Value::Object(m) = v else { unreachable!() };
    m
}

/// Une pièce au rez-de-chaussée sur le relief, avec une route et une piscine à côté.
fn built() -> House {
    let mut h = hilly();
    for c in [
        r#"{"op":"room","min":[-40,17],"max":[-34,23],"level":0}"#,
        r#"{"op":"add_road","a":[-40,-20],"b":[-20,-20],"width":4,"color":[1,1,1],"texture":0}"#,
        r#"{"op":"add_pool","min":[30,-40],"max":[34,-37],"depth":1.5,"color":[1,1,1],"texture":0}"#,
    ] {
        h.apply(serde_json::from_str(c).unwrap()).unwrap();
    }
    h
}

#[test]
fn set_without_moving_leaves_the_relief_alone() {
    let mut h = built();
    let before = h.terrain.h.clone();
    // 4 couleurs de murs successives : le relief ne s'aplanit pas un peu plus à chaque fois
    for i in 0..4 {
        let color = fields(serde_json::json!({"color": [1, 0, 0]}));
        h.apply(Command::Set { target: ElemRef::Wall(i), fields: color }).unwrap();
    }
    for target in [ElemRef::Slab(0), ElemRef::Road(0), ElemRef::Pool(0)] {
        h.apply(Command::Set { target, fields: fields(serde_json::json!({"texture": 3})) }).unwrap();
    }
    assert!(h.terrain.h == before, "le relief a changé");
}

#[test]
fn null_or_refused_moves_leave_the_relief_alone() {
    let mut h = built();
    let before = h.terrain.h.clone();
    for target in [ElemRef::Wall(0), ElemRef::Slab(0), ElemRef::Road(0), ElemRef::Pool(0)] {
        h.apply(Command::Move { target, by: Vec2::ZERO }).unwrap();
        // hors du terrain : refusé, rien ne bouge
        let err = h.apply(Command::Move { target, by: Vec2::new(150.0, 0.0) }).unwrap_err();
        assert!(matches!(err, EditError::OutOfWorld(_)), "{err:?}");
    }
    // une piscine se déplace par mètres entiers : 0,3 m ne la bouge pas
    h.apply(Command::Move { target: ElemRef::Pool(0), by: Vec2::new(0.3, 0.0) }).unwrap();
    assert!(h.terrain.h == before, "le relief a changé");
}

#[test]
fn pool_floor_sits_at_its_centre_height() {
    let mut h = hilly();
    let p = Pool { min: Vec2::new(30.0, -40.0), max: Vec2::new(34.0, -37.0), depth: 1.5, color: WHITE_TEX, texture: 0 };
    let by = Vec2::new(0.0, 20.0);
    let centre = |h: &House, p: &Pool| h.terrain.height((p.min + p.max) / 2.0);
    let (here, there) = (centre(&h, &p), h.terrain.height((p.min + p.max) / 2.0 + by));
    // en pente au départ, ici et là-bas : sinon le test ne prouverait rien
    assert!((h.terrain.height(p.min) - here).abs() > 0.1);
    assert!((h.terrain.height(p.min + by) - there).abs() > 0.1);
    h.apply(Command::AddPool(p.clone())).unwrap();
    assert_eq!((h.terrain.height(p.min), h.terrain.height(p.max)), (here, here));
    // déplacée ou copiée : même règle à sa nouvelle place
    h.apply(Command::Duplicate { target: ElemRef::Pool(0), by }).unwrap();
    let q = h.pools[1].clone();
    assert_eq!((h.terrain.height(q.min), h.terrain.height(q.max)), (there, there));
}

#[test]
fn swapping_wall_ends_is_not_a_move() {
    let mut h = built();
    let w = h.walls[0].clone();
    let before = h.footprint(ElemRef::Wall(0));
    h.walls[0] = Wall { a: w.b, b: w.a, ..w };
    assert_eq!(h.footprint(ElemRef::Wall(0)), before);
    assert_eq!(h.footprint(ElemRef::Prop(0)), None);
}
