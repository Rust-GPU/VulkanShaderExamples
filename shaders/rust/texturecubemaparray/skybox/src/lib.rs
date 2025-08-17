#![no_std]

use spirv_std::spirv;
use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec3, Vec4},
    Image, Sampler,
};

#[repr(C)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub inv_model: Mat4,
    pub lod_bias: f32,
    pub cube_map_index: i32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    out_uvw.x *= -1.0;
    out_uvw.y *= -1.0;

    // Remove translation from view matrix
    let view_mat = Mat3::from_mat4(ubo.model);
    *out_pos = ubo.projection * Mat4::from_mat3(view_mat) * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uvw: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image: &Image!(cube, type=f32, sampled, arrayed),
    out_frag_color: &mut Vec4,
) {
    let coord = vec4(in_uvw.x, in_uvw.y, in_uvw.z, ubo.cube_map_index as f32);
    *out_frag_color = image.sample_by_lod(*sampler, coord, ubo.lod_bias);
}
