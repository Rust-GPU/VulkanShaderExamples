#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec2, vec3, vec4, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles},
    num_traits::Float,
    spirv, Image, Sampler,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub view_pos: Vec4,
    pub lod_bias: f32,
    pub sampler_index: i32,
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
    *out_uv = in_uv * vec2(2.0, 1.0);
    *out_lod_bias = ubo.lod_bias;

    let world_pos = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).xyz();

    *out_position = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    *out_normal = Mat3::from_mat4(ubo.model.inverse().transpose()) * in_normal;
    let light_pos = vec3(-30.0, 0.0, 0.0);
    *out_light_vec = world_pos - light_pos;
    *out_view_vec = ubo.view_pos.xyz() - world_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_lod_bias: f32,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(descriptor_set = 0, binding = 1)] texture_color: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] samplers: &[Sampler; 3],
    out_frag_color: &mut Vec4,
) {
    let sampler = &samplers[ubo.sampler_index as usize];
    let color = texture_color.sample_bias(*sampler, in_uv, in_lod_bias);

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = l.reflect(n);
    let diffuse = n.dot(l).max(0.65) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * color.w;
    let final_color = diffuse * color.xyz() + vec3(specular, specular, specular);
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
