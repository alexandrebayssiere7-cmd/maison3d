//! Tests des commandes : validation, refus, aller-retour JSON, schéma figé, propriétés.

use super::*;

#[test]
fn json_commands_build_a_room() {
    let cmds: Vec<Command> = serde_json::from_str(
        r#"[
            {"op": "add_wall", "a": [0, 0], "b": [4, 0], "level": 0, "height": 3, "thick": 0.2, "color": [1, 1, 1], "openings": []},
            {"op": "add_opening", "wall": 0, "opening": {"kind": "Door", "t": 2, "width": 0.9, "height": 2.1, "sill": 0, "color": [1, 1, 1]}},
            {"op": "add_opening", "wall": 0, "opening": {"kind": "Door", "t": 2.5, "width": 0.9, "height": 2.1, "sill": 0, "color": [1, 1, 1]}}
        ]"#,
    )
    .unwrap();
    let mut h = House::default();
    let mut it = cmds.into_iter();
    assert_eq!(h.apply(it.next().unwrap()), Ok(Some(ElemRef::Wall(0))));
    assert_eq!(h.apply(it.next().unwrap()), Ok(Some(ElemRef::Opening(0, 0))));
    // chevauche la première porte
    assert!(h.apply(it.next().unwrap()).is_err());
    assert!(h.apply(Command::Remove { target: ElemRef::Prop(3) }).is_err());
}

#[test]
fn high_level_commands_fill_defaults() {
    let cmds: Vec<Command> = serde_json::from_str(
        r#"[
            {"op": "room", "min": [0, 0], "max": [5, 4], "level": 0},
            {"op": "place_opening", "wall": 0, "kind": "Window", "t": 2.5},
            {"op": "place_prop", "kind": "Bed", "pos": [2, 2], "level": 0},
            {"op": "place_roof", "min": [0, 0], "max": [5, 4], "level": 1, "kind": "Gable"}
        ]"#,
    )
    .unwrap();
    let mut h = House::default();
    for c in cmds {
        h.apply(c).unwrap();
    }
    assert_eq!((h.walls.len(), h.slabs.len(), h.props.len(), h.roofs.len()), (4, 1, 1, 1));
    assert_eq!(h.walls[0].openings.len(), 1);
    assert!(
        h.apply(Command::Room {
            min: Vec2::ONE,
            max: Vec2::ONE,
            level: 0,
            height: None,
            thick: None,
            wall_texture: None,
            floor_texture: None
        })
        .is_err()
    );
}

fn wall(level: i32, len: f32) -> Wall {
    room_walls(Vec2::ZERO, Vec2::new(len, 1.0), level, LEVEL_H, 0.2, WHITE_TEX).remove(0)
}

fn slab(level: i32, min: Vec2, max: Vec2) -> Slab {
    Slab { min, max, level, color: WHITE_TEX, texture: 0 }
}

fn pool(min: Vec2, max: Vec2) -> Pool {
    Pool { min, max, depth: 1.5, color: WHITE_TEX, texture: 0 }
}

fn road(a: Vec2, b: Vec2) -> Road {
    Road { a, b, width: 4.0, color: WHITE_TEX, texture: 0 }
}

/// Une maison avec un mur de 4 m (porte au milieu), un sol, un toit, un lit, une piscine, une route.
fn furnished() -> House {
    let mut h = House::default();
    for c in [
        Command::AddWall(wall(0, 4.0)),
        Command::PlaceOpening { wall: 0, kind: OpeningKind::Door, t: 2.0 },
        Command::AddSlab(slab(0, Vec2::ZERO, Vec2::splat(4.0))),
        Command::PlaceRoof { min: Vec2::ZERO, max: Vec2::splat(4.0), level: 1, kind: RoofKind::Hip, texture: None },
        Command::PlaceProp { kind: PropKind::Bed, pos: Vec2::ONE, level: 0, rot: 0.0 },
        Command::AddPool(pool(Vec2::new(10.0, 10.0), Vec2::new(14.0, 12.0))),
        Command::AddRoad(road(Vec2::new(-20.0, 0.0), Vec2::new(-20.0, 30.0))),
    ] {
        h.apply(c).unwrap();
    }
    h
}

const ALL_REFS: [ElemRef; 7] = [
    ElemRef::Wall(0),
    ElemRef::Opening(0, 0),
    ElemRef::Slab(0),
    ElemRef::Roof(0),
    ElemRef::Prop(0),
    ElemRef::Pool(0),
    ElemRef::Road(0),
];

