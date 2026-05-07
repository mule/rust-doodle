use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::Material2d,
};

use crate::config::AppConfig;

#[derive(ShaderType, Clone)]
pub struct SpotlightUniforms {
    // mouse_pos is in physical framebuffer pixels to match WGSL's frag_pos.
    pub mouse_pos: Vec2,
    pub radius: f32,
    pub intensity: f32,
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
    let window = windows
        .single()
        .expect("primary window required at Startup to size spotlight quad");
    let w = window.width();
    let h = window.height();

    let material = materials.add(SpotlightMaterial {
        uniforms: SpotlightUniforms {
            mouse_pos: Vec2::ZERO,
            radius: config.spotlight_radius,
            intensity: config.spotlight_intensity,
        },
    });

    commands.insert_resource(SpotlightHandle(material.clone()));

    // Fullscreen quad at z=2 (above background, below particles at z=5 and text at z=10)
    // Acts as a glow layer on the dark background
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(w, h))),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));
}

pub fn track_pointer(
    windows: Query<&Window>,
    touches: Res<Touches>,
    config: Res<AppConfig>,
    mut materials: ResMut<Assets<SpotlightMaterial>>,
    handle: Option<Res<SpotlightHandle>>,
) {
    let Some(handle) = handle else { return };
    let mat = materials
        .get_mut(&handle.0)
        .expect("SpotlightMaterial asset missing despite handle resource being present");
    let Ok(window) = windows.single() else { return };

    let scale = window.scale_factor() as f32;
    let physical_size = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );

    // radius is configured in logical pixels; convert to physical to match
    // mouse_pos and in.position.xy units in the shader.
    mat.uniforms.radius = config.spotlight_radius * scale;

    // Prefer the first active touch; fall back to the mouse cursor on desktop.
    // On Android, cursor_position() returns a stale Some(Vec2::ZERO) rather than
    // None, which would lock the spotlight at the top-left — so we skip it there.
    let logical_pos = touches.iter().next().map(|t| t.position());
    #[cfg(not(target_os = "android"))]
    let logical_pos = logical_pos.or_else(|| window.cursor_position());

    // mouse_pos in physical pixels to match WGSL's frag_pos. Idle at center.
    mat.uniforms.mouse_pos = logical_pos
        .map(|p| p * scale)
        .unwrap_or(physical_size * 0.5);
}
