#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec4, Mat3, Mat4, Vec2, Vec3, Vec4};
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
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = in_uv;
    // Skysphere always at center, only use rotation part of modelview matrix
    let rotation_only = Mat4::from_mat3(Mat3::from_mat4(ubo.modelview));
    *out_position = ubo.projection * rotation_only * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 2)] _sampler_color: &spirv_std::image::SampledImage<
        spirv_std::image::Image2d,
    >,
    out_color: &mut Vec4,
) {
    const GRADIENT_START: Vec4 = vec4(0.93, 0.9, 0.81, 1.0);
    const GRADIENT_END: Vec4 = vec4(0.35, 0.5, 1.0, 1.0);

    let mix_factor = ((0.5 - (in_uv.y + 0.05)).min(0.5) / 0.15) + 0.5;
    *out_color = GRADIENT_START.lerp(GRADIENT_END, mix_factor);
}
