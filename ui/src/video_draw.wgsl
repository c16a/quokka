// Draws the converted video into the window, scaled to fit its place. The colour is bilinear
// filtered, as the CPU path was. When the window's target is sRGB, the value is turned back to
// linear light first, as egui does for its own textures.

override srgb_target: bool = false;

@group(0) @binding(0) var video: texture_2d<f32>;
@group(0) @binding(1) var video_sampler: sampler;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// A triangle that covers the viewport, with the video's texture coordinates across it.
@vertex
fn vs_draw(@builtin(vertex_index) index: u32) -> VertexOut {
    let corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let corner = corners[index];
    var out: VertexOut;
    out.position = vec4<f32>(corner, 0.0, 1.0);
    out.uv = vec2<f32>((corner.x + 1.0) * 0.5, (1.0 - corner.y) * 0.5);
    return out;
}

fn linear_from_gamma(srgb: vec3<f32>) -> vec3<f32> {
    let low = srgb / 12.92;
    let high = pow((srgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, srgb < vec3<f32>(0.04045));
}

@fragment
fn fs_draw(in: VertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(video, video_sampler, in.uv).rgb;
    if srgb_target {
        return vec4<f32>(linear_from_gamma(color), 1.0);
    }
    return vec4<f32>(color, 1.0);
}
