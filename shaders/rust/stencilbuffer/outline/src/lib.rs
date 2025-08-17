#![no_std]

use spirv_std::glam::{Mat4, Vec3, Vec4};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
    pub outline_width: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec4,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position, invariant)] out_position: &mut Vec4,
) {
    // Extrude along normal
    let pos = Vec4::new(
        in_pos.x + in_normal.x * ubo.outline_width,
        in_pos.y + in_normal.y * ubo.outline_width,
        in_pos.z + in_normal.z * ubo.outline_width,
        in_pos.w,
    );
    *out_position = ubo.projection * ubo.model * pos;
}

#[spirv(fragment)]
pub fn main_fs(out_frag_color: &mut Vec4) {
    *out_frag_color = Vec4::new(1.0, 1.0, 1.0, 1.0);
}
