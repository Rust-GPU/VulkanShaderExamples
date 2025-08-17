#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles},
    image::SampledImage,
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
    *out_uv = in_tex_coord;
    *out_normal = (Mat3::from_mat4(ubo.normal) * in_normal).normalize();
    *out_color = in_color;
    let model_view = ubo.view * ubo.model;
    let pos = model_view * in_pos;
    *out_position = ubo.projection * pos;
    *out_eye_pos = (model_view * pos).xyz();
    let light_pos = model_view * vec4(ubo.lightpos.x, ubo.lightpos.y, ubo.lightpos.z, 1.0);
    *out_light_vec = (light_pos.xyz() - *out_eye_pos).normalize();
}

fn specpart(l: Vec3, n: Vec3, h: Vec3) -> f32 {
    if n.dot(l) > 0.0 {
        h.dot(n).clamp(0.0, 1.0).powf(64.0)
    } else {
        0.0
    }
}

#[spirv(fragment)]
pub fn main_fs(
    _in_uv: Vec2,
    in_normal: Vec3,
    in_color: Vec3,
    in_eye_pos: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] _tex: &SampledImage<Image!(2D, type=f32, sampled)>,
    out_frag_color: &mut Vec4,
) {
    let eye = (-in_eye_pos).normalize();
    let reflected = (-in_light_vec).reflect(in_normal).normalize();

    let half_vec = (in_light_vec + in_eye_pos).normalize();
    let diff = in_light_vec.dot(in_normal).clamp(0.0, 1.0);
    let spec = specpart(in_light_vec, in_normal, half_vec);
    let intensity = 0.1 + diff + spec;

    let i_ambient = vec4(0.2, 0.2, 0.2, 1.0);
    let i_diffuse = vec4(0.5, 0.5, 0.5, 0.5) * in_normal.dot(in_light_vec).max(0.0);
    let shininess = 0.75;
    let i_specular = vec4(0.5, 0.5, 0.5, 1.0) * reflected.dot(eye).max(0.0).powf(2.0) * shininess;

    *out_frag_color =
        (i_ambient + i_diffuse) * vec4(in_color.x, in_color.y, in_color.z, 1.0) + i_specular;

    // Some manual saturation
    if intensity > 0.95 {
        *out_frag_color *= 2.25;
    }
    if intensity < 0.15 {
        *out_frag_color = vec4(0.1, 0.1, 0.1, 0.1);
    }
}
