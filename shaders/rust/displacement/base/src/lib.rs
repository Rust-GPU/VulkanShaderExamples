#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Vec2, Vec3, Vec4},
    spirv,
};

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_position = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_uv = in_uv;
    *out_normal = in_normal;
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_uv: Vec2,
    in_eye_pos: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] color_map: &spirv_std::image::SampledImage<
        spirv_std::Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let _n = in_normal.normalize();
    let _l = Vec3::new(1.0, 1.0, 1.0).normalize();

    let color = color_map.sample(in_uv);

    let _eye = (-in_eye_pos).normalize();
    let _reflected = (-in_light_vec).reflect(in_normal).normalize();

    let i_ambient = vec4(0.0, 0.0, 0.0, 1.0);
    let i_diffuse = vec4(1.0, 1.0, 1.0, 1.0) * in_normal.dot(in_light_vec).max(0.0);

    *out_frag_color = (i_ambient + i_diffuse) * vec4(color.x, color.y, color.z, 1.0);
}
