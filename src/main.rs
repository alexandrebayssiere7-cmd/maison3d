mod agent;
mod camera;
mod command;
mod editor;
mod menu;
mod model;
mod render;
mod terrain;
mod textures;
mod ui;
mod visit;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

fn main() {
    if let Some(code) = agent::cli() {
        std::process::exit(code);
    }
    // la maison d'exemple sert de décor derrière les menus
    let mut ed = editor::Editor::default();
    ed.house = model::demo_house();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maison 3D".into(),
                resolution: (1600.0_f32, 950.0_f32).into(),
                ..default()
            }),
            // la fermeture passe par menu::save_on_close
            close_when_requested: false,
            ..default()
        }))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .init_state::<menu::Screen>()
        .init_resource::<menu::Settings>()
        .insert_resource(ed)
        .init_resource::<camera::CamState>()
        .init_resource::<visit::Visitor>()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.75, 0.95)))
        .add_systems(Startup, (render::setup_scene, visit::spawn_avatar))
        .add_systems(
            Update,
            (
                (menu::menu_system, menu::menu_camera).run_if(not(in_state(menu::Screen::Game))),
                (
                    ui::ui_system,
                    menu::hud_system,
                    camera::camera_control,
                    visit::visit_update,
                    visit::animate_avatar,
                    visit::sync_car,
                    editor::editor_input,
                )
                    .chain()
                    .run_if(in_state(menu::Screen::Game)),
                render::rebuild,
                render::update_sun,
            )
                .chain(),
        )
        .add_systems(Update, menu::save_on_close)
        .run();
}
