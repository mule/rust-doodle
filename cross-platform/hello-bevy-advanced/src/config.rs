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

#[cfg(not(target_os = "android"))]
pub const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

#[cfg(not(target_os = "android"))]
pub fn load_config() -> AppConfig {
    let config_path = format!("{}/config.ron", ASSETS_DIR);
    let config_str = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|err| panic!("Failed to read '{}': {}", config_path, err));
    ron::from_str(&config_str)
        .unwrap_or_else(|err| panic!("Failed to parse '{}': {}", config_path, err))
}

#[cfg(target_os = "android")]
pub fn load_config() -> AppConfig {
    const CONFIG_RON: &str = include_str!("../assets/config.ron");
    ron::from_str(CONFIG_RON)
        .unwrap_or_else(|err| panic!("Failed to parse embedded config.ron: {}", err))
}
