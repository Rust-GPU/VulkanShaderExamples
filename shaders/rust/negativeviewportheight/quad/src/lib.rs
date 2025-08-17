#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec4, Vec2, Vec3, Vec4},
    spirv, Image,
};

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = in_uv;
    *out_position = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_color: &mut Vec4,
) {
    *out_color = sampler_color.sample(in_uv);
}
