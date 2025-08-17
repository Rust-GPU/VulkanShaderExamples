#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec2, vec4, Mat4, Vec3, Vec4},
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_pos: &mut Vec4,
) {
    let pos = ubo.projection * ubo.view * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_pos = pos;
    *out_position = pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_pos: Vec4,
    #[spirv(front_facing)] front_facing: bool,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let tmp = vec4(
        1.0 / in_pos.w,
        1.0 / in_pos.w,
        1.0 / in_pos.w,
        1.0 / in_pos.w,
    );
    let proj_coord = in_pos * tmp;

    // Scale and bias
    let proj_coord = proj_coord + vec4(1.0, 1.0, 1.0, 1.0);
    let proj_coord = proj_coord * vec4(0.5, 0.5, 0.5, 0.5);

    // Slow single pass blur
    // For demonstration purposes only
    const BLUR_SIZE: f32 = 1.0 / 512.0;

    *out_frag_color = vec4(0.0, 0.0, 0.0, 1.0);

    if front_facing {
        // Only render mirrored scene on front facing (upper) side of mirror surface
        let mut reflection = vec4(0.0, 0.0, 0.0, 0.0);
        for x in -3..=3 {
            for y in -3..=3 {
                let offset = vec2(x as f32 * BLUR_SIZE, y as f32 * BLUR_SIZE);
                let uv = vec2(proj_coord.x + offset.x, proj_coord.y + offset.y);
                reflection = reflection + sampler_color.sample(uv) / 49.0;
            }
        }
        *out_frag_color = *out_frag_color + reflection;
    }
}
