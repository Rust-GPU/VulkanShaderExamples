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
    pub lod_bias: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_lod_bias: &mut f32,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_uv = in_uv;
    *out_lod_bias = ubo.lod_bias;

    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let model_mat3 = mat3(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    *out_normal = (ubo.model.inverse().transpose()
        * vec4(in_normal.x, in_normal.y, in_normal.z, 0.0))
    .truncate();
    let light_pos = vec3(0.0, 0.0, 0.0);
    let l_pos = model_mat3 * light_pos;
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = ubo.view_pos.truncate() - pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_lod_bias: f32,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] color_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let color = color_sampler.sample(in_uv);

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = n.dot(l).max(0.0) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * color.w;

    let final_color = diffuse * color.truncate() + vec3(specular, specular, specular);
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
