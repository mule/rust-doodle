use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Deserialize, Clone)]
pub struct AppConfig {
    pub text: String,
    pub font_size: f32,
    pub wave_speed: f32,
    pub wave_spread: f32,
    pub particle_count: u32,
    pub particle_speed: f32,
    pub spotlight_radius: f32,
    pub spotlight_intensity: f32,
    pub background_color: [f32; 4],
}

pub const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

pub fn load_config() -> AppConfig {
    let config_path = format!("{}/config.ron", ASSETS_DIR);
    let config_str = std::fs::read_to_string(&config_path)
        .expect("Failed to read assets/config.ron");
    ron::from_str(&config_str).expect("Failed to parse assets/config.ron")
}
