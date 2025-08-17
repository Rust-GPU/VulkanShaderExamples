#![no_std]

use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

const SHADOW_MAP_CASCADE_COUNT: usize = 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub position: Vec4,
    pub cascade_index: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub cascade_view_proj_mat: [Mat4; SHADOW_MAP_CASCADE_COUNT],
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(uniform, descriptor_set = 0, binding = 3)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = in_uv;
    let pos = in_pos + push_consts.position.xyz();
    *out_position =
        ubo.cascade_view_proj_mat[push_consts.cascade_index as usize] * Vec4::from((pos, 1.0));
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 1, binding = 0)] color_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 1, binding = 0)] sampler: &Sampler,
) {
    let alpha = color_map.sample(*sampler, in_uv).w;
    if alpha < 0.5 {
        spirv_std::arch::kill();
    }
}
