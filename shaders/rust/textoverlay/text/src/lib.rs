#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec4, Vec2, Vec4},
    spirv, Image,
};

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec2,
    in_uv: Vec2,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_position = vec4(in_pos.x, in_pos.y, 0.0, 1.0);
    *out_uv = in_uv;
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_font: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let color = sampler_font.sample(in_uv).x;
    *out_frag_color = vec4(color, color, color, color);
}
