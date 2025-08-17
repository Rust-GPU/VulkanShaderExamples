#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::num_traits::Float;
use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub lightpos: Vec4,
    pub model: [Mat4; 3],
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(instance_index)] instance_index: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    let model = ubo.model[instance_index as usize];
    *out_normal = (Mat3::from_mat4(model) * in_normal).normalize();
    *out_color = in_color;

    let model_view = ubo.view * model;
    let pos = model_view * in_pos;
    *out_eye_pos = pos.truncate();

    let light_pos = model_view * vec4(ubo.lightpos.x, ubo.lightpos.y, ubo.lightpos.z, 1.0);
    *out_light_vec = (light_pos.truncate() - *out_eye_pos).normalize();

    *out_position = ubo.projection * pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_eye_pos: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    let eye = (-in_eye_pos).normalize();
    let reflected = (-in_light_vec).reflect(in_normal).normalize();

    let i_ambient = vec4(0.2, 0.2, 0.2, 1.0);
    let i_diffuse = vec4(0.5, 0.5, 0.5, 0.5) * in_normal.dot(in_light_vec).max(0.0);
    let specular = 0.25;
    let i_specular = vec4(0.5, 0.5, 0.5, 1.0) * reflected.dot(eye).max(0.0).powf(0.8) * specular;

    *out_frag_color =
        (i_ambient + i_diffuse) * vec4(in_color.x, in_color.y, in_color.z, 1.0) + i_specular;
}
