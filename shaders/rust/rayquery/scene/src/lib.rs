#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]
#![feature(asm_experimental_arch)]

use spirv_std::ray_tracing::{AccelerationStructure, CommittedIntersection, RayFlags, RayQuery};
use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec2, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_pos: Vec3,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    _in_uv: Vec2,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_world_pos: &mut Vec3,
) {
    *out_color = in_color;
    *out_position = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_world_pos = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).truncate();
    let model_mat3 = Mat3::from_cols(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    *out_normal = model_mat3 * in_normal;
    *out_light_vec = (ubo.light_pos - in_pos).normalize();
    *out_view_vec = -pos.truncate();
}

const AMBIENT: f32 = 0.1;

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    _in_view_vec: Vec3,
    in_light_vec: Vec3,
    in_world_pos: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] top_level_as: &AccelerationStructure,
    out_frag_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let diffuse = n.dot(l).max(AMBIENT) * in_color;

    *out_frag_color = vec4(diffuse.x, diffuse.y, diffuse.z, 1.0);

    unsafe {
        spirv_std::ray_query!(let mut ray_query);
        ray_query.initialize(
            top_level_as,
            RayFlags::TERMINATE_ON_FIRST_HIT,
            0xFF,
            in_world_pos,
            0.01,
            l,
            1000.0,
        );

        // Traverse the acceleration structure
        ray_query.proceed();

        // If the intersection has hit a triangle, the fragment is shadowed
        if ray_query.get_committed_intersection_type() == CommittedIntersection::Triangle {
            *out_frag_color *= 0.1;
        }
    }
}
