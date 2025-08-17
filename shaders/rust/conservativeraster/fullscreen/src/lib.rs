#![no_std]

use spirv_std::glam::{Vec2, Vec4};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = Vec2::new(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_position = Vec4::new(out_uv.x * 2.0 - 1.0, out_uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_color: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 0)] sampler: &Sampler,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = sampler_color.sample(*sampler, in_uv);
}
