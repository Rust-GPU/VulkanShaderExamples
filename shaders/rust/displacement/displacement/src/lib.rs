#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat4, Vec2, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub light_pos: Vec4,
    pub tess_alpha: f32,
    pub tess_strength: f32,
    pub tess_level: f32,
}

#[spirv(tessellation_control(output_vertices = 3))]
pub fn main_tcs(
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_uv: [Vec2; 3],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut [Vec4; 3],
    out_normal: &mut [Vec3; 3],
    out_uv: &mut [Vec2; 3],
    #[spirv(tess_level_inner)] tess_level_inner: &mut [f32; 2],
    #[spirv(tess_level_outer)] tess_level_outer: &mut [f32; 4],
) {
    if invocation_id == 0 {
        tess_level_inner[0] = ubo.tess_level;
        tess_level_outer[0] = ubo.tess_level;
        tess_level_outer[1] = ubo.tess_level;
        tess_level_outer[2] = ubo.tess_level;
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
    #[spirv(descriptor_set = 0, binding = 1)] displacement_map: &spirv_std::image::SampledImage<
        spirv_std::Image!(2D, type=f32, sampled),
    >,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_eye_pos: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    let pos = tess_coord.x * in_position[0]
        + tess_coord.y * in_position[1]
        + tess_coord.z * in_position[2];
    *out_uv = tess_coord.x * in_uv[0] + tess_coord.y * in_uv[1] + tess_coord.z * in_uv[2];
    *out_normal =
        tess_coord.x * in_normal[0] + tess_coord.y * in_normal[1] + tess_coord.z * in_normal[2];

    let displacement = displacement_map.sample_by_lod(*out_uv, 0.0).w.max(0.0) * ubo.tess_strength;
    let normal_normalized = out_normal.normalize();
    let displaced_pos = pos
        + vec4(
            normal_normalized.x,
            normal_normalized.y,
            normal_normalized.z,
            0.0,
        ) * displacement;

    *out_eye_pos = displaced_pos.truncate();
    *out_light_vec = (ubo.light_pos.truncate() - *out_eye_pos).normalize();

    *out_position = ubo.projection * ubo.modelview * displaced_pos;
}
