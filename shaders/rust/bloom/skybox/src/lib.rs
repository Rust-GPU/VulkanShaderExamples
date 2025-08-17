#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{Mat4, Vec3, Vec4},
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    *out_position =
        ubo.projection * ubo.view * ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(descriptor_set = 0, binding = 1)] sampler_cube_map: &SampledImage<
        Image!(cube, type=f32, sampled),
    >,
    in_uvw: Vec3,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = sampler_cube_map.sample(in_uvw);
}
