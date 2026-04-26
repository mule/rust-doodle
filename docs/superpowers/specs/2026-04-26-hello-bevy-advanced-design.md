# hello-bevy-advanced Design Spec

A graphically enhanced "Hello, Bevy!" demo featuring per-character color wave animation, floating particles, and a mouse-tracked spotlight — all in 2D with a hybrid ECS + shader approach.

## Rendering Approach

Hybrid: pure Rust ECS for character animation and particles, one custom WGSL shader for the spotlight effect.

## Project Structure

```
cross-platform/hello-bevy-advanced/
├── Cargo.toml
├── assets/
│   ├── config.ron
│   ├── fonts/
│   │   └── HackNerdFont-Regular.ttf
│   └── shaders/
│       └── spotlight.wgsl
└── src/
    ├── main.rs          # App setup, plugin registration
    ├── config.rs        # RON config loading
    ├── text_wave.rs     # Per-character spawning + color wave animation
    ├── particles.rs     # Particle spawning + movement
    └── spotlight.rs     # Custom 2D material + mouse tracking
```

## Dependencies

- `bevy = "0.18.1"` (same version as hello-bevy)
- `serde` + `ron` for config
- `rand` for particle randomization

## Config (assets/config.ron)

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

All visual parameters are tuneable without recompilation.

## Systems

### Text Wave (text_wave.rs)

**Startup:** Split config text into individual characters. Spawn each as a separate `Text2d` entity positioned horizontally. Each gets a `WaveIndex(usize)` component.

**Update (`animate_color_wave`):** Per frame, compute each character's hue as `(time * wave_speed + index * wave_spread) % 1.0`. Convert HSL to sRGB and write to `TextColor`. Creates a smooth rainbow flowing left-to-right.

### Particles (particles.rs)

**Startup:** Spawn `particle_count` small circle sprites at random positions around the text. Each gets a `Particle` component with random phase offset, orbit radius, and base color.

**Update (`animate_particles`):** Each particle floats using `sin`/`cos` with its phase offset + elapsed time. Colors shift slowly over time. Particles stay within a bounding area around the text.

### Spotlight (spotlight.rs)

**Rust side:** A `SpotlightMaterial` struct implementing `Material2d` with uniforms: `mouse_pos`, `radius`, `intensity`. A fullscreen quad entity uses this material, rendered above the background but below text/particles. An update system reads `CursorMoved` events and writes mouse position to the material.

**WGSL shader (assets/shaders/spotlight.wgsl):** Computes distance from fragment to `mouse_pos`. Outputs a soft radial gradient — bright near cursor, transparent at distance. Blends additively to illuminate text and particles beneath.

## Render Layering (back to front)

1. Background (solid dark color via `ClearColor`)
2. Spotlight fullscreen quad (additive blend)
3. Particles (small glowing sprites)
4. Text characters (per-character `Text2d` entities)

## main.rs Responsibilities

- Load RON config before app startup (same pattern as hello-bevy)
- Set `ClearColor` from config
- Configure `AssetPlugin` with `CARGO_MANIFEST_DIR`
- Register `Material2d` plugin for `SpotlightMaterial`
- Add startup and update systems from each module
