#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Mat3, Mat4, Vec2, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_uv: Vec2,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo.projection * ubo.view * ubo.model * in_pos;

    let light_pos = Vec3::new(-5.0, -5.0, 0.0);
    let pos = ubo.view * ubo.model * in_pos;
    let normal_matrix = Mat3::from_mat4(ubo.view * ubo.model);
    *out_normal = normal_matrix * in_normal;
    *out_light_vec = light_pos - Vec3::new(pos.x, pos.y, pos.z);
    *out_view_vec = -Vec3::new(pos.x, pos.y, pos.z);
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    let mut ambient = Vec3::ZERO;

    // Adjust light calculations for glow color
    if in_color.x >= 0.9 || in_color.y >= 0.9 || in_color.z >= 0.9 {
        ambient = in_color * 0.25;
    }

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = n.dot(l).max(0.0) * in_color;
    let specular = r.dot(v).max(0.0).powf(8.0) * Vec3::new(0.75, 0.75, 0.75);

    *out_frag_color = Vec4::new(
        ambient.x + diffuse.x + specular.x,
        ambient.y + diffuse.y + specular.y,
        ambient.z + diffuse.z + specular.z,
        1.0,
    );
}
