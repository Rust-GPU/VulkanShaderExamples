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
}

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

#[spirv(geometry(input_lines = 3, output_points = 6))]
pub fn main_gs(
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
) {
    let normal_length = 0.02;

    for i in 0..3 {
        let pos = in_position[i].truncate();
        let normal = in_normal[i];

        *out_position = ubo.projection * (ubo.model * Vec4::new(pos.x, pos.y, pos.z, 1.0));
        *out_color = Vec3::new(1.0, 0.0, 0.0);
        unsafe {
            spirv_std::arch::emit_vertex();
        }

        let end_pos = pos + normal * normal_length;
        *out_position =
            ubo.projection * (ubo.model * Vec4::new(end_pos.x, end_pos.y, end_pos.z, 1.0));
        *out_color = Vec3::new(0.0, 0.0, 1.0);
        unsafe {
            spirv_std::arch::emit_vertex();
        }

        unsafe {
            spirv_std::arch::end_primitive();
        }
    }
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, out_frag_color: &mut Vec4) {
    *out_frag_color = Vec4::new(in_color.x, in_color.y, in_color.z, 1.0);
}
