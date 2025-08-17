#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Mat3, Mat4, Vec2, Vec3, Vec4},
    image::SampledImage,
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
pub struct PushConstants {
    pub model: Mat4,
    pub alpha_mask: u32,
    pub alpha_mask_cutoff: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_tangent: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo_scene: &UboScene,
    #[spirv(push_constant)] push_consts: &PushConstants,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_tangent: &mut Vec4,
) {
    *out_normal = in_normal;
    *out_uv = in_uv;
    *out_tangent = in_tangent;
    *out_position = ubo_scene.projection
        * ubo_scene.view
        * push_consts.model
        * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    *out_normal = Mat3::from_mat4(push_consts.model) * in_normal;
    let pos = push_consts.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_light_vec = vec3(
        ubo_scene.light_pos.x,
        ubo_scene.light_pos.y,
        ubo_scene.light_pos.z,
    ) - vec3(pos.x, pos.y, pos.z);
    *out_view_vec = vec3(
        ubo_scene.view_pos.x,
        ubo_scene.view_pos.y,
        ubo_scene.view_pos.z,
    ) - vec3(pos.x, pos.y, pos.z);
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_uv: Vec2,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    in_tangent: Vec4,
    #[spirv(descriptor_set = 1, binding = 0)] sampler_color_map: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 1, binding = 1)] sampler_normal_map: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(push_constant)] push_consts: &PushConstants,
    out_frag_color: &mut Vec4,
) {
    let color: Vec4 = sampler_color_map.sample(in_uv);

    if push_consts.alpha_mask == 1 {
        if color.w < push_consts.alpha_mask_cutoff {
            spirv_std::arch::kill();
        }
    }

    let mut n = in_normal.normalize();
    let t = vec3(in_tangent.x, in_tangent.y, in_tangent.z).normalize();
    let b = in_normal.cross(vec3(in_tangent.x, in_tangent.y, in_tangent.z)) * in_tangent.w;
    let tbn = Mat3::from_cols(t, b, n);
    let normal_sample: Vec4 = sampler_normal_map.sample(in_uv);
    n = tbn
        * (vec3(normal_sample.x, normal_sample.y, normal_sample.z) * 2.0 - vec3(1.0, 1.0, 1.0))
            .normalize();

    const AMBIENT: f32 = 0.1;
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse_factor = n.dot(l).max(AMBIENT);
    let diffuse = vec3(diffuse_factor, diffuse_factor, diffuse_factor);
    let specular = r.dot(v).max(0.0).powf(32.0);

    *out_frag_color = vec4(
        diffuse.x * color.x + specular,
        diffuse.y * color.y + specular,
        diffuse.z * color.z + specular,
        color.w,
    );
}
