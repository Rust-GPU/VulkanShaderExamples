#![no_std]

use spirv_std::glam::{vec2, vec4, Mat3, Mat4, Vec2, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

const SSAO_KERNEL_SIZE: usize = 64;
const SSAO_RADIUS: f32 = 0.5;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBOSSAOKernel {
    pub samples: [Vec4; SSAO_KERNEL_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub noise_scale: Vec2,
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_position_depth: &Sampler,
    #[spirv(descriptor_set = 0, binding = 0)] texture_position_depth: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler_normal: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] texture_normal: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] sampler_ssao_noise: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] texture_ssao_noise: &Image!(2D, type=f32, sampled),
    #[spirv(uniform, descriptor_set = 0, binding = 3)] ubo_ssao_kernel: &UBOSSAOKernel,
    #[spirv(uniform, descriptor_set = 0, binding = 4)] ubo: &UBO,
    out_frag_color: &mut f32,
) {
    // Get G-Buffer values
    let frag_pos = texture_position_depth
        .sample(*sampler_position_depth, in_uv)
        .xyz();
    let normal = (texture_normal.sample(*sampler_normal, in_uv).xyz() * 2.0 - 1.0).normalize();

    // Get a random vector using a noise lookup
    let noise_uv = ubo.noise_scale * in_uv;
    let random_vec = texture_ssao_noise
        .sample(*sampler_ssao_noise, noise_uv)
        .xyz()
        * 2.0
        - 1.0;

    // Create TBN matrix
    let tangent = (random_vec - normal * random_vec.dot(normal)).normalize();
    let bitangent = tangent.cross(normal);
    let tbn = Mat3::from_cols(tangent, bitangent, normal);

    // Calculate occlusion value
    let mut occlusion = 0.0f32;
    let bias = 0.025f32;

    for i in 0..SSAO_KERNEL_SIZE {
        let sample_vec = ubo_ssao_kernel.samples[i].xyz();
        let sample_pos = frag_pos + tbn * sample_vec * SSAO_RADIUS;

        // Project
        let mut offset = vec4(sample_pos.x, sample_pos.y, sample_pos.z, 1.0);
        offset = ubo.projection * offset;
        let offset_xyz = offset.xyz() / offset.w;
        let offset_xy = vec2(offset_xyz.x * 0.5 + 0.5, offset_xyz.y * 0.5 + 0.5);

        let sample_depth = -texture_position_depth
            .sample(*sampler_position_depth, offset_xy)
            .w;

        let range_check = smoothstep(0.0, 1.0, SSAO_RADIUS / (frag_pos.z - sample_depth).abs());
        occlusion += if sample_depth >= sample_pos.z + bias {
            1.0
        } else {
            0.0
        } * range_check;
    }

    occlusion = 1.0 - (occlusion / SSAO_KERNEL_SIZE as f32);
    *out_frag_color = occlusion;
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
