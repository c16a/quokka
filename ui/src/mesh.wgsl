// Mesh gradient background: a 3x3 grid of control points that drift in small loops. Each point takes
// the cover's dominant colour, or a lighter or deeper shade of it, and every pixel is a Gaussian-weighted
// blend of the points near it. The colours therefore flow into one another.

struct Params {
    base: vec4<f32>,
    light: vec4<f32>,
    deep: vec4<f32>,
    // x: time in seconds, y: per-album seed, z: width over height, w: 1 when the target is sRGB.
    settings: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_mesh(@builtin(vertex_index) index: u32) -> VertexOut {
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

fn hash1(n: f32) -> f32 {
    return fract(sin(n * 127.1) * 43758.5453);
}

fn linear_from_gamma(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c < vec3<f32>(0.04045));
}

@fragment
fn fs_mesh(in: VertexOut) -> @location(0) vec4<f32> {
    // Each point completes a drift in roughly 9 to 17 seconds, so the motion is visible.
    let time = params.settings.x * 0.6;
    let seed = params.settings.y;
    let aspect = params.settings.z;
    let p = vec2<f32>(in.uv.x * aspect, in.uv.y);
    var sum = vec3<f32>(0.0);
    var weight_sum = 0.0;
    for (var j = 0; j < 3; j++) {
        for (var i = 0; i < 3; i++) {
            let k = f32(j * 3 + i);
            let phase = hash1(k + seed * 13.0) * 6.2831853;
            let speed = 0.6 + 0.6 * hash1(k * 3.7 + seed);
            // The grid spans the area, and each point drifts in its own small loop.
            let home = vec2<f32>(f32(i) * 0.5 * aspect, f32(j) * 0.5);
            let drift = 0.18 * vec2<f32>(
                sin(time * speed + phase),
                cos(time * speed * 0.8 + phase * 1.7),
            );
            let d = p - (home + drift);
            let w = exp(-dot(d, d) / 0.05);
            // The point's colour moves between the dominant colour's lighter and deeper shades.
            let shade = 0.5 + 0.5 * sin(time * speed * 0.9 + phase);
            let tint = mix(params.light.rgb, params.deep.rgb, shade);
            let colour = mix(params.base.rgb, tint, 0.5);
            sum += w * colour;
            weight_sum += w;
        }
    }
    var out_colour = sum / max(weight_sum, 0.0001);
    if params.settings.w > 0.5 {
        out_colour = linear_from_gamma(out_colour);
    }
    return vec4<f32>(out_colour, 1.0);
}
