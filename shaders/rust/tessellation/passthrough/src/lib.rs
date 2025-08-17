#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub tess_alpha: f32,
    pub tess_level: f32,
}

#[spirv(tessellation_control(output_vertices = 3))]
pub fn main_tcs(
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_uv: [Vec2; 3],
    #[spirv(position)] out_position: &mut [Vec4; 3],
    out_normal: &mut [Vec3; 3],
    out_uv: &mut [Vec2; 3],
    #[spirv(tess_level_inner)] tess_level_inner: &mut [f32; 2],
    #[spirv(tess_level_outer)] tess_level_outer: &mut [f32; 4],
) {
    if invocation_id == 0 {
        tess_level_inner[0] = 1.0;
        tess_level_outer[0] = 1.0;
        tess_level_outer[1] = 1.0;
        tess_level_outer[2] = 1.0;
    }

    out_position[invocation_id as usize] = in_position[invocation_id as usize];
    out_normal[invocation_id as usize] = in_normal[invocation_id as usize];
    out_uv[invocation_id as usize] = in_uv[invocation_id as usize];
}

#[spirv(tessellation_evaluation(triangles, spacing_equal, vertex_order_cw))]
pub fn main_tes(
    #[spirv(tess_coord)] tess_coord: Vec3,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_uv: [Vec2; 3],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    let pos = tess_coord.x * in_position[0]
        + tess_coord.y * in_position[1]
        + tess_coord.z * in_position[2];

    *out_position = ubo.projection * ubo.model * pos;

    *out_normal =
        tess_coord.x * in_normal[0] + tess_coord.y * in_normal[1] + tess_coord.z * in_normal[2];

    *out_uv = tess_coord.x * in_uv[0] + tess_coord.y * in_uv[1] + tess_coord.z * in_uv[2];
}
