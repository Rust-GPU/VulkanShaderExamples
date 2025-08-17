#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec2, UVec2, Vec2, Vec4},
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub blur_scale: f32,
    pub blur_strength: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    // Generate fullscreen triangle using vertex index
    *out_uv = vec2(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_position = Vec4::new(out_uv.x * 2.0 - 1.0, out_uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(spec_constant(id = 0, default = 0))] blur_direction: u32,
    in_uv: Vec2,
    out_frag_color: &mut Vec4,
) {
    // Gaussian blur weights
    let weight = [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216];

    // Get texture size for offset calculation (matches original GLSL)
    let tex_size: UVec2 = sampler_color.query_size_lod(0);
    let tex_offset = vec2(1.0 / tex_size.x as f32, 1.0 / tex_size.y as f32) * ubo.blur_scale;

    // Sample current fragment
    let mut result = sampler_color.sample(in_uv).truncate() * weight[0];

    // Sample surrounding pixels for blur
    for i in 1..5i32 {
        let offset = i as f32;

        if blur_direction == 1 {
            // Horizontal blur
            result += sampler_color
                .sample(in_uv + vec2(tex_offset.x * offset, 0.0))
                .truncate()
                * weight[i as usize]
                * ubo.blur_strength;
            result += sampler_color
                .sample(in_uv - vec2(tex_offset.x * offset, 0.0))
                .truncate()
                * weight[i as usize]
                * ubo.blur_strength;
        } else {
            // Vertical blur
            result += sampler_color
                .sample(in_uv + vec2(0.0, tex_offset.y * offset))
                .truncate()
                * weight[i as usize]
                * ubo.blur_strength;
            result += sampler_color
                .sample(in_uv - vec2(0.0, tex_offset.y * offset))
                .truncate()
                * weight[i as usize]
                * ubo.blur_strength;
        }
    }

    *out_frag_color = Vec4::new(result.x, result.y, result.z, 1.0);
}
