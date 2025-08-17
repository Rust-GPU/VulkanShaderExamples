#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec3, vec4, Mat3, Mat4, Vec3, Vec3Swizzles, Vec4},
    num_traits::Float,
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub normal: Mat4,
    pub view: Mat4,
    pub tex_index: i32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_normal: &mut Vec3,
    #[spirv(flat)] out_tex_index: &mut i32,
) {
    *out_color = in_color;
    let model_view = ubo.view * ubo.model;
    *out_eye_pos = (model_view * in_pos).truncate().normalize();
    *out_tex_index = ubo.tex_index;

    let normal_mat3 = Mat3::from_cols(
        ubo.normal.x_axis.truncate(),
        ubo.normal.y_axis.truncate(),
        ubo.normal.z_axis.truncate(),
    );
    *out_normal = (normal_mat3 * in_normal).normalize();

    let r = out_eye_pos.reflect(*out_normal);
    let _m = 2.0 * (r.x.powi(2) + r.y.powi(2) + (r.z + 1.0).powi(2)).sqrt();
    *out_position = ubo.projection * model_view * in_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_color: Vec3,
    in_eye_pos: Vec3,
    in_normal: Vec3,
    #[spirv(flat)] in_tex_index: i32,
    #[spirv(descriptor_set = 0, binding = 1)] mat_cap: &SampledImage<
        Image!(2D, type=f32, sampled, arrayed),
    >,
    out_frag_color: &mut Vec4,
) {
    let r = in_eye_pos.reflect(in_normal);
    let r2 = vec3(r.x, r.y, r.z + 1.0);
    let m = 2.0 * r2.length();
    let v_n = r.xy() / m + 0.5;

    let tex_coord = vec3(v_n.x, v_n.y, in_tex_index as f32);
    let sampled_color = mat_cap.sample(tex_coord);

    *out_frag_color = vec4(
        sampled_color.x * (in_color.x * 2.0).clamp(0.0, 1.0),
        sampled_color.y * (in_color.x * 2.0).clamp(0.0, 1.0),
        sampled_color.z * (in_color.x * 2.0).clamp(0.0, 1.0),
        1.0,
    );
}
