use bevy::prelude::*;
use serde::Deserialize;

const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

#[derive(Resource, Deserialize)]
struct AppConfig {
    text: String,
    font_size: f32,
    color: [f32; 4],
}

fn main() {
    let config_path = format!("{}/config.ron", ASSETS_DIR);
    let config_str = std::fs::read_to_string(&config_path)
        .expect("Failed to read assets/config.ron");
    let config: AppConfig =
        ron::from_str(&config_str).expect("Failed to parse assets/config.ron");

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: ASSETS_DIR.to_string(),
            ..default()
        }))
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, hello_world)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<AppConfig>) {
    let [r, g, b, a] = config.color;
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child((
            Text::new(config.text.clone()),
            TextFont {
                font: asset_server.load("fonts/HackNerdFont-Regular.ttf"),
                font_size: config.font_size,
                ..default()
            },
            TextColor(Color::srgba(r, g, b, a)),
        ));
}

fn hello_world() {
    // Runs every frame — uncomment to see it spam:
    //info!("frame");
}