#![no_std]

use spirv_std::glam::{Mat4, Vec3, Vec4};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub depth_mvp: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
) {
    *out_position = ubo.depth_mvp * Vec4::from((in_pos, 1.0));
}

#[spirv(fragment)]
pub fn main_fs(out_color: &mut Vec4) {
    // Shadow pass only writes depth, color output is not used
    *out_color = Vec4::new(1.0, 0.0, 0.0, 1.0);
}
