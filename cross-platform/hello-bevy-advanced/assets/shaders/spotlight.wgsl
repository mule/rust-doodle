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
    let frag_pos = in.position.xy;

    // Convert world-space mouse to screen-space (Y is flipped)
    let screen_center = uniforms.viewport_size / 2.0;
    let mouse_screen = vec2<f32>(
        screen_center.x + uniforms.mouse_pos.x,
        screen_center.y - uniforms.mouse_pos.y,
    );

    let dist = distance(frag_pos, mouse_screen);
    let falloff = 1.0 - smoothstep(0.0, uniforms.radius, dist);
    let brightness = falloff * uniforms.intensity;

    return vec4<f32>(brightness, brightness, brightness, 1.0);
}
