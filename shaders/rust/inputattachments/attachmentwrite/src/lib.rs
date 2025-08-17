#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_position = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_color = in_color;
    *out_normal = in_normal;
    *out_light_vec = vec3(0.0, 5.0, 15.0) - in_pos;
    *out_view_vec = -in_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_color: Vec3,
    in_normal: Vec3,
    _in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_color: &mut Vec4,
) {
    // Toon shading color attachment output
    let intensity = in_normal.normalize().dot(in_light_vec.normalize());
    let mut shade = 1.0;
    if intensity < 0.5 {
        shade = 0.75;
    }
    if intensity < 0.35 {
        shade = 0.6;
    }
    if intensity < 0.25 {
        shade = 0.5;
    }
    if intensity < 0.1 {
        shade = 0.25;
    }

    *out_color = vec4(
        in_color.x * 3.0 * shade,
        in_color.y * 3.0 * shade,
        in_color.z * 3.0 * shade,
        1.0,
    );

    // Depth attachment does not need to be explicitly written
}
