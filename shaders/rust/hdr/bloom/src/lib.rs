#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{UVec2, Vec2, Vec4},
    spirv, Image, Sampler,
};

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = Vec2::new(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_uv = uv;
    *out_position = Vec4::new(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] _sampler_color0: &Sampler,
    #[spirv(descriptor_set = 0, binding = 0)] _image_color0: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color1: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image_color1: &Image!(2D, type=f32, sampled),
    #[spirv(spec_constant(id = 0, default = 0))] dir: u32,
    out_color: &mut Vec4,
) {
    // From the OpenGL Super bible
    const WEIGHTS: [f32; 25] = [
        0.0024499299678342,
        0.0043538453346397,
        0.0073599963704157,
        0.0118349786570722,
        0.0181026699707781,
        0.0263392293891488,
        0.0364543006660986,
        0.0479932050577658,
        0.0601029809166942,
        0.0715974486241365,
        0.0811305381519717,
        0.0874493212267511,
        0.0896631113333857,
        0.0874493212267511,
        0.0811305381519717,
        0.0715974486241365,
        0.0601029809166942,
        0.0479932050577658,
        0.0364543006660986,
        0.0263392293891488,
        0.0181026699707781,
        0.0118349786570722,
        0.0073599963704157,
        0.0043538453346397,
        0.0024499299678342,
    ];

    const BLUR_SCALE: f32 = 0.003;
    const BLUR_STRENGTH: f32 = 1.0;

    let mut ar = 1.0;
    // Aspect ratio for vertical blur pass
    if dir == 1 {
        let ts: UVec2 = image_color1.query_size_lod(0);
        ar = ts.y as f32 / ts.x as f32;
    }

    let p = Vec2::new(in_uv.y, in_uv.x)
        - Vec2::new(0.0, (WEIGHTS.len() as f32 / 2.0) * ar * BLUR_SCALE);

    let mut color = Vec4::ZERO;
    for i in 0..WEIGHTS.len() {
        let dv = Vec2::new(0.0, i as f32 * BLUR_SCALE) * ar;
        let sample = image_color1.sample(*sampler_color1, p + dv);
        color += sample * WEIGHTS[i] * BLUR_STRENGTH;
    }

    *out_color = color;
}
