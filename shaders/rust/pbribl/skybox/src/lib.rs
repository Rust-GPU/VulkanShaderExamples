#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use spirv_std::image::{Cubemap, SampledImage};
use spirv_std::{num_traits::Float, spirv};

// UBO structure for skybox matrices
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UBO {
    projection: Mat4,
    model: Mat4,
}

// UBO structure for lighting parameters
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UBOParams {
    lights: [Vec4; 4], // offset 0, size 64
    exposure: f32,     // offset 64
    gamma: f32,        // offset 68
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    _in_normal: Vec3, // Unused but needed to match GLSL layout
    _in_uv: Vec2,     // Unused but needed to match GLSL layout
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    *out_pos = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

// Uncharted 2 tone mapping
fn uncharted2_tonemap(color: Vec3) -> Vec3 {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;

    ((color * (a * color + c * b) + d * e) / (color * (a * color + b) + d * f)) - e / f
}

#[spirv(fragment)]
pub fn main_fs(
    in_uvw: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo_params: &UBOParams,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_env: &SampledImage<Cubemap>,
    out_color: &mut Vec4,
) {
    let mut color = sampler_env.sample(in_uvw).truncate();

    // Tone mapping
    color = uncharted2_tonemap(color * ubo_params.exposure);
    let white_scale = Vec3::ONE / uncharted2_tonemap(Vec3::splat(11.2));
    color = color * white_scale;

    // Gamma correction
    let inv_gamma = 1.0 / ubo_params.gamma;
    color = vec3(
        color.x.powf(inv_gamma),
        color.y.powf(inv_gamma),
        color.z.powf(inv_gamma),
    );

    *out_color = vec4(color.x, color.y, color.z, 1.0);
}
