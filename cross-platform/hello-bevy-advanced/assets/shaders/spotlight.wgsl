#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SpotlightUniforms {
    mouse_pos: vec2<f32>,
    radius: f32,
    intensity: f32,
};

@group(2) @binding(0)
var<uniform> uniforms: SpotlightUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // frag_pos and mouse_pos are both in physical framebuffer pixels.
    let dist = distance(in.position.xy, uniforms.mouse_pos);
    let falloff = 1.0 - smoothstep(0.0, uniforms.radius, dist);
    let brightness = falloff * uniforms.intensity;
    return vec4<f32>(brightness, brightness, brightness, 1.0);
}
