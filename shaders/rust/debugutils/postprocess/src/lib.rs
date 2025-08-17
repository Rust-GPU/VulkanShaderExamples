#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec2, vec4, Vec2, Vec4},
    image::SampledImage,
    spirv, Image,
};

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_idx: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    *out_position = vec4(out_uv.x * 2.0 - 1.0, out_uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    // Single pass gauss blur
    let tex_offset = vec2(0.01, 0.01);

    let tc0 = in_uv + vec2(-tex_offset.x, -tex_offset.y);
    let tc1 = in_uv + vec2(0.0, -tex_offset.y);
    let tc2 = in_uv + vec2(tex_offset.x, -tex_offset.y);
    let tc3 = in_uv + vec2(-tex_offset.x, 0.0);
    let tc4 = in_uv + vec2(0.0, 0.0);
    let tc5 = in_uv + vec2(tex_offset.x, 0.0);
    let tc6 = in_uv + vec2(-tex_offset.x, tex_offset.y);
    let tc7 = in_uv + vec2(0.0, tex_offset.y);
    let tc8 = in_uv + vec2(tex_offset.x, tex_offset.y);

    let col0: Vec4 = sampler_color.sample(tc0);
    let col1: Vec4 = sampler_color.sample(tc1);
    let col2: Vec4 = sampler_color.sample(tc2);
    let col3: Vec4 = sampler_color.sample(tc3);
    let col4: Vec4 = sampler_color.sample(tc4);
    let col5: Vec4 = sampler_color.sample(tc5);
    let col6: Vec4 = sampler_color.sample(tc6);
    let col7: Vec4 = sampler_color.sample(tc7);
    let col8: Vec4 = sampler_color.sample(tc8);

    let sum = (1.0 * col0
        + 2.0 * col1
        + 1.0 * col2
        + 2.0 * col3
        + 4.0 * col4
        + 2.0 * col5
        + 1.0 * col6
        + 2.0 * col7
        + 1.0 * col8)
        / 16.0;

    *out_frag_color = vec4(sum.x, sum.y, sum.z, 1.0);
}
