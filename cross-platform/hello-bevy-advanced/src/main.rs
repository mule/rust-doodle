use bevy::prelude::*;

mod config;
mod particles;
mod text_wave;

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
        .add_systems(Startup, text_wave::spawn_text)
        .add_systems(Startup, particles::spawn_particles)
        .add_systems(Update, text_wave::animate_color_wave)
        .add_systems(Update, particles::animate_particles)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
