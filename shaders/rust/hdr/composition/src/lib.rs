#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Vec2, Vec4},
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
    #[spirv(descriptor_set = 0, binding = 0)] sampler_color0: &Sampler,
    #[spirv(descriptor_set = 0, binding = 0)] image_color0: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color1: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image_color1: &Image!(2D, type=f32, sampled),
    out_color: &mut Vec4,
) {
    *out_color = image_color0.sample(*sampler_color0, in_uv);
}
