use bevy::prelude::*;

mod config;

fn main() {
    let config = config::load_config();
    let [r, g, b, a] = config.background_color;

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: config::ASSETS_DIR.to_string(),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgba(r, g, b, a)))
        .insert_resource(config)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
