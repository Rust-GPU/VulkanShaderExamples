#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{UVec2, Vec2, Vec4},
    spirv, Image, Sampler,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub radial_blur_scale: f32,
    pub radial_blur_strength: f32,
    pub radial_origin: Vec2,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = Vec2::new(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_uv = uv;
    *out_position = Vec4::new(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image_color: &Image!(2D, type=f32, sampled),
    out_frag_color: &mut Vec4,
) {
    let tex_dim: UVec2 = image_color.query_size_lod(0);
    let radial_size = Vec2::new(1.0 / tex_dim.x as f32, 1.0 / tex_dim.y as f32);

    let mut uv = in_uv;
    let mut color = Vec4::ZERO;
    uv += radial_size * 0.5 - ubo.radial_origin;

    const SAMPLES: i32 = 32;

    for i in 0..SAMPLES {
        let scale = 1.0 - ubo.radial_blur_scale * (i as f32 / (SAMPLES - 1) as f32);
        color += image_color.sample(*sampler_color, uv * scale + ubo.radial_origin);
    }

    *out_frag_color = (color / SAMPLES as f32) * ubo.radial_blur_strength;
}
