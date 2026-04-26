use bevy::prelude::*;

use crate::config::AppConfig;

#[derive(Component)]
pub struct WaveIndex(pub usize);

pub fn spawn_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<AppConfig>,
) {
    let font = asset_server.load("fonts/HackNerdFont-Regular.ttf");
    let char_count = config.text.chars().count();
    // Approximate character width — Hack is monospace, so font_size * 0.6 is close
    let char_width = config.font_size * 0.6;
    let total_width = char_count as f32 * char_width;
    let start_x = -total_width / 2.0;

    for (i, ch) in config.text.chars().enumerate() {
        commands.spawn((
            Text2d::new(ch.to_string()),
            TextFont {
                font: font.clone(),
                font_size: config.font_size,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(start_x + i as f32 * char_width, 0.0, 10.0),
            WaveIndex(i),
        ));
    }
}

pub fn animate_color_wave(
    time: Res<Time>,
    config: Res<AppConfig>,
    mut query: Query<(&WaveIndex, &mut TextColor)>,
) {
    let t = time.elapsed_secs();
    for (wave_index, mut color) in &mut query {
        let hue = (t * config.wave_speed + wave_index.0 as f32 * config.wave_spread).rem_euclid(1.0);
        *color = TextColor(Color::hsl(hue * 360.0, 1.0, 0.6));
    }
}
