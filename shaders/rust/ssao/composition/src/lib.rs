#![no_std]

use spirv_std::glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub _dummy: Mat4,
    pub ssao: i32,
    pub ssao_only: i32,
    pub ssao_blur: i32,
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_position: &Sampler,
    #[spirv(descriptor_set = 0, binding = 0)] texture_position: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler_normal: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] texture_normal: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] sampler_albedo: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] texture_albedo: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 3)] sampler_ssao: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] texture_ssao: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 4)] sampler_ssao_blur: &Sampler,
    #[spirv(descriptor_set = 0, binding = 4)] texture_ssao_blur: &Image!(2D, type=f32, sampled),
    #[spirv(uniform, descriptor_set = 0, binding = 5)] ubo_params: &UBO,
    out_frag_color: &mut Vec4,
) {
    let frag_pos = texture_position.sample(*sampler_position, in_uv).xyz();
    let normal = (texture_normal.sample(*sampler_normal, in_uv).xyz() * 2.0 - 1.0).normalize();
    let albedo = texture_albedo.sample(*sampler_albedo, in_uv);

    let ssao = if ubo_params.ssao_blur == 1 {
        texture_ssao_blur.sample(*sampler_ssao_blur, in_uv).x
    } else {
        texture_ssao.sample(*sampler_ssao, in_uv).x
    };

    let light_pos = Vec3::ZERO;
    let l = (light_pos - frag_pos).normalize();
    let n_dot_l = normal.dot(l).max(0.5);

    if ubo_params.ssao_only == 1 {
        *out_frag_color = vec4(ssao, ssao, ssao, 1.0);
    } else {
        let base_color = albedo.xyz() * n_dot_l;

        if ubo_params.ssao == 1 {
            let mut color = vec3(ssao, ssao, ssao);

            if ubo_params.ssao_only != 1 {
                color *= base_color;
            }
            *out_frag_color = vec4(color.x, color.y, color.z, 1.0);
        } else {
            *out_frag_color = vec4(base_color.x, base_color.y, base_color.z, 1.0);
        }
    }
}
