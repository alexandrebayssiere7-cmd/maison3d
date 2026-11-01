mod camera;
mod editor;
mod model;
mod render;
mod textures;
mod visit;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

fn main() {
    let mut ed = editor::Editor::default();
    // reprend le dernier projet enregistré, sinon montre la maison d'exemple
    match std::fs::read_to_string(&ed.file).ok().and_then(|s| serde_json::from_str(&s).ok()) {
        Some(h) => {
            ed.house = h;
            ed.message = format!("Projet {} rechargé", ed.file);
        }
        None => {
            ed.house = model::demo_house();
            ed.message = "Maison d'exemple — « Nouveau » pour partir de zéro".into();
        }
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Maison 3D".into(), resolution: (1600.0_f32, 950.0_f32).into(), ..default() }),
            ..default()
        }))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .insert_resource(ed)
        .init_resource::<camera::CamState>()
        .init_resource::<visit::Visitor>()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.75, 0.95)))
        .add_systems(Startup, (render::setup_scene, visit::spawn_avatar))
        .add_systems(
            Update,
            (
                ui::ui_system,
                camera::camera_control,
                visit::visit_update,
                visit::animate_avatar,
                editor::editor_input,
                render::rebuild,
                render::update_sun,
            )
                .chain(),
        )
        .run();
}
