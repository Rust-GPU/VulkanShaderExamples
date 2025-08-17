#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec2, vec3, vec4, Mat4, UVec2, Vec2, Vec3, Vec4};
use spirv_std::image::{Cubemap, SampledImage};
use spirv_std::{num_traits::Float, spirv};

// Push constants with padding to match GLSL layout
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PushConsts {
    _padding: [f32; 16], // offset 0-63 (64 bytes)
    roughness: f32,      // offset 64
    num_samples: u32,    // offset 68
}

use core::f32::consts::{PI, TAU};

// Vertex shader push constants
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PushConstsVertex {
    mvp: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(push_constant)] push_consts: &PushConstsVertex,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    *out_pos = push_consts.mvp * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

// Random function
fn random(co: Vec2) -> f32 {
    let a = 12.9898;
    let b = 78.233;
    let c = 43758.5453;
    let dt = co.dot(vec2(a, b));
    let sn = dt % 3.14;
    let val = sn.sin() * c;
    val - val.floor()
}

// Hammersley 2D sequence
fn hammersley2d(i: u32, n: u32) -> Vec2 {
    let mut bits = (i << 16) | (i >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    let rdi = (bits as f32) * 2.3283064365386963e-10;
    vec2((i as f32) / (n as f32), rdi)
}

// Importance sampling for GGX
fn importance_sample_ggx(xi: Vec2, roughness: f32, normal: Vec3) -> Vec3 {
    let alpha = roughness * roughness;
    let phi = TAU * xi.x + random(vec2(normal.x, normal.z)) * 0.1;
    let cos_theta = ((1.0 - xi.y) / (1.0 + (alpha * alpha - 1.0) * xi.y)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let h = vec3(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);

    // Tangent space
    let up = if normal.z.abs() < 0.999 {
        vec3(0.0, 0.0, 1.0)
    } else {
        vec3(1.0, 0.0, 0.0)
    };
    let tangent_x = up.cross(normal).normalize();
    let tangent_y = normal.cross(tangent_x).normalize();

    // Convert to world space
    (tangent_x * h.x + tangent_y * h.y + normal * h.z).normalize()
}

// Normal Distribution function (GGX)
fn d_ggx(dot_nh: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let denom = dot_nh * dot_nh * (alpha2 - 1.0) + 1.0;
    alpha2 / (PI * denom * denom)
}

// Prefilter environment map
fn prefilter_env_map(
    r: Vec3,
    roughness: f32,
    num_samples: u32,
    sampler_env: &SampledImage<Cubemap>,
) -> Vec3 {
    let n = r;
    let v = r;
    let mut color = Vec3::ZERO;
    let mut total_weight = 0.0;

    // Get environment map dimensions
    // For cubemaps, query_size_lod returns a UVec2 with the dimensions of one face
    let env_map_size: UVec2 = sampler_env.query_size_lod(0);
    let env_map_dim = env_map_size.x as f32;

    for i in 0..num_samples {
        let xi = hammersley2d(i, num_samples);
        let h = importance_sample_ggx(xi, roughness, n);
        let l = 2.0 * v.dot(h) * h - v;
        let dot_nl = n.dot(l).clamp(0.0, 1.0);

        if dot_nl > 0.0 {
            let dot_nh = n.dot(h).clamp(0.0, 1.0);
            let dot_vh = v.dot(h).clamp(0.0, 1.0);

            // Probability Distribution Function
            let pdf = d_ggx(dot_nh, roughness) * dot_nh / (4.0 * dot_vh) + 0.0001;
            // Solid angle of current sample
            let omega_s = 1.0 / ((num_samples as f32) * pdf);
            // Solid angle of 1 pixel across all cube faces
            let omega_p = 4.0 * PI / (6.0 * env_map_dim * env_map_dim);
            // Biased (+1.0) mip level for better result
            let mip_level = if roughness == 0.0 {
                0.0
            } else {
                (0.5 * (omega_s / omega_p).log2() + 1.0).max(0.0)
            };

            color += sampler_env.sample_by_lod(l, mip_level).truncate() * dot_nl;
            total_weight += dot_nl;
        }
    }

    color / total_weight
}

#[spirv(fragment)]
pub fn main_fs(
    in_pos: Vec3,
    #[spirv(push_constant)] consts: &PushConsts,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_env: &SampledImage<Cubemap>,
    out_color: &mut Vec4,
) {
    let n = in_pos.normalize();
    let result = prefilter_env_map(n, consts.roughness, consts.num_samples, sampler_env);
    *out_color = vec4(result.x, result.y, result.z, 1.0);
}
