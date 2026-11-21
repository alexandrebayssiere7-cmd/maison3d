//! Maison d'exemple : un rez-de-chaussée meublé, un toit, un jardin, une piscine et une route.

use bevy::math::Vec2;

use crate::command::Command;

use super::{
    House, LEVEL_H, Opening, OpeningKind, Pool, Prop, PropKind, Road, Roof, RoofKind, Slab, WHITE_TEX, Wall, room_walls,
};

pub fn demo_house() -> House {
    use crate::textures::find;
    let mut h = House { walls: demo_walls(), props: demo_props(), ..House::default() };
    h.slabs.push(Slab {
        min: Vec2::ZERO,
        max: Vec2::new(10.0, 8.0),
        level: 0,
        color: WHITE_TEX,
        texture: find("Parquet chêne"),
    });
    h.slabs.push(Slab {
        min: Vec2::new(-2.0, -5.0),
        max: Vec2::new(12.0, -0.13),
        level: 0,
        color: WHITE_TEX,
        texture: find("Dalles de jardin"),
    });
    h.roofs.push(Roof {
        min: Vec2::ZERO,
        max: Vec2::new(10.0, 8.0),
        level: 1,
        kind: RoofKind::Gable,
        height: 2.5,
        overhang: 0.5,
        color: WHITE_TEX,
        texture: find("Tuiles terre cuite"),
    });
    h.generate_terrain(7, 6.0, 45.0, false);
    // par `apply`, comme toute modification : piscine et route se posent sur le relief de la même façon
    let pool = Pool {
        min: Vec2::new(-10.0, 1.0),
        max: Vec2::new(-4.0, 5.0),
        depth: 1.5,
        color: WHITE_TEX,
        texture: find("Mosaïque bleue"),
    };
    h.apply(Command::AddPool(pool)).expect("la piscine de la démo est valide");
    let road = Road {
        a: Vec2::new(14.0, -6.0),
        b: Vec2::new(14.0, -60.0),
        width: 4.0,
        color: [0.55, 0.55, 0.57],
        texture: find("Béton brut"),
    };
    h.apply(Command::AddRoad(road)).expect("la route de la démo est valide");
    h
}

fn demo_walls() -> Vec<Wall> {
    use crate::textures::find;
    let mut walls = room_walls(Vec2::ZERO, Vec2::new(10.0, 8.0), 0, LEVEL_H, 0.25, WHITE_TEX);
    for w in &mut walls {
        w.texture = find("Crépi beige");
    }
    walls.push(Wall {
        a: Vec2::new(6.0, 0.0),
        b: Vec2::new(6.0, 8.0),
        level: 0,
        height: LEVEL_H,
        thick: 0.12,
        color: [0.85, 0.9, 0.95],
        openings: vec![],
        texture: find("Plâtre"),
    });
    let op = Opening::new;
    walls[0].openings = vec![op(OpeningKind::Door, 3.0), op(OpeningKind::Window, 1.2), op(OpeningKind::Window, 8.0)];
    walls[1].openings = vec![op(OpeningKind::Window, 4.0)];
    walls[2].openings = vec![op(OpeningKind::Bay, 6.5), op(OpeningKind::Window, 2.0)];
    walls[3].openings = vec![op(OpeningKind::Window, 4.0)];
    walls[4].openings = vec![op(OpeningKind::Door, 5.0)];
    walls
}

fn demo_props() -> Vec<Prop> {
    let mut props = Vec::new();
    let mut prop = |kind: PropKind, x: f32, z: f32, rot: f32| {
        props.push(Prop::new(kind, Vec2::new(x, z), 0, rot));
    };
    prop(PropKind::Sofa, 2.5, 6.8, 180.0);
    prop(PropKind::Table, 3.0, 3.5, 0.0);
    prop(PropKind::Chair, 2.5, 2.8, 0.0);
    prop(PropKind::Chair, 3.5, 2.8, 0.0);
    prop(PropKind::Chair, 2.5, 4.2, 180.0);
    prop(PropKind::Chair, 3.5, 4.2, 180.0);
    prop(PropKind::Kitchen, 1.6, 0.45, 0.0);
    prop(PropKind::Fridge, 5.3, 0.5, 0.0);
    prop(PropKind::Plant, 5.4, 7.4, 0.0);
    prop(PropKind::Bed, 8.0, 6.8, 180.0);
    prop(PropKind::Wardrobe, 9.4, 2.0, 270.0);
    prop(PropKind::Lamp, 0.6, 7.4, 0.0);
    prop(PropKind::Tree, -4.0, -3.0, 0.0);
    prop(PropKind::Pine, 14.0, 10.0, 0.0);
    prop(PropKind::Tree, -5.0, 9.0, 0.0);
    prop(PropKind::Pine, 16.0, -2.0, 0.0);
    prop(PropKind::Car, 8.6, -2.6, 90.0);
    for i in 0..5 {
        prop(PropKind::Hedge, 11.5, 1.0 + 2.0 * i as f32, 90.0);
    }
    prop(PropKind::Bush, 1.0, -1.2, 0.0);
    prop(PropKind::Bush, 5.0, -1.2, 0.0);
    for i in 0..6 {
        prop(PropKind::Fence, -2.0 + 2.0 * i as f32 + 1.0, -4.0, 0.0);
    }
    prop(PropKind::Flowers, 8.5, -1.2, 0.0);
    prop(PropKind::Rosebush, -1.2, -1.2, 0.0);
    prop(PropKind::Birch, -12.0, 4.0, 0.0);
    prop(PropKind::AppleTree, 15.0, 4.0, 0.0);
    prop(PropKind::Palm, -7.0, -3.0, 0.0);
    props
}
