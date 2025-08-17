#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec4, Mat4, Vec3, Vec4};
use spirv_std::spirv;

// Push constants
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PushConsts {
    mvp: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    *out_pos = push_consts.mvp * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}
