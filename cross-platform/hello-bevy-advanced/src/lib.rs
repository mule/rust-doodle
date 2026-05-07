use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;

mod config;
mod particles;
mod spotlight;
mod text_wave;

#[bevy_main]
pub fn main() {
    let config = config::load_config();

    let default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Hello Bevy Advanced".to_string(),
            ..default()
        }),
        ..default()
    });

    // Desktop reads assets from a filesystem path baked in at build time, so a
    // binary moved away from CARGO_MANIFEST_DIR fails fast here rather than
    // booting to a black screen via Bevy's per-asset error logs. Android pulls
    // assets from the APK via AAssetManager, which is Bevy's default.
    #[cfg(not(target_os = "android"))]
    let default_plugins = {
        assert!(
            std::path::Path::new(config::ASSETS_DIR).is_dir(),
            "Assets dir not found: '{}'. Was this binary moved away from CARGO_MANIFEST_DIR?",
            config::ASSETS_DIR,
        );
        default_plugins.set(AssetPlugin {
            file_path: config::ASSETS_DIR.to_string(),
            ..default()
        })
    };

    App::new()
        .add_plugins(default_plugins)
        .insert_resource(ClearColor(config.background_color.into()))
        .insert_resource(config)
        .add_plugins(Material2dPlugin::<spotlight::SpotlightMaterial>::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, text_wave::spawn_text)
        .add_systems(Startup, particles::spawn_particles)
        .add_systems(Startup, spotlight::spawn_spotlight)
        .add_systems(Update, text_wave::animate_color_wave)
        .add_systems(Update, particles::animate_particles)
        .add_systems(Update, spotlight::track_pointer)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
