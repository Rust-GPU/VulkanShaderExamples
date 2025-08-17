#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{mat3, vec3, vec4, Mat4, Vec2, Vec3, Vec4},
    num_traits::Float,
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub view_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_uv = in_uv;

    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let inverse_transpose_model = ubo.model.inverse().transpose();
    let normal_mat3 = mat3(
        inverse_transpose_model.x_axis.truncate(),
        inverse_transpose_model.y_axis.truncate(),
        inverse_transpose_model.z_axis.truncate(),
    );
    *out_normal = normal_mat3 * in_normal;
    let light_pos = vec3(0.0, 0.0, 0.0);
    let model_mat3 = mat3(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    let l_pos = model_mat3 * light_pos;
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = ubo.view_pos.truncate() - pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 1, binding = 0)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let color = sampler_color.sample(in_uv);

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = n.dot(l).max(0.0) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * color.w;

    *out_frag_color = vec4(
        diffuse.x * color.x + specular,
        diffuse.y * color.y + specular,
        diffuse.z * color.z + specular,
        1.0,
    );
}