#[test]
fn invalid_additions_are_refused() {
    let mut h = House::default();
    let refused = [
        Command::AddWall(wall(10, 4.0)),
        Command::AddWall(wall(-2, 4.0)),
        Command::AddWall(wall(0, 0.01)),
        Command::AddSlab(slab(0, Vec2::ONE, Vec2::ZERO)),
        Command::AddSlab(slab(12, Vec2::ZERO, Vec2::ONE)),
        Command::PlaceRoof { min: Vec2::ONE, max: Vec2::ONE, level: 1, kind: RoofKind::Flat, texture: None },
        Command::PlaceRoof { min: Vec2::ZERO, max: Vec2::ONE, level: 10, kind: RoofKind::Flat, texture: None },
        Command::PlaceProp { kind: PropKind::Lamp, pos: Vec2::ZERO, level: -2, rot: 0.0 },
        Command::AddPool(pool(Vec2::new(0.5, 0.0), Vec2::new(3.0, 3.0))),
        Command::AddPool(pool(Vec2::new(3.0, 0.0), Vec2::new(0.0, 3.0))),
        Command::AddRoad(road(Vec2::ONE, Vec2::ONE)),
        Command::PlaceOpening { wall: 0, kind: OpeningKind::Window, t: 1.0 },
        Command::AddStorey { level: 9, min: None, max: None },
    ];
    for c in refused {
        assert!(h.apply(c.clone()).is_err(), "{c:?} aurait dû être refusée");
    }
    assert_eq!(h, House::default());
}

#[test]
fn every_kind_of_element_can_be_added() {
    let h = furnished();
    assert_eq!(h.walls[0].openings[0].kind, OpeningKind::Door);
    assert_eq!(h.roofs[0].overhang, 0.5);
    assert_eq!(h.props[0].color, PropKind::Bed.default_color());
    assert_eq!((h.pools.len(), h.roads.len()), (1, 1));
    // la piscine est creusée, la route terrassée
    assert!(h.ground(Vec2::new(12.0, 11.0)) < -1.0);
}

#[test]
fn opening_too_wide_for_its_wall_is_refused() {
    let mut h = House::default();
    h.apply(Command::AddWall(wall(0, 1.0))).unwrap();
    let err = h.apply(Command::PlaceOpening { wall: 0, kind: OpeningKind::Bay, t: 0.5 }).unwrap_err();
    assert!(matches!(err, EditError::OpeningOutsideWall { kind: OpeningKind::Bay, .. }), "{err}");
    assert!(err.to_string().contains("trop court"), "{err}");
}

#[test]
fn missing_targets_are_refused() {
    let mut h = House::default();
    for target in ALL_REFS {
        for c in [
            Command::Remove { target },
            Command::Move { target, by: Vec2::X },
            Command::Duplicate { target, by: Vec2::X },
            Command::Rotate { target, deg: 90.0 },
            Command::Paint { target, part: 0, color: None, texture: None },
        ] {
            assert!(matches!(h.apply(c), Err(EditError::NotFound(t, 0)) if t == target));
        }
    }
}

#[test]
fn move_and_duplicate_every_kind() {
    let mut h = furnished();
    for target in ALL_REFS {
        let before = h.clone();
        assert_eq!(h.apply(Command::Move { target, by: Vec2::new(0.5, 0.0) }), Ok(Some(target)));
        assert_ne!(h, before, "{target:?} n'a pas bougé");
    }
    assert_eq!(h.walls[0].openings[0].t, 2.5);
    for target in ALL_REFS.into_iter().filter(|t| !matches!(t, ElemRef::Opening(..))) {
        let n = h.count(target);
        let copy = h.apply(Command::Duplicate { target, by: Vec2::new(0.0, 20.0) }).unwrap();
        assert_eq!(copy.map(|c| h.count(c)), Some(n + 1), "{target:?}");
    }
    assert_eq!(
        h.apply(Command::Duplicate { target: ElemRef::Opening(0, 0), by: Vec2::Y }),
        Err(EditError::OpeningNotDuplicable)
    );
    // une copie qui sortirait du terrain est refusée
    let err = h.apply(Command::Duplicate { target: ElemRef::Slab(0), by: Vec2::new(0.0, 150.0) }).unwrap_err();
    assert!(matches!(err, EditError::OutOfWorld(_)), "{err}");
}

