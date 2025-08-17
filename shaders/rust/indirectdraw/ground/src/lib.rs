#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4};
use spirv_std::spirv;

#[repr(C)]
pub struct UBO {
    projection: Mat4,
    modelview: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    _in_normal: Vec3,
    in_uv: Vec2,
    _in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = in_uv * 32.0;
    *out_position = ubo.projection * ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_color: &spirv_std::image::SampledImage<
        spirv_std::image::Image2d,
    >,
    out_color: &mut Vec4,
) {
    *out_color = sampler_color.sample(in_uv);
}
