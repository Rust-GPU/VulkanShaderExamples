#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec2, vec3, vec4, Vec2, Vec4},
    image::SampledImage,
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    _padding: [Vec4; 17], // offset = 272 bytes = 17 vec4s
    pub distortion_alpha: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = vec2(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    let pos_xy = *out_uv * 2.0 - vec2(1.0, 1.0);
    *out_position = vec4(pos_xy.x, pos_xy.y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_view: &SampledImage<
        Image!(2D, type=f32, sampled, arrayed),
    >,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(spec_constant(id = 0, default = 0))] view_layer: u32,
    out_color: &mut Vec4,
) {
    let alpha = ubo.distortion_alpha;

    let p1 = 2.0 * in_uv - vec2(1.0, 1.0);
    let p2 = p1 / (1.0 - alpha * p1.length());
    let p2 = (p2 + vec2(1.0, 1.0)) * 0.5;

    let inside = p2.x >= 0.0 && p2.x <= 1.0 && p2.y >= 0.0 && p2.y <= 1.0;
    *out_color = if inside {
        let layer = if view_layer == 0 { 0.0 } else { 1.0 };
        sampler_view.sample(vec3(p2.x, p2.y, layer))
    } else {
        vec4(0.0, 0.0, 0.0, 0.0)
    };
}
