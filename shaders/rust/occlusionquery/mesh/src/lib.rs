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
    pub color: Vec4,
    pub light_pos: Vec4,
    pub visible: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_visible: &mut f32,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color * ubo.color.truncate();
    *out_visible = ubo.visible;

    *out_position = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let model_mat3 = Mat3::from_cols(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    *out_normal = model_mat3 * in_normal;
    *out_light_vec = ubo.light_pos.truncate() - pos.truncate();
    *out_view_vec = -pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_visible: f32,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    if in_visible > 0.0 {
        let n = in_normal.normalize();
        let l = in_light_vec.normalize();
        let v = in_view_vec.normalize();
        let r = (-l).reflect(n);
        let diffuse = n.dot(l).max(0.25) * in_color;
        let specular = r.dot(v).max(0.0).powf(8.0) * vec3(0.75, 0.75, 0.75);
        *out_frag_color = vec4(
            diffuse.x + specular.x,
            diffuse.y + specular.y,
            diffuse.z + specular.z,
            1.0,
        );
    } else {
        *out_frag_color = vec4(0.1, 0.1, 0.1, 1.0);
    }
}
