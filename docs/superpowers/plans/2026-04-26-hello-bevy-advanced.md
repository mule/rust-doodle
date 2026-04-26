# hello-bevy-advanced Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a graphically enhanced "Hello, Bevy!" with per-character rainbow wave, floating particles, and a mouse-tracked spotlight — all in 2D.

**Architecture:** Hybrid ECS + shader. Three independent visual systems (text_wave, particles, spotlight) each as their own module, driven by a shared RON config. Only the spotlight uses a custom WGSL shader; everything else is pure Rust ECS.

**Tech Stack:** Bevy 0.18.1, serde + ron 0.12, rand 0.10

---

### Task 1: Project Scaffold + Config

**Files:**
- Create: `cross-platform/hello-bevy-advanced/Cargo.toml`
- Create: `cross-platform/hello-bevy-advanced/src/main.rs`
- Create: `cross-platform/hello-bevy-advanced/src/config.rs`
- Create: `cross-platform/hello-bevy-advanced/assets/config.ron`
- Copy: `cross-platform/hello-bevy/assets/fonts/HackNerdFont-Regular.ttf` → `cross-platform/hello-bevy-advanced/assets/fonts/HackNerdFont-Regular.ttf`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "hello-bevy-advanced"
version = "0.1.0"
edition = "2024"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[dependencies]
bevy = "0.18.1"
serde = { version = "1", features = ["derive"] }
ron = "0.12"
rand = "0.10"
```

- [ ] **Step 2: Create config.rs**

```rust
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
```

- [ ] **Step 3: Create main.rs (minimal — just config + camera + background)**

```rust
use bevy::prelude::*;

mod config;

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
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

- [ ] **Step 4: Create assets/config.ron**

```ron
AppConfig(
    text: "Hello, Bevy!",
    font_size: 60.0,
    wave_speed: 2.0,
    wave_spread: 0.3,
    particle_count: 40,
    particle_speed: 50.0,
    spotlight_radius: 200.0,
    spotlight_intensity: 0.6,
    background_color: (0.04, 0.04, 0.1, 1.0),
)
```

- [ ] **Step 5: Copy font file**

```bash
mkdir -p cross-platform/hello-bevy-advanced/assets/fonts
cp cross-platform/hello-bevy/assets/fonts/HackNerdFont-Regular.ttf cross-platform/hello-bevy-advanced/assets/fonts/
```

- [ ] **Step 6: Build and run to verify dark background + empty window**

```bash
cargo build --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
```

Expected: A window opens with a dark navy background (`#0a0a1a`), nothing else.

- [ ] **Step 7: Commit**

```bash
git add cross-platform/hello-bevy-advanced/
git commit -m "feat: scaffold hello-bevy-advanced with config and dark background"
```

---

### Task 2: Text Wave System

**Files:**
- Create: `cross-platform/hello-bevy-advanced/src/text_wave.rs`
- Modify: `cross-platform/hello-bevy-advanced/src/main.rs`

- [ ] **Step 1: Create text_wave.rs**

```rust
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
        let hue = (t * config.wave_speed + wave_index.0 as f32 * config.wave_spread) % 1.0;
        *color = TextColor(Color::hsl(hue * 360.0, 1.0, 0.6));
    }
}
```

- [ ] **Step 2: Wire text_wave into main.rs**

Add to main.rs:

```rust
mod text_wave;
```

And in the `App::new()` chain, add:

```rust
        .add_systems(Startup, text_wave::spawn_text)
        .add_systems(Update, text_wave::animate_color_wave)
```

- [ ] **Step 3: Build and run**

```bash
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
```

Expected: "Hello, Bevy!" appears centered, each character cycling through rainbow colors as a flowing wave.

- [ ] **Step 4: Commit**

```bash
git add cross-platform/hello-bevy-advanced/src/
git commit -m "feat: add per-character rainbow wave text animation"
```

---

### Task 3: Particle System

**Files:**
- Create: `cross-platform/hello-bevy-advanced/src/particles.rs`
- Modify: `cross-platform/hello-bevy-advanced/src/main.rs`

- [ ] **Step 1: Create particles.rs**

```rust
use bevy::prelude::*;
use rand::Rng;

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
```

- [ ] **Step 2: Wire particles into main.rs**

Add to main.rs:

```rust
mod particles;
```

And in the `App::new()` chain, add:

```rust
        .add_systems(Startup, particles::spawn_particles)
        .add_systems(Update, particles::animate_particles)
```

- [ ] **Step 3: Build and run**

```bash
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
```

Expected: Small colored circles float and orbit around the text area, gently shifting colors.

- [ ] **Step 4: Commit**

```bash
git add cross-platform/hello-bevy-advanced/src/
git commit -m "feat: add floating particle system with color shifting"
```

---

