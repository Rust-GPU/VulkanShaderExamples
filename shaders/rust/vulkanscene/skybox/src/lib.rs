#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec3, Vec4},
    image::{Cubemap, SampledImage},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub normal: Mat4,
    pub view: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    // Remove translation from view matrix
    let view_mat = Mat4::from_mat3(Mat3::from_mat4(ubo.view));
    *out_position = ubo.projection * view_mat * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uvw: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_cube_map: &SampledImage<Cubemap>,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = sampler_cube_map.sample(in_uvw);
}
