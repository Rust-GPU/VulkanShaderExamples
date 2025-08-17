#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub color: Vec4,
    pub position: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
) {
    *out_color = in_color * push_consts.color.truncate();
    let loc_pos = ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let world_pos = loc_pos + push_consts.position;
    *out_position = ubo.projection * ubo.view * world_pos;
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, out_frag_color: &mut Vec4) {
    *out_frag_color = Vec4::new(in_color.x, in_color.y, in_color.z, 1.0);
}
