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
pub struct UboScene {
    pub projection: Mat4,
    pub view: Mat4,
    pub light_pos: Vec4,
    pub view_pos: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo_scene: &UboScene,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo_scene.projection
        * ubo_scene.view
        * push_consts.model
        * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo_scene.view * push_consts.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let view_mat3 = mat3(
        ubo_scene.view.x_axis.truncate(),
        ubo_scene.view.y_axis.truncate(),
        ubo_scene.view.z_axis.truncate(),
    );
    *out_normal = view_mat3 * in_normal;
    *out_light_vec = ubo_scene.light_pos.truncate() - pos.truncate();
    *out_view_vec = ubo_scene.view_pos.truncate() - pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_uv: Vec2,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 1, binding = 0)] sampler_color_map: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let color = sampler_color_map.sample(in_uv) * vec4(in_color.x, in_color.y, in_color.z, 1.0);

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = l.reflect(n);
    let diffuse = n.dot(l).max(0.15) * in_color;
    let specular = v.dot(r).max(0.0).powf(16.0) * vec3(0.75, 0.75, 0.75);
    *out_frag_color = vec4(
        diffuse.x * color.x + specular.x,
        diffuse.y * color.y + specular.y,
        diffuse.z * color.z + specular.z,
        1.0,
    );
}
