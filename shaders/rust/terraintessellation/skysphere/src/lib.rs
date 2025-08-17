#![no_std]

use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub mvp: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    _in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_position = ubo.mvp * Vec4::from((in_pos, 1.0));
    *out_uv = in_uv;
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    out_frag_color: &mut Vec4,
) {
    let color = sampler_color_map.sample(*sampler, in_uv);
    *out_frag_color = Vec4::from((color.xyz(), 1.0));
}
