#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Mat3, Mat4, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: [Mat4; 2],
    pub modelview: [Mat4; 2],
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(view_index)] view_index: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_color = in_color;
    *out_normal = Mat3::from_mat4(ubo.modelview[view_index as usize]) * in_normal;

    let pos = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let world_pos = ubo.modelview[view_index as usize] * pos;

    let l_pos = ubo.modelview[view_index as usize] * ubo.light_pos;
    *out_light_vec = vec3(l_pos.x, l_pos.y, l_pos.z) - vec3(world_pos.x, world_pos.y, world_pos.z);
    *out_view_vec = -vec3(world_pos.x, world_pos.y, world_pos.z);

    *out_position = ubo.projection[view_index as usize] * world_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let ambient = vec3(0.1, 0.1, 0.1);
    let diffuse = n.dot(l).max(0.0) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * vec3(0.75, 0.75, 0.75);
    *out_color = vec4(
        (ambient.x + diffuse.x) * in_color.x + specular.x,
        (ambient.y + diffuse.y) * in_color.y + specular.y,
        (ambient.z + diffuse.z) * in_color.z + specular.z,
        1.0,
    );
}
