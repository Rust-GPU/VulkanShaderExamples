#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat4, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    #[spirv(clip_distance)] clip_distance: &mut [f32; 1],
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_position = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let eye_pos = ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_eye_pos = eye_pos.truncate();
    *out_light_vec = (ubo.light_pos.truncate() - *out_eye_pos).normalize();

    // Clip against reflection plane
    let clip_plane = vec4(0.0, 0.0, 0.0, 0.0);
    clip_distance[0] = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0).dot(clip_plane);
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

    let i_ambient = vec4(0.1, 0.1, 0.1, 1.0);
    let i_diffuse = vec4(
        in_normal.dot(in_light_vec).max(0.0),
        in_normal.dot(in_light_vec).max(0.0),
        in_normal.dot(in_light_vec).max(0.0),
        in_normal.dot(in_light_vec).max(0.0),
    );
    let specular = 0.75;
    let mut i_specular = vec4(0.0, 0.0, 0.0, 0.0);
    if in_eye_pos.dot(in_normal) < 0.0 {
        let spec_factor = reflected.dot(eye).max(0.0).powf(16.0) * specular;
        i_specular = vec4(0.5, 0.5, 0.5, 1.0) * spec_factor;
    }

    let color_vec4 = vec4(in_color.x, in_color.y, in_color.z, 1.0);
    *out_frag_color = (i_ambient + i_diffuse) * color_vec4 + i_specular;
}
