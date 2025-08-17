#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Vec3, Vec4},
    spirv,
};

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_position = Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, out_frag_color: &mut Vec4) {
    *out_frag_color = Vec4::new(in_color.x, in_color.y, in_color.z, 1.0);
}
