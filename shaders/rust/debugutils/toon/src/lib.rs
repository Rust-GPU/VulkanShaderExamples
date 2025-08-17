#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Mat3, Mat4, Vec2, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_normal = Mat3::from_mat4(ubo.model) * in_normal;
    let l_pos =
        Mat3::from_mat4(ubo.model) * vec3(ubo.light_pos.x, ubo.light_pos.y, ubo.light_pos.z);
    *out_light_vec = l_pos - vec3(pos.x, pos.y, pos.z);
    *out_view_vec = -vec3(pos.x, pos.y, pos.z);
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_uv: Vec2,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] _sampler_color_map: &spirv_std::Image!(2D, type=f32, sampled),
    out_frag_color: &mut Vec4,
) {
    // Desaturate color
    let gray = vec3(0.2126, 0.7152, 0.0722).dot(in_color);
    let color = in_color.lerp(vec3(gray, gray, gray), 0.65);

    // High ambient colors because mesh materials are pretty dark
    let ambient = color * 1.0;
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = n.dot(l).max(0.0) * color;
    let specular = r.dot(v).max(0.0).powf(16.0) * vec3(0.75, 0.75, 0.75);
    *out_frag_color = vec4(
        ambient.x + diffuse.x * 1.75 + specular.x,
        ambient.y + diffuse.y * 1.75 + specular.y,
        ambient.z + diffuse.z * 1.75 + specular.z,
        1.0,
    );

    let intensity = n.dot(l);
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

    out_frag_color.x = in_color.x * 3.0 * shade;
    out_frag_color.y = in_color.y * 3.0 * shade;
    out_frag_color.z = in_color.z * 3.0 * shade;
}