/// Chaque refus a un message qui dit quoi corriger (un libellé par variante).
#[test]
fn every_error_has_an_actionable_message() {
    let k = OpeningKind::Door;
    let cases = [
        (EditError::LevelOutOfRange(12), "-1 à 9"),
        (EditError::EmptyRect { min: Vec2::ONE, max: Vec2::ZERO }, "doit être < max"),
        (EditError::NotFound(ElemRef::Prop(4), 0), "aucun élément"),
        (EditError::NotFound(ElemRef::Prop(4), 3), "0 à 2"),
        (EditError::WallTooShort, "5 cm"),
        (EditError::PoolOffGrid, "mètre"),
        (EditError::RoadTooShort, "5 cm"),
        (EditError::WallNotFound(9, 4), "il y a 4 murs"),
        (EditError::OpeningOutsideWall { kind: k, t: 1.0, t_min: 2.0, t_max: 1.0, wall_len: 0.5 }, "trop court"),
        (
            EditError::OpeningOutsideWall { kind: k, t: 0.2, t_min: 0.6, t_max: 3.4, wall_len: 4.0 },
            "entre 0.60 et 3.40",
        ),
        (EditError::OpeningOverlaps { kind: k, t: 2.0, other: 1 }, "l'ouverture 1"),
        (EditError::OpeningTooTall { top: 4.0, wall_height: 3.0 }, "dépasse le mur de 3 m"),
        (EditError::PropOutsideWall { t: 0.2, t_min: 0.6, t_max: 3.4, wall_len: 4.0 }, "entre 0.60 et 3.40"),
        (EditError::SideUnknown(2), "Mur 2"),
        (EditError::OpeningNotDuplicable, "ouvertures"),
        (EditError::NotRotatable, "Prop"),
        (EditError::OutOfWorld(Vec2::new(200.0, 0.0)), "-80 et 80"),
        (EditError::OutOfRange { field: "thick", value: 0.0, min: 0.02, max: 3.0 }, "thick = 0"),
        (EditError::BadColor([2.0, 0.0, 0.0]), "entre 0 et 1"),
        (EditError::UnknownTexture { texture: 99, max: 61 }, "0 (aucune) à 61"),
        (EditError::BadFields("champ inconnu `x`".into()), "champ inconnu"),
        (EditError::TooMany { count: 300, max: 200 }, "200 au plus"),
    ];
    for (err, want) in cases {
        assert!(err.to_string().contains(want), "{err:?} → {err}");
    }
}

#[test]
fn rotate_wraps_and_only_applies_to_props() {
    let mut h = furnished();
    h.apply(Command::Rotate { target: ElemRef::Prop(0), deg: 300.0 }).unwrap();
    h.apply(Command::Rotate { target: ElemRef::Prop(0), deg: 90.0 }).unwrap();
    assert_eq!(h.props[0].rot, 30.0);
    h.apply(Command::Rotate { target: ElemRef::Prop(0), deg: -60.0 }).unwrap();
    assert_eq!(h.props[0].rot, 330.0);
    assert!(h.apply(Command::Rotate { target: ElemRef::Wall(0), deg: 90.0 }).is_err());
}

#[test]
fn paint_sets_color_and_texture_when_given() {
    let mut h = furnished();
    let red = [1.0, 0.0, 0.0];
    h.apply(Command::Paint { target: ElemRef::Slab(0), part: 0, color: Some(red), texture: Some(3) }).unwrap();
    assert_eq!((h.slabs[0].color, h.slabs[0].texture), (red, 3));
    h.apply(Command::Paint { target: ElemRef::Slab(0), part: 0, color: None, texture: Some(5) }).unwrap();
    assert_eq!((h.slabs[0].color, h.slabs[0].texture), (red, 5));
    h.apply(Command::Paint { target: ElemRef::Opening(0, 0), part: 0, color: Some(red), texture: Some(5) }).unwrap();
    assert_eq!((h.walls[0].openings[0].color, h.walls[0].openings[0].texture), (red, 5));
}

