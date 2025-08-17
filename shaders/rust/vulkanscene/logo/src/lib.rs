#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub normal: Mat4,
    pub view: Mat4,
    pub lightpos: Vec3,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_normal: Vec3,
    in_tex_coord: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    let model_view = ubo.view * ubo.model;
    let pos = model_view * in_pos;
    *out_uv = in_tex_coord;
    *out_normal = (Mat3::from_mat4(ubo.normal) * in_normal).normalize();
    *out_color = in_color;
    *out_position = ubo.projection * pos;
    *out_eye_pos = (model_view * pos).xyz();
    let light_pos = model_view * vec4(1.0, 2.0, 0.0, 1.0);
    *out_light_vec = (light_pos.xyz() - *out_eye_pos).normalize();
}

#[spirv(fragment)]
pub fn main_fs(
    _in_uv: Vec2,
    in_normal: Vec3,
    in_color: Vec3,
    in_eye_pos: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    let eye = (-in_eye_pos).normalize();
    let reflected = (-in_light_vec).reflect(in_normal).normalize();

    let diff = vec4(in_color.x, in_color.y, in_color.z, 1.0) * in_normal.dot(in_light_vec).max(0.0);
    let shininess = 0.0;
    let spec = vec4(1.0, 1.0, 1.0, 1.0) * reflected.dot(eye).max(0.0).powf(2.5) * shininess;

    *out_frag_color = diff + spec;
    out_frag_color.w = 1.0;
}
