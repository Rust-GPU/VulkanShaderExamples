#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4},
    spirv, Image,
};
// Each Instance needs to be exactly 80 bytes for proper alignment in arrays
// NOTE: rust-gpu has issues with array padding in structs that are themselves in arrays,
// so we use individual padding fields instead of _pad: [f32; 3]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Instance {
    pub model: Mat4,      // 64 bytes
    pub array_index: f32, // 4 bytes
    pub _pad0: f32,       // 4 bytes padding
    pub _pad1: f32,       // 4 bytes padding
    pub _pad2: f32,       // 4 bytes padding
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub instance: [Instance; 8],
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    #[spirv(instance_index)] instance_index: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec3,
) {
    let instance = &ubo.instance[instance_index as usize];
    *out_uv = vec3(in_uv.x, in_uv.y, instance.array_index);
    let model_view = ubo.view * instance.model;
    *out_position = ubo.projection * model_view * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_array: &SampledImage<
        Image!(2D, type=f32, sampled, arrayed),
    >,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = sampler_array.sample(in_uv);
}
