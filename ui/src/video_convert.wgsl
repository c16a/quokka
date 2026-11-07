// Converts one decoded video frame from Y'CbCr to RGB, one pixel per texel of the output. The
// arithmetic matches `quokka_playback::decode::frames::convert`, so the two give the same pixels.

struct Params {
    width: u32,
    height: u32,
    shift_x: u32,
    shift_y: u32,
    // Bits to drop from each sample to get its 8-bit value.
    sample_shift: u32,
    // 1 when Cb and Cr share one plane, with Cb first.
    interleaved: u32,
    luma_offset: f32,
    luma_scale: f32,
    chroma_scale: f32,
    r_from_cr: f32,
    g_from_cb: f32,
    g_from_cr: f32,
    b_from_cb: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var luma: texture_2d<u32>;
@group(0) @binding(2) var chroma_a: texture_2d<u32>;
@group(0) @binding(3) var chroma_b: texture_2d<u32>;

// A triangle that covers the whole target.
@vertex
fn vs_full(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(corners[index], 0.0, 1.0);
}

@fragment
fn fs_convert(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let x = u32(position.x);
    let y = u32(position.y);
    let luma_raw = textureLoad(luma, vec2<i32>(i32(x), i32(y)), 0).r;
    let luma_value = f32(luma_raw >> params.sample_shift);

    // Chroma is shared by a block of pixels, so each pixel reads the chroma sample for its block.
    let chroma_pos = vec2<i32>(i32(x >> params.shift_x), i32(y >> params.shift_y));
    let cb_texel = textureLoad(chroma_a, chroma_pos, 0);
    let cr_texel = textureLoad(chroma_b, chroma_pos, 0);
    let cb_raw = cb_texel.r;
    let cr_raw = select(cr_texel.r, cb_texel.g, params.interleaved == 1u);
    let cb = (f32(cb_raw >> params.sample_shift) - 128.0) * params.chroma_scale;
    let cr = (f32(cr_raw >> params.sample_shift) - 128.0) * params.chroma_scale;

    let y_part = (luma_value - params.luma_offset) * params.luma_scale;
    let r = clamp(255.0 * (y_part + params.r_from_cr * cr), 0.0, 255.0);
    let g = clamp(255.0 * (y_part - params.g_from_cb * cb - params.g_from_cr * cr), 0.0, 255.0);
    let b = clamp(255.0 * (y_part + params.b_from_cb * cb), 0.0, 255.0);
    return vec4<f32>(r, g, b, 255.0) / 255.0;
}
