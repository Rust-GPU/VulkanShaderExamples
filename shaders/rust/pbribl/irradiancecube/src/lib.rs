#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Mat4, Vec3, Vec4};
use spirv_std::image::{Cubemap, SampledImage};
use spirv_std::{num_traits::Float, spirv};

// Push constants with padding to match GLSL layout
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PushConsts {
    _padding: [f32; 16], // offset 0-63 (64 bytes)
    delta_phi: f32,      // offset 64
    delta_theta: f32,    // offset 68
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

#[spirv(fragment)]
pub fn main_fs(
    in_pos: Vec3,
    #[spirv(push_constant)] consts: &PushConsts,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_env: &SampledImage<Cubemap>,
    out_color: &mut Vec4,
) {
    let n = in_pos.normalize();
    let up = vec3(0.0, 1.0, 0.0);
    let mut right = up.cross(n).normalize();
    let up = n.cross(right);

    const HALF_PI: f32 = PI * 0.5;

    let mut color = Vec3::ZERO;
    let mut sample_count = 0u32;

    let mut phi = 0.0;
    while phi < TAU {
        let mut theta = 0.0;
        while theta < HALF_PI {
            let temp_vec = phi.cos() * right + phi.sin() * up;
            let sample_vector = theta.cos() * n + theta.sin() * temp_vec;
            color += sampler_env.sample(sample_vector).truncate() * theta.cos() * theta.sin();
            sample_count += 1;

            theta += consts.delta_theta;
        }
        phi += consts.delta_phi;
    }

    *out_color = vec4(
        color.x * PI / (sample_count as f32),
        color.y * PI / (sample_count as f32),
        color.z * PI / (sample_count as f32),
        1.0,
    );
}
