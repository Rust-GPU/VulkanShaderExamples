#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub color: Vec4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    _out_color: &mut Vec3,
) {
    *out_position = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(_in_color: Vec3, out_frag_color: &mut Vec4) {
    *out_frag_color = vec4(1.0, 1.0, 1.0, 1.0);
}
