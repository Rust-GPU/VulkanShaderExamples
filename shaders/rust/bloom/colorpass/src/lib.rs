#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    spirv,
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
    in_pos: Vec4,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_uv = in_uv;
    *out_color = in_color;
    *out_position = ubo.projection * ubo.view * ubo.model * in_pos;
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, _in_uv: Vec2, out_frag_color: &mut Vec4) {
    *out_frag_color = Vec4::new(in_color.x, in_color.y, in_color.z, 1.0);
}
