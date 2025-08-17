#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{mat3, vec3, vec4, Mat3, Mat4, Vec2, Vec3, Vec4},
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
    in_tangent: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo_scene: &UboScene,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_tangent: &mut Vec4,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = in_uv;
    *out_tangent = in_tangent;
    *out_position = ubo_scene.projection
        * ubo_scene.view
        * push_consts.model
        * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let model_mat3 = mat3(
        push_consts.model.x_axis.truncate(),
        push_consts.model.y_axis.truncate(),
        push_consts.model.z_axis.truncate(),
    );
    *out_normal = model_mat3 * in_normal;
    let pos = push_consts.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_light_vec = ubo_scene.light_pos.truncate() - pos.truncate();
    *out_view_vec = ubo_scene.view_pos.truncate() - pos.truncate();
}

#[cfg_attr(target_arch = "spirv", spirv(fragment))]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
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
    #[spirv(spec_constant(id = 0, default = 0))] alpha_mask: u32,
    #[spirv(spec_constant(id = 1, default = 0))] alpha_mask_cutoff_bits: u32,
    out_frag_color: &mut Vec4,
) {
    let color = sampler_color_map.sample(in_uv) * vec4(in_color.x, in_color.y, in_color.z, 1.0);

    let alpha_mask_enabled = alpha_mask != 0;
    let alpha_mask_cutoff = f32::from_bits(alpha_mask_cutoff_bits);

    if alpha_mask_enabled {
        if color.w < alpha_mask_cutoff {
            #[cfg(target_arch = "spirv")]
            spirv_std::arch::kill();
        }
    }

    let mut n = in_normal.normalize();
    let t = in_tangent.truncate().normalize();
    let b = in_normal.cross(in_tangent.truncate()) * in_tangent.w;
    let tbn = Mat3::from_cols(t, b, n);
    let normal_sample = sampler_normal_map.sample(in_uv).truncate() * 2.0 - vec3(1.0, 1.0, 1.0);
    n = tbn * normal_sample.normalize();

    const AMBIENT: f32 = 0.1;
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = vec3(
        n.dot(l).max(AMBIENT),
        n.dot(l).max(AMBIENT),
        n.dot(l).max(AMBIENT),
    );
    let specular = r.dot(v).max(0.0).powf(32.0);
    *out_frag_color = vec4(
        diffuse.x * color.x + specular,
        diffuse.y * color.y + specular,
        diffuse.z * color.z + specular,
        color.w,
    );
}
