#![no_std]

use spirv_std::glam::{ivec2, vec2, vec4, IVec2, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::image::{sample_with, ImageWithMethods};
use spirv_std::{num_traits::Float, spirv, Image};

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec4,     // 16 bytes, aligned to 16
    pub color_radius: Vec4, // 16 bytes - store color.xyz in xyz, radius in w
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct UBO {
    pub lights: [Light; 6],         // 6 * 32 = 192 bytes
    pub view_pos: Vec4,             // 16 bytes, total = 208 bytes
    pub _padding1: Vec4,            // 16 bytes padding to align next member to 224
    pub debug_display_target: Vec4, // Use Vec4 with debug_display_target in x component
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = vec2(((vert_index << 1) & 2) as f32, (vert_index & 2) as f32);
    *out_uv = uv;
    *out_position = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

const NUM_LIGHTS: usize = 6;

fn resolve(
    tex: &Image!(2D, format = rgba8, sampled, multisampled),
    uv: IVec2,
    num_samples: u32,
) -> Vec4 {
    let mut result = Vec4::ZERO;
    for i in 0..num_samples {
        let val: Vec4 = tex.fetch_with(uv, sample_with::sample_index(i as i32));
        result += val;
    }
    // Average resolved samples
    result / (num_samples as f32)
}

fn calculate_lighting(pos: Vec3, normal: Vec3, albedo: Vec4, ubo: &UBO) -> Vec3 {
    let mut result = Vec3::ZERO;

    for i in 0..NUM_LIGHTS {
        // Vector to light
        let l = ubo.lights[i].position.xyz() - pos;
        // Distance from light to fragment position
        let dist = l.length();

        // Viewer to fragment
        let v = (ubo.view_pos.xyz() - pos).normalize();

        // Light to fragment
        let l = l.normalize();

        // Attenuation
        let atten = ubo.lights[i].color_radius.w / (dist.powf(2.0) + 1.0);

        // Diffuse part
        let n = normal.normalize();
        let n_dot_l = n.dot(l).max(0.0);
        let diff = ubo.lights[i].color_radius.xyz() * albedo.xyz() * n_dot_l * atten;

        // Specular part
        let r = (-l).reflect(n);
        let n_dot_r = r.dot(v).max(0.0);
        let spec = ubo.lights[i].color_radius.xyz() * albedo.w * n_dot_r.powf(8.0) * atten;

        result += diff + spec;
    }
    result
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_position: &Image!(
        2D,
        format = rgba16f,
        sampled,
        multisampled
    ),
    #[spirv(descriptor_set = 0, binding = 2)] sampler_normal: &Image!(
        2D,
        format = rgba16f,
        sampled,
        multisampled
    ),
    #[spirv(descriptor_set = 0, binding = 3)] sampler_albedo: &Image!(
        2D,
        format = rgba8,
        sampled,
        multisampled
    ),
    #[spirv(uniform, descriptor_set = 0, binding = 4)] ubo: &UBO,
    #[spirv(spec_constant(id = 0, default = 8))] num_samples: u32,
    out_frag_color: &mut Vec4,
) {
    let att_dim: UVec2 = sampler_position.query_size();
    let uv = ivec2(
        (in_uv.x * att_dim.x as f32) as i32,
        (in_uv.y * att_dim.y as f32) as i32,
    );

    // Debug display
    if ubo.debug_display_target.x as i32 > 0 {
        let val: Vec4 = match ubo.debug_display_target.x as i32 {
            1 => sampler_position.fetch_with(uv, sample_with::sample_index(0)),
            2 => sampler_normal.fetch_with(uv, sample_with::sample_index(0)),
            3 => sampler_albedo.fetch_with(uv, sample_with::sample_index(0)),
            4 => {
                let alb: Vec4 = sampler_albedo.fetch_with(uv, sample_with::sample_index(0));
                vec4(alb.w, alb.w, alb.w, 1.0)
            }
            _ => Vec4::ZERO,
        };
        *out_frag_color = vec4(val.x, val.y, val.z, 1.0);
        return;
    }

    const AMBIENT: f32 = 0.15;

    // Ambient part
    let alb = resolve(sampler_albedo, uv, num_samples);
    let mut frag_color = Vec3::ZERO;

    // Calculate lighting for every MSAA sample
    for i in 0..num_samples {
        let pos: Vec4 = sampler_position.fetch_with(uv, sample_with::sample_index(i as i32));
        let normal: Vec4 = sampler_normal.fetch_with(uv, sample_with::sample_index(i as i32));
        let albedo: Vec4 = sampler_albedo.fetch_with(uv, sample_with::sample_index(i as i32));
        frag_color += calculate_lighting(pos.xyz(), normal.xyz(), albedo, ubo);
    }

    frag_color = (alb.xyz() * AMBIENT) + frag_color / (num_samples as f32);

    *out_frag_color = vec4(frag_color.x, frag_color.y, frag_color.z, 1.0);
}
