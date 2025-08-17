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
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub matrix: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushBlock {
    pub base_color_factor: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    _in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] node: &Node,
    #[spirv(push_constant)] material: &PushBlock,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = material.base_color_factor.truncate();
    let pos = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_position = ubo.projection * ubo.view * ubo.model * node.matrix * pos;

    let normal_matrix = Mat3::from_mat4(ubo.view * ubo.model * node.matrix);
    *out_normal = normal_matrix * in_normal;

    let localpos = ubo.view * ubo.model * node.matrix * pos;
    let light_pos = vec3(10.0, -10.0, 10.0);
    *out_light_vec = light_pos - localpos.truncate();
    *out_view_vec = -localpos.truncate();
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
    let ambient = vec3(0.1, 0.1, 0.1);
    let diffuse = n.dot(l).max(0.0) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * vec3(0.75, 0.75, 0.75);
    let final_color = (ambient + diffuse) * in_color + specular;
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
