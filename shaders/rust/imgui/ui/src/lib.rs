#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Vec2, Vec4},
    image::SampledImage,
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConstants {
    pub scale: Vec2,
    pub translate: Vec2,
}

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
    let xy = in_pos * push_constants.scale + push_constants.translate;
    *out_position = vec4(xy.x, xy.y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_color: Vec4,
    #[spirv(descriptor_set = 0, binding = 0)] font_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_color: &mut Vec4,
) {
    *out_color = in_color * font_sampler.sample(in_uv);
}
