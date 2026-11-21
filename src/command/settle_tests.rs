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
    let (ya, yb) = h.terrain.road_ends(r.a, r.b);
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
fn pool_floor_sits_at_the_vertex_nearest_its_centre() {
    let mut h = hilly();
    let p = Pool { min: Vec2::new(30.0, -40.0), max: Vec2::new(34.0, -37.0), depth: 1.5, color: WHITE_TEX, texture: 0 };
    let by = Vec2::new(0.0, 20.0);
    let centre = (p.min + p.max) / 2.0;
    let (here, there) = (h.terrain.at_anchor(centre), h.terrain.at_anchor(centre + by));
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

#[test]
fn back_and_forth_moves_flatten_only_once() {
    let mut h = built();
    let step = |h: &mut House, dx: f32| h.apply(Command::Move { target: ElemRef::Wall(0), by: Vec2::new(dx, 0.0) });
    step(&mut h, 0.5).unwrap();
    step(&mut h, -0.5).unwrap();
    let once = h.terrain.h.clone();
    for _ in 0..10 {
        step(&mut h, 0.5).unwrap();
        step(&mut h, -0.5).unwrap();
    }
    assert!(h.terrain.h == once, "chaque aller-retour aplanit un peu plus");
}

#[test]
fn same_building_twice_flattens_like_once() {
    let room = r#"{"op":"room","min":[-40,17],"max":[-34,23],"level":0}"#;
    let mut once = hilly();
    once.apply(serde_json::from_str(room).unwrap()).unwrap();
    let mut twice = once.clone();
    twice.apply(serde_json::from_str(room).unwrap()).unwrap();
    assert!(twice.terrain == once.terrain, "deux fois la même pièce");
    // copies des murs sur place, mur aux extrémités inversées : rien de plus
    let mut copies = once.clone();
    for i in 0..4 {
        copies.apply(Command::Duplicate { target: ElemRef::Wall(i), by: Vec2::ZERO }).unwrap();
    }
    let w = &copies.walls[0];
    let swap = fields(serde_json::json!({"a": [w.b.x, w.b.y], "b": [w.a.x, w.a.y]}));
    copies.apply(Command::Set { target: ElemRef::Wall(0), fields: swap }).unwrap();
    assert!(copies.terrain == once.terrain, "copies sur place");
    let ten_m = Vec2::new(-24.0, 20.0);
    assert!(once.terrain.height(ten_m).abs() > 0.5, "il reste du relief à 10 m de la maison");
}

#[test]
fn moving_a_building_leaves_roads_where_they_are() {
    let mut h = hilly();
    let r = Road { a: Vec2::new(-40.3, -20.6), b: Vec2::new(-20.2, -19.7), width: 4.0, color: WHITE_TEX, texture: 0 };
    h.apply(Command::AddRoad(r)).unwrap();
    h.apply(serde_json::from_str(r#"{"op":"room","min":[40,40],"max":[46,46],"level":0}"#).unwrap()).unwrap();
    let before = h.terrain.clone();
    for k in 0..40 {
        let dx = if k % 2 == 0 { 0.5 } else { -0.5 };
        h.apply(Command::Move { target: ElemRef::Wall(1), by: Vec2::new(dx, 0.0) }).unwrap();
    }
    let r = &h.roads[0];
    let ends = |t: &crate::terrain::Terrain| (t.height(r.a), t.height(r.b));
    assert_eq!(ends(&h.terrain), ends(&before), "les bouts de la route glissent");
    let under =
        |t: &crate::terrain::Terrain| (0..=20).map(|i| t.height(r.a.lerp(r.b, i as f32 / 20.0))).collect::<Vec<_>>();
    assert_eq!(under(&h.terrain), under(&before));
}

/// Une pièce loin de tout, dont on fait aller et venir un mur : chaque pas relance `regrade`.
fn wiggle(h: &mut House, wall: usize, times: usize) {
    for k in 0..times {
        let dx = if k % 2 == 0 { 0.5 } else { -0.5 };
        h.apply(Command::Move { target: ElemRef::Wall(wall), by: Vec2::new(dx, 0.0) }).unwrap();
    }
}

#[test]
fn road_ending_in_a_pool_keeps_the_pool_floor() {
    let mut h = hilly();
    let p = Pool { min: Vec2::new(30.0, -40.0), max: Vec2::new(34.0, -37.0), depth: 1.5, color: WHITE_TEX, texture: 0 };
    h.apply(Command::AddPool(p.clone())).unwrap();
    let r = Road { a: Vec2::new(10.3, -38.6), b: Vec2::new(32.2, -38.4), width: 4.0, color: WHITE_TEX, texture: 0 };
    h.apply(Command::AddRoad(r.clone())).unwrap();
    h.apply(serde_json::from_str(r#"{"op":"room","min":[-60,40],"max":[-54,46],"level":0}"#).unwrap()).unwrap();
    let centre = (p.min + p.max) / 2.0;
    let before = (h.terrain.height(centre), h.terrain.height(r.a), h.terrain.height(r.b));
    wiggle(&mut h, 1, 40);
    let after = (h.terrain.height(centre), h.terrain.height(r.a), h.terrain.height(r.b));
    assert_eq!(after, before, "fond de piscine et bouts de route");
}

#[test]
fn driveway_down_to_a_house_leaves_the_house_floor_flat() {
    let mut h = hilly();
    let (lo, hi) = (Vec2::new(-40.0, 17.0), Vec2::new(-34.0, 23.0));
    h.apply(Command::AddSlab(Slab { min: lo, max: hi, level: 0, color: WHITE_TEX, texture: 0 })).unwrap();
    // l'allée part du relief et s'arrête au bord nord de la maison
    let r = Road { a: Vec2::new(-37.0, -5.0), b: Vec2::new(-37.0, 17.0), width: 4.0, color: WHITE_TEX, texture: 0 };
    assert!(h.terrain.height(r.a).abs() > 1.0, "l'allée doit être en pente");
    h.apply(Command::AddRoad(r)).unwrap();
    // et une route en pente qui traverse la maison de part en part
    let through =
        Road { a: Vec2::new(-60.0, 21.0), b: Vec2::new(-14.0, 19.0), width: 4.0, color: WHITE_TEX, texture: 0 };
    assert!((h.terrain.height(through.a) - h.terrain.height(through.b)).abs() > 1.0, "en pente");
    h.apply(Command::AddRoad(through)).unwrap();
    let floor: Vec<f32> = (lo.y as usize..=hi.y as usize)
        .flat_map(|z| (lo.x as i32..=hi.x as i32).map(move |x| Vec2::new(x as f32, z as f32)))
        .map(|p| h.terrain.height(p))
        .collect();
    assert!(floor.iter().all(|&y| y == 0.0), "sol sous la maison : {floor:?}");
}

/// Monde piège : relief haut, maison, piscines, routes hors grille, en biais, croisées, en
/// chaîne, en T, qui finissent dans une piscine ou au bord de la maison, et un tronçon court.
fn tangled() -> House {
    let mut h = House::default();
    h.apply(Command::GenerateTerrain { seed: 11, amp: 40.0, scale: 30.0, mountains: true }).unwrap();
    let road = |a: [f32; 2], b: [f32; 2], width: f32| {
        Command::AddRoad(Road { a: Vec2::from(a), b: Vec2::from(b), width, color: WHITE_TEX, texture: 0 })
    };
    let pool = |min: [f32; 2], max: [f32; 2]| {
        Command::AddPool(Pool { min: Vec2::from(min), max: Vec2::from(max), depth: 1.5, color: WHITE_TEX, texture: 0 })
    };
    for c in [
        serde_json::from_str(r#"{"op":"room","min":[-6,-5],"max":[6,5],"level":0}"#).unwrap(),
        pool([20.0, 20.0], [26.0, 24.0]),
        pool([-30.0, 25.0], [-26.0, 28.0]),
        road([-50.3, -30.7], [40.6, 35.2], 5.0),
        road([-45.1, 30.4], [35.8, -40.3], 4.0),
        road([0.4, -40.6], [0.0, -5.0], 3.0),
        road([0.4, -40.6], [-20.3, -55.1], 3.0),
        road([-20.3, -55.1], [-41.7, -52.2], 3.0),
        road([10.2, 22.1], [23.4, 22.3], 4.0),
        road([-15.6, 10.9], [-28.2, 26.6], 2.5),
        road([30.3, -10.4], [31.6, -10.9], 2.0),
        road([-38.8, -20.2], [-38.8, 5.4], 6.0),
        road([-38.8, -7.4], [-25.0, -7.4], 3.0),
    ] {
        h.apply(c).unwrap();
    }
    h
}

/// Générateur pseudo-aléatoire minimal (xorshift) : la même graine redonne la même suite.
struct Dice(u64);

impl Dice {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn offset(&mut self) -> Vec2 {
        let mut f = || (self.below(81) as f32 - 40.0) / 8.0;
        Vec2::new(f(), f())
    }
}

/// Un point au hasard du terrain, hors grille.
fn spot(d: &mut Dice) -> Vec2 {
    Vec2::new(d.below(141) as f32 - 70.0, d.below(141) as f32 - 70.0) + d.offset() / 10.0
}

fn move_by(d: &mut Dice, target: ElemRef) -> Command {
    Command::Move { target, by: d.offset() }
}

fn stay(_: &mut Dice, target: ElemRef) -> Command {
    Command::Move { target, by: Vec2::ZERO }
}

fn copy_by(d: &mut Dice, target: ElemRef) -> Command {
    Command::Duplicate { target, by: d.offset() * 4.0 }
}

fn retexture(d: &mut Dice, target: ElemRef) -> Command {
    Command::Set { target, fields: fields(serde_json::json!({"texture": d.below(5)})) }
}

fn relevel(d: &mut Dice, target: ElemRef) -> Command {
    Command::Set { target, fields: fields(serde_json::json!({"level": d.below(2), "width": 1 + d.below(6)})) }
}

fn remove(_: &mut Dice, target: ElemRef) -> Command {
    Command::Remove { target }
}

/// Route nouvelle, courte une fois sur trois.
fn new_road(d: &mut Dice, _: ElemRef) -> Command {
    let a = spot(d);
    let b = a + d.offset() * [0.5, 6.0, 6.0][d.below(3)];
    Command::AddRoad(Road { a, b, width: 0.5 + d.below(8) as f32, color: WHITE_TEX, texture: 0 })
}

fn new_pool(d: &mut Dice, _: ElemRef) -> Command {
    let min = spot(d).round();
    let max = min + Vec2::new(1.0 + d.below(6) as f32, 1.0 + d.below(5) as f32);
    Command::AddPool(Pool { min, max, depth: 1.5, color: WHITE_TEX, texture: 0 })
}

fn new_relief(d: &mut Dice, _: ElemRef) -> Command {
    Command::GenerateTerrain { seed: d.below(99) as u32, amp: 60.0, scale: 30.0, mountains: d.below(2) == 0 }
}

fn flatten(_: &mut Dice, _: ElemRef) -> Command {
    Command::FlattenTerrain
}

/// Une commande au hasard sur un mur, un sol, une piscine ou une route existants, ou un ajout.
/// Une table plutôt qu'un `match` : la fonction reste sans branche (score CRAP du code de test).
fn random_command(h: &House, d: &mut Dice) -> Command {
    let lists = [h.walls.len(), h.slabs.len(), h.pools.len(), h.roads.len()];
    let kind = d.below(4);
    let i = d.below(lists[kind].max(1));
    let target = [ElemRef::Wall(i), ElemRef::Slab(i), ElemRef::Pool(i), ElemRef::Road(i)][kind];
    let makers: [fn(&mut Dice, ElemRef) -> Command; 12] =
        [move_by, move_by, move_by, stay, copy_by, retexture, relevel, remove, new_road, new_pool, new_relief, flatten];
    makers[d.below(makers.len())](d, target)
}

/// Un regrade de plus ne doit rien changer ; renvoie l'écart le plus grand sinon.
fn regrade_moves(h: &mut House) -> f32 {
    let settled = h.terrain.clone();
    h.regrade();
    h.terrain.h.iter().zip(&settled.h).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max)
}

#[test]
fn random_edits_never_drift_nor_diverge() {
    for seed in [0x9E37_79B9_7F4A_7C15, 0xD1B5_4A32_D192_ED03, 0x2545_F491_4F6C_DD1D] {
        let mut h = tangled();
        assert_eq!(regrade_moves(&mut h), 0.0, "monde de départ");
        let mut d = Dice(seed);
        let mut applied = 0;
        for step in 0..600 {
            let cmd = random_command(&h, &mut d);
            // une commande refusée (hors du terrain, cible absente) laisse le monde intact
            applied += usize::from(h.apply(cmd.clone()).is_ok());
            if step % 7 == 0 {
                let c = Vec2::new(d.below(121) as f32 - 60.0, d.below(121) as f32 - 60.0);
                h.sculpt(crate::terrain::Brush::Raise, c, 6.0, 3.0, 0.0, 0);
            }
            // borne : relief généré (au plus 2 × 60 m) plus ce qu'une route peut prolonger
            let worst = h.terrain.h.iter().fold(0.0_f32, |m, y| m.max(y.abs()));
            assert!(worst.is_finite() && worst <= 200.0, "graine {seed:x} étape {step} ({cmd:?}) : altitude {worst} m");
            assert_eq!(regrade_moves(&mut h), 0.0, "graine {seed:x} étape {step} ({cmd:?}) : un regrade de plus bouge");
        }
        assert!(applied > 250, "trop de commandes refusées : {applied}");
    }
}

#[test]
fn chained_t_junctions_settle_in_one_regrade() {
    // chaque route finit sur la suivante : sans réécrire les bouts, chacune attendrait la suivante
    let mut h = House::default();
    for (a, b) in [
        ((-50.3, -50.2), (-30.4, -40.0)),
        ((-40.1, -40.0), (-20.0, -40.0)),
        ((-20.0, -52.3), (-20.0, -20.0)),
        ((-35.0, -20.0), (-5.0, -20.0)),
    ] {
        let r = Road { a: Vec2::new(a.0, a.1), b: Vec2::new(b.0, b.1), width: 4.0, color: WHITE_TEX, texture: 0 };
        h.apply(Command::AddRoad(r)).unwrap();
    }
    h.apply(Command::GenerateTerrain { seed: 5, amp: 40.0, scale: 25.0, mountains: true }).unwrap();
    assert_eq!(regrade_moves(&mut h), 0.0);
}
