#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
use spirv_std::{num_traits::Float, spirv};

use core::f32::consts::PI;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = vec2(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_uv = uv;
    *out_pos = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

// Random function based on http://byteblacksmith.com/improvements-to-the-canonical-one-liner-glsl-rand-for-opengl-es-2-0/
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
    // Radical inverse based on http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
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
    let phi = 2.0 * PI * xi.x + random(vec2(normal.x, normal.z)) * 0.1;
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

// Geometric Shadowing function
fn g_schlicksmith_ggx(dot_nl: f32, dot_nv: f32, roughness: f32) -> f32 {
    let k = (roughness * roughness) / 2.0;
    let gl = dot_nl / (dot_nl * (1.0 - k) + k);
    let gv = dot_nv / (dot_nv * (1.0 - k) + k);
    gl * gv
}

// BRDF integration
fn brdf(nov: f32, roughness: f32, num_samples: u32) -> Vec2 {
    // Normal always points along z-axis for the 2D lookup
    let n = vec3(0.0, 0.0, 1.0);
    let v = vec3((1.0 - nov * nov).sqrt(), 0.0, nov);

    let mut lut = Vec2::ZERO;

    for i in 0..num_samples {
        let xi = hammersley2d(i, num_samples);
        let h = importance_sample_ggx(xi, roughness, n);
        let l = 2.0 * v.dot(h) * h - v;

        let dot_nl = n.dot(l).max(0.0);
        let dot_nv = n.dot(v).max(0.0);
        let dot_vh = v.dot(h).max(0.0);
        let dot_nh = h.dot(n).max(0.0);

        if dot_nl > 0.0 {
            let g = g_schlicksmith_ggx(dot_nl, dot_nv, roughness);
            let g_vis = (g * dot_vh) / (dot_nh * dot_nv);
            let one_minus_vh = 1.0 - dot_vh;
            let fc = one_minus_vh * one_minus_vh * one_minus_vh * one_minus_vh * one_minus_vh;
            lut += vec2((1.0 - fc) * g_vis, fc * g_vis);
        }
    }

    lut / (num_samples as f32)
}

#[spirv(fragment)]
pub fn main_fs(in_uv: Vec2, out_color: &mut Vec4) {
    // Default to 1024 samples as in the GLSL version
    const NUM_SAMPLES: u32 = 1024;
    let result = brdf(in_uv.x, in_uv.y, NUM_SAMPLES);
    *out_color = vec4(result.x, result.y, 0.0, 1.0);
}
