#![no_std]

use spirv_std::glam::{Vec2, Vec3, Vec4};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub position: Vec4,
    pub cascade_index: u32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    #[spirv(flat)] out_cascade_index: &mut u32,
) {
    let uv = Vec2::new(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_uv = uv;
    *out_cascade_index = push_consts.cascade_index;
    *out_position = Vec4::new(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(flat)] in_cascade_index: u32,
    #[spirv(descriptor_set = 0, binding = 1)] shadow_map: &Image!(2D, type=f32, sampled, arrayed),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    out_frag_color: &mut Vec4,
) {
    let depth = shadow_map
        .sample(
            *sampler,
            Vec3::new(in_uv.x, in_uv.y, in_cascade_index as f32),
        )
        .x;
    *out_frag_color = Vec4::new(depth, depth, depth, 1.0);
}
