use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, hello_world)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            Text::new("Hello, Bevy!"),
            TextFont {
                font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn hello_world() {
    // Runs every frame — uncomment to see it spam:
    info!("frame");
}