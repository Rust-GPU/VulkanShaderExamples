#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec2, vec4, Vec2, Vec4},
    spirv, Image,
};

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv_x = ((vertex_index << 1) & 2) as f32;
    let uv_y = (vertex_index & 2) as f32;
    *out_uv = vec2(uv_x, uv_y);
    *out_position = vec4(uv_x * 2.0 - 1.0, uv_y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = sampler_color.sample(in_uv);
}
