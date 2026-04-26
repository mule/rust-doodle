use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

use crate::config::AppConfig;

#[derive(ShaderType, Clone)]
pub struct SpotlightUniforms {
    pub mouse_pos: Vec2,
    pub radius: f32,
    pub intensity: f32,
    pub viewport_size: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SpotlightMaterial {
    #[uniform(0)]
    pub uniforms: SpotlightUniforms,
}

impl Material2d for SpotlightMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/spotlight.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Resource)]
pub struct SpotlightHandle(pub Handle<SpotlightMaterial>);

pub fn spawn_spotlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SpotlightMaterial>>,
    config: Res<AppConfig>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else { return };
    let w = window.width();
    let h = window.height();

    let material = materials.add(SpotlightMaterial {
        uniforms: SpotlightUniforms {
            mouse_pos: Vec2::ZERO,
            radius: config.spotlight_radius,
            intensity: config.spotlight_intensity,
            viewport_size: Vec2::new(w, h),
        },
    });

    commands.insert_resource(SpotlightHandle(material.clone()));

    // Fullscreen quad at z=15 (above text at z=10) — acts as a darkness overlay
    // with a spotlight hole cut by the shader
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(w, h))),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 15.0),
    ));
}

pub fn track_mouse(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut materials: ResMut<Assets<SpotlightMaterial>>,
    handle: Option<Res<SpotlightHandle>>,
) {
    let Some(handle) = handle else { return };
    let Some(mat) = materials.get_mut(&handle.0) else { return };
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            mat.uniforms.mouse_pos = world_pos;
        }
    }

    mat.uniforms.viewport_size = Vec2::new(window.width(), window.height());
}
