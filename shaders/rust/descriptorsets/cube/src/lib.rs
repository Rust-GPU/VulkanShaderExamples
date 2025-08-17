#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec4, Mat4, Vec2, Vec3, Vec4},
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UboMatrices {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo_matrices: &UboMatrices,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo_matrices.projection
        * ubo_matrices.view
        * ubo_matrices.model
        * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] color_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let texture_color = color_sampler.sample(in_uv);
    *out_frag_color = texture_color * vec4(in_color.x, in_color.y, in_color.z, 1.0);
}
