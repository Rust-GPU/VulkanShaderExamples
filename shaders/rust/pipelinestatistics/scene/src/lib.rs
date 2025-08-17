#![no_std]

use spirv_std::glam::{Mat3, Mat4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub light_pos: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub obj_pos: Vec3,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;

    let _loc_pos = (ubo.modelview * Vec4::from((in_pos, 1.0))).xyz();
    let world_pos = (ubo.modelview * Vec4::from((in_pos + push_consts.obj_pos, 1.0))).xyz();
    *out_position = ubo.projection * Vec4::from((world_pos, 1.0));

    let pos = ubo.modelview * Vec4::from((world_pos, 1.0));
    *out_normal = Mat3::from_mat4(ubo.modelview) * in_normal;
    *out_light_vec = ubo.light_pos.xyz() - pos.xyz();
    *out_view_vec = -pos.xyz();
}

#[spirv(tessellation_control(output_vertices = 3))]
pub fn main_tcs(
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_color: [Vec3; 3],
    in_view_vec: [Vec3; 3],
    in_light_vec: [Vec3; 3],
    #[spirv(position)] out_position: &mut [Vec4; 3],
    out_normal: &mut [Vec3; 3],
    out_color: &mut [Vec3; 3],
    out_view_vec: &mut [Vec3; 3],
    out_light_vec: &mut [Vec3; 3],
    #[spirv(tess_level_inner)] tess_level_inner: &mut [f32; 2],
    #[spirv(tess_level_outer)] tess_level_outer: &mut [f32; 4],
) {
    if invocation_id == 0 {
        tess_level_inner[0] = 2.0;
        tess_level_outer[0] = 1.0;
        tess_level_outer[1] = 1.0;
        tess_level_outer[2] = 1.0;
    }

    let idx = invocation_id as usize;
    out_position[idx] = in_position[idx];
    out_normal[idx] = in_normal[idx];
    out_color[idx] = in_color[idx];
    out_view_vec[idx] = in_view_vec[idx];
    out_light_vec[idx] = in_light_vec[idx];
}

#[spirv(tessellation_evaluation(triangles))]
pub fn main_tes(
    #[spirv(tess_coord)] tess_coord: Vec3,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_color: [Vec3; 3],
    in_view_vec: [Vec3; 3],
    in_light_vec: [Vec3; 3],
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_position = tess_coord.x * in_position[2]
        + tess_coord.y * in_position[1]
        + tess_coord.z * in_position[0];
    *out_normal =
        tess_coord.x * in_normal[2] + tess_coord.y * in_normal[1] + tess_coord.z * in_normal[0];
    *out_view_vec = tess_coord.x * in_view_vec[2]
        + tess_coord.y * in_view_vec[1]
        + tess_coord.z * in_view_vec[0];
    *out_light_vec = tess_coord.x * in_light_vec[2]
        + tess_coord.y * in_light_vec[1]
        + tess_coord.z * in_light_vec[0];
    *out_color = in_color[0];
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = n.dot(l).max(0.0) * in_color;
    let specular = r.dot(v).max(0.0).powi(8) * Vec3::splat(0.75);
    *out_frag_color = Vec4::from((diffuse + specular, 0.5));
}
