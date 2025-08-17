#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::num_traits::Float;
use spirv_std::{
    glam::{Vec3, Vec4},
    spirv,
};

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
) {
    *out_color = in_color;
    *out_normal = in_normal;
    *out_position = Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let ambient = Vec3::splat(0.1);
    let diffuse = n.dot(l).max(0.0) * Vec3::ONE;
    let specular = r.dot(v).max(0.0).powf(16.0) * Vec3::splat(0.75);
    *out_color = Vec4::new(
        ((ambient + diffuse) * in_color + specular).x,
        ((ambient + diffuse) * in_color + specular).y,
        ((ambient + diffuse) * in_color + specular).z,
        1.0,
    );
}
