#![no_std]

use spirv_std::spirv;
use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec3, Vec4},
    image::{Cubemap, SampledImage},
};

#[repr(C)]
pub struct SkyboxUBO {
    pub projection: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &SkyboxUBO,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    out_uvw.x *= -1.0;
    out_uvw.y *= -1.0;

    let view_mat = Mat4::from_mat3(Mat3::from_mat4(ubo.model));
    *out_pos = ubo.projection * view_mat * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uvw: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_cube_map: &SampledImage<Cubemap>,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = sampler_cube_map.sample(in_uvw);
}
