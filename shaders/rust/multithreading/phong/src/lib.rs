#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{mat3, vec3, vec4, Mat4, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub mvp: Mat4,
    pub color: Vec3,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;

    // If color is red (1.0, 0.0, 0.0), use push constant color
    if in_color.x == 1.0 && in_color.y == 0.0 && in_color.z == 0.0 {
        *out_color = push_consts.color;
    } else {
        *out_color = in_color;
    }

    *out_position = push_consts.mvp * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = push_consts.mvp * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let mvp_mat3 = mat3(
        push_consts.mvp.x_axis.truncate(),
        push_consts.mvp.y_axis.truncate(),
        push_consts.mvp.z_axis.truncate(),
    );
    *out_normal = mvp_mat3 * in_normal;
    let l_pos = vec3(0.0, 0.0, 0.0);
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = -pos.truncate();
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
    let diffuse = n.dot(l).max(0.0) * in_color;
    let specular = r.dot(v).max(0.0).powf(8.0) * vec3(0.75, 0.75, 0.75);
    *out_frag_color = vec4(
        diffuse.x + specular.x,
        diffuse.y + specular.y,
        diffuse.z + specular.z,
        1.0,
    );
}