### Task 4: Spotlight Shader

**Files:**
- Create: `cross-platform/hello-bevy-advanced/assets/shaders/spotlight.wgsl`
- Create: `cross-platform/hello-bevy-advanced/src/spotlight.rs`
- Modify: `cross-platform/hello-bevy-advanced/src/main.rs`

- [ ] **Step 1: Create the WGSL shader**

Create `cross-platform/hello-bevy-advanced/assets/shaders/spotlight.wgsl`:

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SpotlightUniforms {
    mouse_pos: vec2<f32>,
    radius: f32,
    intensity: f32,
    viewport_size: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> uniforms: SpotlightUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert fragment position to screen space
    let frag_pos = in.position.xy;

    // Mouse position is in world coordinates, convert to screen center-origin
    let screen_center = uniforms.viewport_size / 2.0;
    let mouse_screen = screen_center + uniforms.mouse_pos;

    let dist = distance(frag_pos, mouse_screen);
    let falloff = 1.0 - smoothstep(0.0, uniforms.radius, dist);
    let brightness = falloff * uniforms.intensity;

    return vec4<f32>(brightness, brightness, brightness, brightness);
}
```

- [ ] **Step 2: Create spotlight.rs**

```rust
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use crate::config::{AppConfig, ASSETS_DIR};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SpotlightMaterial {
    #[uniform(0)]
    pub mouse_pos: Vec2,
    #[uniform(0)]
    pub radius: f32,
    #[uniform(0)]
    pub intensity: f32,
    #[uniform(0)]
    pub viewport_size: Vec2,
}

impl Material2d for SpotlightMaterial {
    fn fragment_shader() -> ShaderRef {
        format!("{}/shaders/spotlight.wgsl", ASSETS_DIR).into()
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
    let window = windows.single();
    let w = window.width();
    let h = window.height();

    let material = materials.add(SpotlightMaterial {
        mouse_pos: Vec2::ZERO,
        radius: config.spotlight_radius,
        intensity: config.spotlight_intensity,
        viewport_size: Vec2::new(w, h),
    });

    commands.insert_resource(SpotlightHandle(material.clone()));

    // Fullscreen quad at z=2 (above background, below particles at z=5 and text at z=10)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(w, h))),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 2.0),
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
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            mat.mouse_pos = world_pos;
        }
    }

    mat.viewport_size = Vec2::new(window.width(), window.height());
}
```

- [ ] **Step 3: Wire spotlight into main.rs**

Add to main.rs:

```rust
mod spotlight;
```

And in the `App::new()` chain, add:

```rust
        .add_plugins(Material2dPlugin::<spotlight::SpotlightMaterial>::default())
        .add_systems(Startup, spotlight::spawn_spotlight)
        .add_systems(Update, spotlight::track_mouse)
```

Also add this import to main.rs:

```rust
use bevy::sprite::Material2dPlugin;
```

- [ ] **Step 4: Build and run**

```bash
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
```

Expected: Moving the mouse creates a soft white radial light that illuminates the area around the cursor. Text and particles near the cursor appear brighter.

- [ ] **Step 5: Commit**

```bash
git add cross-platform/hello-bevy-advanced/
git commit -m "feat: add mouse-tracked spotlight shader"
```

---

### Task 5: Polish + VS Code Launch Config

**Files:**
- Modify: `cross-platform/hello-bevy-advanced/src/main.rs` (window title)
- Modify: `.vscode/launch.json`

- [ ] **Step 1: Set window title**

In main.rs, update `DefaultPlugins` to also set the window title:

```rust
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: config::ASSETS_DIR.to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Hello Bevy Advanced".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
```

- [ ] **Step 2: Add launch config to .vscode/launch.json**

Add a new entry at the top of the `configurations` array (so it becomes the default):

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug hello-bevy-advanced",
    "cargo": {
        "args": ["build", "--manifest-path", "cross-platform/hello-bevy-advanced/Cargo.toml"],
        "filter": {
            "name": "hello-bevy-advanced",
            "kind": "bin"
        }
    },
    "args": [],
    "cwd": "${workspaceFolder}/cross-platform/hello-bevy-advanced"
}
```

- [ ] **Step 3: Build and do a final run**

```bash
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
```

Expected: Window titled "Hello Bevy Advanced" with dark background, rainbow wave text, floating particles, and mouse spotlight all working together.

- [ ] **Step 4: Commit**

```bash
git add cross-platform/hello-bevy-advanced/ .vscode/launch.json
git commit -m "feat: add window title and VS Code launch config for hello-bevy-advanced"
```

---

### Task 6: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Add hello-bevy-advanced to build commands and mention in architecture**

Add the new project's build command alongside the existing ones, and note the shader asset pattern.

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add hello-bevy-advanced to CLAUDE.md"
```