#[test]
fn paint_reaches_each_part_and_refuses_missing_ones() {
    let mut h = furnished();
    let i = h.props.len();
    h.apply(json(r#"{"op":"place_prop","kind":"Car","pos":[30,30],"level":0}"#)).unwrap();
    let car = ElemRef::Prop(i);
    let tyres = PropKind::Car.slots().len() - 1;
    h.apply(Command::Paint { target: car, part: tyres, color: Some([1.0, 1.0, 0.0]), texture: None }).unwrap();
    assert_eq!(h.props[i].finish(tyres).color, [1.0, 1.0, 0.0]);
    assert_eq!(h.props[i].color, PropKind::Car.default_color(), "la carrosserie ne change pas");
    let before = h.clone();
    let err = h.apply(Command::Paint { target: car, part: tyres + 1, color: Some([0.0; 3]), texture: None });
    assert!(matches!(err, Err(EditError::NoSuchPart { count, .. }) if count == tyres + 1));
    let err = h.apply(Command::Paint { target: ElemRef::Wall(0), part: 1, color: Some([0.0; 3]), texture: None });
    assert!(matches!(err, Err(EditError::NoSuchPart { count: 1, .. })));
    assert_eq!(h, before, "un refus laisse la maison intacte");
    // la partie est facultative en JSON
    h.apply(json(r#"{"op":"paint","target":{"Opening":[0,0]},"color":[0,0,1]}"#)).unwrap();
    assert_eq!(h.walls[0].openings[0].color, [0.0, 0.0, 1.0]);
}

#[test]
fn finishes_given_by_add_or_set_are_validated() {
    let mut h = furnished();
    let i = h.props.len();
    let sofa = r#"{"op":"add_prop","kind":"Sofa","pos":[30,30],"level":0,"rot":0,"color":[1,1,1],"texture":0,"height":3,"finishes":[{"color":[0,0,0]}]}"#;
    h.apply(json(sofa)).unwrap();
    assert_eq!(h.props[i].finish(1).color, [0.0; 3]);
    let two = serde_json::json!({ "finishes": [{"color": [0, 0, 0]}, {"color": [0, 0, 0]}] });
    let set = |fields: serde_json::Value| Command::Set {
        target: ElemRef::Prop(i),
        fields: serde_json::from_value(fields).unwrap(),
    };
    assert!(matches!(h.apply(set(two)), Err(EditError::TooManyFinishes { count: 2, max: 1 })));
    let bad = serde_json::json!({ "finishes": [{"color": [2, 0, 0]}] });
    assert!(matches!(h.apply(set(bad)), Err(EditError::BadColor(_))));
    let line = serde_json::json!({ "height": 50 });
    assert!(
        matches!(h.apply(set(line)), Err(EditError::OutOfRange { field: "height", .. })),
        "30 m au plus hors tracés"
    );
}

#[test]
fn remove_storey_and_terrain_commands() {
    let mut h = furnished();
    h.apply(Command::Remove { target: ElemRef::Opening(0, 0) }).unwrap();
    assert_eq!(h.walls[0].openings, []);
    h.apply(Command::AddStorey { level: 0, min: None, max: None }).unwrap();
    assert_eq!(h.walls.iter().filter(|w| w.level == 1).count(), 1);
    h.apply(Command::GenerateTerrain { seed: 7, amp: 5.0, scale: 30.0, mountains: true }).unwrap();
    assert!(h.terrain.h.iter().any(|&y| y != 0.0));
    assert_eq!(h.apply(Command::FlattenTerrain), Ok(None));
    assert_eq!(h.terrain.h, Vec::<f32>::new());
}

#[test]
fn every_command_roundtrips_through_json() {
    let h = furnished();
    let cmds = vec![
        Command::Room {
            min: Vec2::ZERO,
            max: Vec2::ONE,
            level: 0,
            height: None,
            thick: None,
            wall_texture: None,
            floor_texture: None,
        },
        Command::PlaceOpening { wall: 0, kind: OpeningKind::Window, t: 1.5 },
        Command::PlaceProp { kind: PropKind::Sofa, pos: Vec2::ONE, level: 1, rot: 45.0 },
        Command::PlaceAgainst { kind: PropKind::Bed, wall: 0, t: 2.0, side: Some(Side::Left) },
        Command::PlaceAgainst { kind: PropKind::Bed, wall: 0, t: 2.0, side: None },
        Command::PlaceRoof { min: Vec2::ZERO, max: Vec2::ONE, level: 1, kind: RoofKind::Shed, texture: None },
        Command::AddWall(h.walls[0].clone()),
        Command::AddSlab(h.slabs[0].clone()),
        Command::AddRoof(h.roofs[0].clone()),
        Command::AddProp(h.props[0].clone()),
        Command::AddPool(h.pools[0].clone()),
        Command::AddRoad(h.roads[0].clone()),
        Command::AddOpening { wall: 0, opening: h.walls[0].openings[0].clone() },
        Command::AddStorey { level: 2, min: None, max: None },
        Command::GenerateTerrain { seed: 3, amp: 2.5, scale: 20.0, mountains: false },
        Command::FlattenTerrain,
        Command::Remove { target: ElemRef::Opening(1, 2) },
        Command::Move { target: ElemRef::Road(0), by: Vec2::new(0.5, -1.0) },
        Command::Duplicate { target: ElemRef::Pool(0), by: Vec2::Y },
        Command::Rotate { target: ElemRef::Prop(0), deg: -90.0 },
        Command::Paint { target: ElemRef::Wall(0), part: 0, color: Some([0.1, 0.2, 0.3]), texture: None },
        Command::PlaceRow { kind: PropKind::Hedge, from: Vec2::ZERO, to: Vec2::X, step: 2.0, level: 0, rot: None },
        Command::Set { target: ElemRef::Prop(0), fields: serde_json::from_str(r#"{"height": 2}"#).unwrap() },
    ];
    for c in cmds {
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(serde_json::from_str::<Command>(&json).unwrap(), c, "{json}");
    }
}

fn json(s: &str) -> Command {
    serde_json::from_str(s).unwrap()
}

#[test]
fn absurd_values_are_refused_with_the_bounds() {
    let mut h = furnished();
    let before = h.clone();
    for (cmd, want) in [
        (r#"{"op":"generate_terrain","seed":1,"amp":8,"scale":0,"mountains":false}"#, "scale"),
        (r#"{"op":"generate_terrain","seed":1,"amp":-3,"scale":30,"mountains":false}"#, "amp"),
        (r#"{"op":"place_prop","kind":"Sofa","pos":[1e39,2],"level":0}"#, "hors du terrain"),
        (r#"{"op":"room","min":[195,0],"max":[200,4],"level":0}"#, "hors du terrain"),
        (
            r#"{"op":"add_wall","a":[0,0],"b":[4,0],"level":0,"height":-3,"thick":0.2,"color":[1,1,1],"openings":[]}"#,
            "height",
        ),
        (
            r#"{"op":"add_wall","a":[0,0],"b":[4,0],"level":0,"height":3,"thick":0,"color":[1,1,1],"openings":[]}"#,
            "thick",
        ),
        (
            r#"{"op":"add_wall","a":[0,0],"b":[4,0],"level":0,"height":3,"thick":0.2,"color":[5,-2,30],"openings":[]}"#,
            "Couleur",
        ),
        (
            r#"{"op":"add_wall","a":[0,0],"b":[4,0],"level":0,"height":3,"thick":0.2,"color":[1,1,1],"openings":[],"texture":999}"#,
            "Texture 999",
        ),
        (r#"{"op":"add_pool","min":[0,0],"max":[4,4],"depth":-2,"color":[1,1,1],"texture":39}"#, "depth"),
        (r#"{"op":"add_road","a":[0,0],"b":[9,0],"width":-4,"color":[1,1,1],"texture":0}"#, "width"),
        (
            r#"{"op":"add_opening","wall":0,"opening":{"kind":"Window","t":2,"width":1,"height":6,"sill":-1,"color":[1,1,1]}}"#,
            "sill",
        ),
        (
            r#"{"op":"add_opening","wall":0,"opening":{"kind":"Window","t":2,"width":1,"height":2.5,"sill":1,"color":[1,1,1]}}"#,
            "trop haute",
        ),
        (r#"{"op":"move","target":{"Slab":0},"by":[500,0]}"#, "by[0]"),
        (r#"{"op":"move","target":{"Opening":[0,0]},"by":[50,0]}"#, "dépasse du mur"),
        (r#"{"op":"paint","target":{"Wall":0},"texture":5000}"#, "Texture 5000"),
        (r#"{"op":"place_row","kind":"Hedge","from":[-70,0],"to":[70,0],"step":0.5,"level":0}"#, "au plus"),
    ] {
        let err = h.apply(json(cmd)).unwrap_err().to_string();
        assert!(err.contains(want), "{cmd} → {err}");
    }
    assert_eq!(h, before, "un refus ne change rien");
}

#[test]
fn unknown_fields_and_typos_are_refused() {
    let typo =
        serde_json::from_str::<Command>(r#"{"op":"place_prop","kind":"Sofa","pos":[2,2],"level":0,"rotation":90}"#);
    assert!(typo.unwrap_err().to_string().contains("rotation"));
    assert!(serde_json::from_str::<Command>(r#"{"op":"room","min":[0,0],"max":[4,4],"level":0,"foo":1}"#).is_err());
    let wall = r#"{"op":"add_wall","a":[0,0],"b":[4,0],"level":0,"height":3,"thick":0.2,"color":[1,1,1],"openings":[],"colour":[1,0,0]}"#;
    assert!(serde_json::from_str::<Command>(wall).is_err());
}

fn against(kind: PropKind, wall: usize, t: f32, side: Option<Side>) -> Command {
    Command::PlaceAgainst { kind, wall, t, side }
}

/// Bornes `[x, z]` de l'emprise au sol d'un objet posé.
fn footprint(p: &Prop) -> (Vec2, Vec2) {
    p.corners().iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), c| (lo.min(*c), hi.max(*c)))
}

#[test]
fn place_against_puts_the_back_on_the_wall_face() {
    let mut h = House::default();
    h.apply(json(r#"{"op":"room","min":[0,0],"max":[5,4],"level":0}"#)).unwrap();
    // mur nord (0, d'ouest en est) : le sol est au sud, l'avant regarde +z
    assert_eq!(h.apply(against(PropKind::Bed, 0, 2.5, None)), Ok(Some(ElemRef::Prop(0))));
    let (lo, hi) = footprint(&h.props[0]);
    assert!(h.props[0].rot.abs() < 1e-3, "rot {}", h.props[0].rot);
    assert!((lo.y - 0.1).abs() < 1e-3 && (f32::midpoint(lo.x, hi.x) - 2.5).abs() < 1e-3, "{lo} {hi}");
    // mur est (1, du nord au sud) : l'avant regarde l'ouest
    h.apply(against(PropKind::Wardrobe, 1, 3.0, None)).unwrap();
    let (lo, hi) = footprint(&h.props[1]);
    assert!((h.props[1].rot - 270.0).abs() < 1e-3 && (hi.x - 4.9).abs() < 1e-3, "{lo} {hi}");
    assert!((f32::midpoint(lo.y, hi.y) - 3.0).abs() < 1e-3);
    assert_eq!(h.warnings(), Vec::<String>::new());
    // côté explicite : dehors, contre le mur sud (2, d'est en ouest), l'avant regarde le sud
    h.apply(against(PropKind::Bench, 2, 2.5, Some(Side::Left))).unwrap();
    let (lo, hi) = footprint(&h.props[2]);
    assert!(h.props[2].rot.abs() < 1e-3 && (lo.y - 4.1).abs() < 1e-3, "{lo} {hi}");
    assert!((f32::midpoint(lo.x, hi.x) - 2.5).abs() < 1e-3, "t mesuré depuis a = [5, 4]");
}

#[test]
fn place_against_refuses_missing_walls_unknown_sides_and_overflow() {
    let mut h = House::default();
    h.apply(json(r#"{"op":"room","min":[0,0],"max":[5,4],"level":0}"#)).unwrap();
    let before = h.clone();
    assert_eq!(h.apply(against(PropKind::Bed, 9, 1.0, None)), Err(EditError::WallNotFound(9, 4)));
    let err = h.apply(against(PropKind::Bed, 0, 0.5, None)).unwrap_err();
    assert!(matches!(err, EditError::PropOutsideWall { t_min, t_max, .. } if t_min < t_max), "{err}");
    assert!(err.to_string().contains("t doit être entre 0.95 et 4.05"), "{err}");
    assert_eq!(h, before);
    // un sol de l'autre côté aussi (cloison) : le côté doit être donné
    h.apply(Command::AddSlab(slab(0, Vec2::new(0.0, -3.0), Vec2::new(5.0, 0.0)))).unwrap();
    assert_eq!(h.apply(against(PropKind::Bed, 0, 2.5, None)), Err(EditError::SideUnknown(0)));
    assert!(h.apply(against(PropKind::Bed, 0, 2.5, Some(Side::Left))).is_ok());
    // aucun sol à l'étage du mur
    h.apply(Command::AddWall(wall(1, 4.0))).unwrap();
    assert_eq!(h.apply(against(PropKind::Desk, 4, 2.0, None)), Err(EditError::SideUnknown(4)));
    // mur plus court que l'objet
    h.apply(json(r#"{"op":"room","min":[10,0],"max":[11,1],"level":0}"#)).unwrap();
    let err = h.apply(against(PropKind::Bed, 5, 0.5, None)).unwrap_err();
    assert!(matches!(err, EditError::PropOutsideWall { t_min, t_max, .. } if t_min > t_max), "{err}");
}

#[test]
fn opening_errors_say_why() {
    let mut h = furnished();
    let err = h.apply(Command::PlaceOpening { wall: 0, kind: OpeningKind::Window, t: 2.2 }).unwrap_err();
    assert!(matches!(err, EditError::OpeningOverlaps { other: 0, .. }), "{err}");
    let err = h.apply(Command::PlaceOpening { wall: 0, kind: OpeningKind::Door, t: 0.5 }).unwrap_err();
    assert!(matches!(err, EditError::OpeningOutsideWall { .. }), "{err}");
    assert!(err.to_string().contains("entre 0.60 et 3.40"), "{err}");
    let err = h.apply(Command::PlaceOpening { wall: 7, kind: OpeningKind::Door, t: 1.0 }).unwrap_err();
    assert_eq!(err, EditError::WallNotFound(7, 1));
}

#[test]
fn set_changes_fields_in_place_and_validates() {
    let mut h = furnished();
    let set = |target, fields: &str| Command::Set { target, fields: serde_json::from_str(fields).unwrap() };
    assert_eq!(h.apply(set(ElemRef::Prop(0), r#"{"height": 4.5, "rot": 90}"#)), Ok(Some(ElemRef::Prop(0))));
    assert_eq!((h.props[0].height, h.props[0].rot), (4.5, 90.0));
    h.apply(set(ElemRef::Opening(0, 0), r#"{"width": 1.2}"#)).unwrap();
    assert_eq!(h.walls[0].openings[0].width, 1.2);
    h.apply(set(ElemRef::Road(0), r#"{"width": 6}"#)).unwrap();
    assert_eq!(h.roads[0].width, 6.0);
    h.apply(set(ElemRef::Roof(0), r#"{"overhang": 0.2, "kind": "Flat"}"#)).unwrap();
    assert_eq!((h.roofs[0].overhang, h.roofs[0].kind), (0.2, RoofKind::Flat));
    // une piscine agrandie est recreusée : son nouveau coin est au fond
    h.apply(set(ElemRef::Pool(0), r#"{"max": [16, 12], "depth": 2}"#)).unwrap();
    assert_eq!(h.pools[0].depth, 2.0);
    assert!(h.ground(Vec2::new(15.5, 11.0)) < -1.0);
    let before = h.clone();
    for (target, bad) in [
        (ElemRef::Wall(0), r#"{"thick": 0}"#),
        (ElemRef::Wall(0), r#"{"thickness": 0.3}"#),
        (ElemRef::Slab(0), r#"{"min": "nord"}"#),
        (ElemRef::Opening(0, 0), r#"{"t": 9}"#),
    ] {
        assert!(h.apply(set(target, bad)).is_err(), "{bad}");
    }
    assert_eq!(h, before);
}

#[test]
fn room_options_flat_roof_and_rows() {
    let mut h = House::default();
    h.apply(json(r#"{"op":"room","min":[0,0],"max":[4,4],"level":0,"height":6,"thick":0.6,"wall_texture":7}"#))
        .unwrap();
    assert!(h.walls.iter().all(|w| (w.height, w.thick, w.texture) == (6.0, 0.6, 7)));
    h.apply(json(r#"{"op":"place_roof","min":[0,0],"max":[4,4],"level":2,"kind":"Flat"}"#)).unwrap();
    assert_eq!(h.roofs[0].texture, find("Zinc"));
    let last = h.apply(json(r#"{"op":"place_row","kind":"Hedge","from":[0,-3],"to":[10,-3],"step":2,"level":0}"#));
    assert_eq!(last, Ok(Some(ElemRef::Prop(5))));
    assert!(h.props.iter().all(|p| p.rot.abs() < 1e-4), "rangée est-ouest : axe des haies le long de x");
    h.apply(json(r#"{"op":"place_row","kind":"Fence","from":[0,0],"to":[0,4],"step":2,"level":0}"#)).unwrap();
    assert!((h.props[6].rot - 270.0).abs() < 1e-3, "rangée vers le sud : {}", h.props[6].rot);
    h.apply(json(r#"{"op":"place_prop","kind":"Sofa","pos":[2,2],"level":0,"rot":1000}"#)).unwrap();
    assert_eq!(h.props.last().unwrap().rot, 280.0);
}

#[test]
fn building_after_the_terrain_flattens_it_around() {
    let mut h = House::default();
    h.apply(Command::GenerateTerrain { seed: 7, amp: 10.0, scale: 30.0, mountains: true }).unwrap();
    let spot = Vec2::new(-51.0, 28.0);
    h.apply(json(r#"{"op":"room","min":[-55,25],"max":[-47,31],"level":0}"#)).unwrap();
    assert_eq!(h.terrain.height(spot), 0.0);
    assert!(h.warnings().iter().all(|w| !w.contains("enterré")), "{:?}", h.warnings());
}

#[test]
fn add_storey_can_target_one_building() {
    let mut h = House::default();
    h.apply(json(r#"{"op":"room","min":[0,0],"max":[4,4],"level":0}"#)).unwrap();
    h.apply(json(r#"{"op":"room","min":[10,10],"max":[14,14],"level":0}"#)).unwrap();
    h.apply(json(r#"{"op":"add_storey","level":0,"min":[0,0],"max":[4,4]}"#)).unwrap();
    assert_eq!(h.walls.iter().filter(|w| w.level == 1).count(), 4);
    assert_eq!(h.slabs.iter().filter(|s| s.level == 1).count(), 1);
}

/// Schéma figé : les agents en dépendent. Après un changement voulu : `UPDATE_SCHEMA=1 cargo test schema_is_frozen`.
#[test]
fn schema_is_frozen() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/commands.schema.json");
    // même texte que `maison3d schema`
    let schema = serde_json::to_string_pretty(&schemars::schema_for!(Vec<Command>)).unwrap() + "\n";
    if std::env::var_os("UPDATE_SCHEMA").is_some() {
        std::fs::write(path, &schema).unwrap();
    }
    let frozen = std::fs::read_to_string(path).unwrap_or_default();
    assert!(frozen == schema, "schéma des commandes modifié : relancer avec UPDATE_SCHEMA=1 si c'est voulu");
}

mod props {
    use super::*;
    use proptest::prelude::*;

    fn v2() -> impl Strategy<Value = Vec2> {
        (-20.0f32..20.0, -20.0f32..20.0).prop_map(|(x, y)| Vec2::new(x, y))
    }

    /// Coins tantôt quelconques, tantôt entiers (piscines).
    fn corner() -> impl Strategy<Value = Vec2> {
        prop_oneof![v2(), v2().prop_map(Vec2::round)]
    }

    fn target() -> impl Strategy<Value = ElemRef> {
        prop_oneof![
            (0..6usize).prop_map(ElemRef::Wall),
            (0..6usize, 0..3usize).prop_map(|(w, o)| ElemRef::Opening(w, o)),
            (0..3usize).prop_map(ElemRef::Slab),
            (0..3usize).prop_map(ElemRef::Roof),
            (0..3usize).prop_map(ElemRef::Prop),
            (0..3usize).prop_map(ElemRef::Pool),
            (0..3usize).prop_map(ElemRef::Road),
        ]
    }

    fn command() -> impl Strategy<Value = Command> {
        let level = -3..12i32;
        prop_oneof![
            (corner(), corner(), level.clone()).prop_map(|(min, max, level)| Command::Room {
                min,
                max,
                level,
                height: None,
                thick: None,
                wall_texture: None,
                floor_texture: None
            }),
            (0..6usize, prop::sample::select(&OpeningKind::ALL[..]), -1.0f32..6.0)
                .prop_map(|(wall, kind, t)| Command::PlaceOpening { wall, kind, t }),
            (prop::sample::select(&PropKind::ALL[..]), v2(), level.clone(), -400.0f32..400.0)
                .prop_map(|(kind, pos, level, rot)| Command::PlaceProp { kind, pos, level, rot }),
            (
                prop::sample::select(&PropKind::ALL[..]),
                0..6usize,
                -1.0f32..6.0,
                prop::option::of(prop::sample::select(&[Side::Left, Side::Right][..]))
            )
                .prop_map(|(kind, wall, t, side)| Command::PlaceAgainst { kind, wall, t, side }),
            (corner(), corner(), level.clone(), prop::sample::select(&RoofKind::ALL[..]))
                .prop_map(|(min, max, level, kind)| Command::PlaceRoof { min, max, level, kind, texture: None }),
            (v2(), v2(), level.clone()).prop_map(|(a, b, level)| {
                let mut w = wall(level, 1.0);
                (w.a, w.b) = (a, b);
                Command::AddWall(w)
            }),
            (corner(), corner()).prop_map(|(min, max)| Command::AddPool(pool(min, max))),
            (v2(), v2()).prop_map(|(a, b)| Command::AddRoad(road(a, b))),
            level.clone().prop_map(|level| Command::AddStorey { level, min: None, max: None }),
            (any::<u32>(), 0.0f32..10.0, 5.0f32..50.0, any::<bool>())
                .prop_map(|(seed, amp, scale, mountains)| Command::GenerateTerrain { seed, amp, scale, mountains }),
            Just(Command::FlattenTerrain),
            target().prop_map(|target| Command::Remove { target }),
            (target(), v2()).prop_map(|(target, by)| Command::Move { target, by }),
            (target(), v2()).prop_map(|(target, by)| Command::Duplicate { target, by }),
            (target(), -400.0f32..400.0).prop_map(|(target, deg)| Command::Rotate { target, deg }),
            (target(), -2.0f32..8.0).prop_map(|(target, v)| Command::Set {
                target,
                fields: serde_json::from_value(serde_json::json!({ "height": v })).unwrap(),
            }),
            (prop::sample::select(&PropKind::ALL[..]), v2(), v2(), 0.5f32..5.0, level)
                .prop_map(|(kind, from, to, step, level)| Command::PlaceRow { kind, from, to, step, level, rot: None }),
            (target(), any::<bool>(), 0..10usize, 0..8usize).prop_map(|(target, c, t, part)| Command::Paint {
                target,
                part,
                color: c.then_some([0.5, 0.2, 0.1]),
                texture: Some(t),
            }),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn random_commands_never_panic_and_errors_change_nothing(cmds in prop::collection::vec(command(), 0..30)) {
            let mut h = House::default();
            for c in cmds {
                let before = h.clone();
                if h.apply(c.clone()).is_err() {
                    prop_assert_eq!(&h, &before, "{:?} a modifié la maison malgré l'erreur", c);
                }
            }
        }
    }
}

#[test]
fn schema_lists_every_command() {
    let schema = serde_json::to_string(&schemars::schema_for!(Vec<Command>)).unwrap();
    for op in ["room", "place_opening", "add_pool", "generate_terrain", "flatten_terrain", "paint"] {
        assert!(schema.contains(op), "{op} absent du schéma");
    }
}
