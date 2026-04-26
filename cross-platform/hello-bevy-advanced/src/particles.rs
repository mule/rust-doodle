use bevy::prelude::*;
use rand::RngExt;

use crate::config::AppConfig;

#[derive(Component)]
pub struct Particle {
    pub phase: f32,
    pub orbit_radius: f32,
    pub base_x: f32,
    pub base_y: f32,
    pub speed_multiplier: f32,
    pub base_hue: f32,
}

pub fn spawn_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<AppConfig>,
) {
    let mut rng = rand::rng();
    let circle = meshes.add(Circle::new(3.0));

    for _ in 0..config.particle_count {
        let base_x = rng.random_range(-400.0..400.0);
        let base_y = rng.random_range(-250.0..250.0);
        let phase = rng.random_range(0.0..std::f32::consts::TAU);
        let orbit_radius = rng.random_range(20.0..80.0);
        let speed_multiplier = rng.random_range(0.5..1.5);
        let base_hue = rng.random_range(0.0..360.0);

        let material = materials.add(ColorMaterial {
            color: Color::hsla(base_hue, 1.0, 0.7, 0.8),
            ..default()
        });

        commands.spawn((
            Mesh2d(circle.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(base_x, base_y, 5.0),
            Particle {
                phase,
                orbit_radius,
                base_x,
                base_y,
                speed_multiplier,
                base_hue,
            },
        ));
    }
}

pub fn animate_particles(
    time: Res<Time>,
    config: Res<AppConfig>,
    mut query: Query<(&Particle, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let t = time.elapsed_secs();
    for (particle, mut transform, material_handle) in &mut query {
        let speed = config.particle_speed * particle.speed_multiplier * 0.02;
        let angle = t * speed + particle.phase;
        transform.translation.x = particle.base_x + angle.cos() * particle.orbit_radius;
        transform.translation.y = particle.base_y + angle.sin() * particle.orbit_radius;

        // Slowly shift hue over time
        let hue = (particle.base_hue + t * 20.0) % 360.0;
        if let Some(mat) = materials.get_mut(&material_handle.0) {
            mat.color = Color::hsla(hue, 1.0, 0.7, 0.8);
        }
    }
}
