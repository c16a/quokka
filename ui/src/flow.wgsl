// Flow background: warped noise, with thin strands where its bands are narrow. The time input moves
// the warp slowly, so the strands drift.

struct Params {
    base: vec4<f32>,
    light: vec4<f32>,
    deep: vec4<f32>,
    // x: time in seconds, y: per-movie seed, z: width over height, w: 1 when the target is sRGB.
    settings: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_flow(@builtin(vertex_index) index: u32) -> VertexOut {
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

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y,
    );
}

fn fbm(p: vec2<f32>) -> f32 {
    var sum = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var octave = 0; octave < 4; octave++) {
        sum += amplitude * noise(q);
        q = q * 2.03 + vec2<f32>(1.7, 9.2);
        amplitude *= 0.5;
    }
    return sum;
}

fn linear_from_gamma(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c < vec3<f32>(0.04045));
}

@fragment
fn fs_flow(in: VertexOut) -> @location(0) vec4<f32> {
    let time = params.settings.x * 0.05;
    let seed = params.settings.y;
    let p = vec2<f32>(in.uv.x * params.settings.z, in.uv.y) * 2.5
        + vec2<f32>(seed, seed * 0.7);
    // Two warps of the same noise, so the field bends and drifts over time.
    let warp = vec2<f32>(
        fbm(p + vec2<f32>(0.0, time)),
        fbm(p + vec2<f32>(5.2, 1.3) - vec2<f32>(time * 0.7, 0.0)),
    );
    let field = fbm(p + 2.0 * warp + vec2<f32>(0.3, 0.9) + vec2<f32>(time * 0.5, 0.0));
    // Thin strands where the field's bands are narrow.
    let strand = smoothstep(0.92, 1.0, abs(sin(field * 34.0)));
    let soft = smoothstep(0.3, 0.7, warp.x);
    var colour = mix(params.base.rgb, params.light.rgb, 0.22 * strand);
    colour = mix(colour, params.deep.rgb, 0.18 * (1.0 - soft));
    if params.settings.w > 0.5 {
        colour = linear_from_gamma(colour);
    }
    return vec4<f32>(colour, 1.0);
}
