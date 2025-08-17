#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec4, Vec2, Vec4},
    spirv, Image,
};

// Push constants structure for UI overlay
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConstants {
    pub scale: Vec2,
    pub translate: Vec2,
}

// UI overlay vertex shader with push constants
#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec2,
    in_uv: Vec2,
    in_color: Vec4,
    #[spirv(push_constant)] push_constants: &PushConstants,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_color: &mut Vec4,
) {
    *out_uv = in_uv;
    *out_color = in_color;
    *out_position = vec4(
        in_pos.x * push_constants.scale.x + push_constants.translate.x,
        in_pos.y * push_constants.scale.y + push_constants.translate.y,
        0.0,
        1.0,
    );
}

// UI overlay fragment shader with texture sampling
#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_color: Vec4,
    #[spirv(descriptor_set = 0, binding = 0)] font_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let tex_color = font_sampler.sample(in_uv);
    *out_frag_color = in_color * tex_color;
}
