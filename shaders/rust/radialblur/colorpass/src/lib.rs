#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    spirv, Image, Sampler,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub gradient_pos: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    _in_uv: Vec2, // Location 1 - unused but needed to match GLSL layout
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_color = in_color;
    *out_uv = Vec2::new(ubo.gradient_pos, 0.0);
    *out_position = ubo.projection * ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_color: Vec3,
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_gradient_ramp: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image_gradient_ramp: &Image!(2D, type=f32, sampled),
    out_frag_color: &mut Vec4,
) {
    // Use max. color channel value to detect bright glow emitters
    if in_color.x >= 0.9 || in_color.y >= 0.9 || in_color.z >= 0.9 {
        let gradient_color = image_gradient_ramp.sample(*sampler_gradient_ramp, in_uv);
        out_frag_color.x = gradient_color.x;
        out_frag_color.y = gradient_color.y;
        out_frag_color.z = gradient_color.z;
        // Note: GLSL version doesn't set alpha, leaving it undefined
    } else {
        out_frag_color.x = in_color.x;
        out_frag_color.y = in_color.y;
        out_frag_color.z = in_color.z;
        // Note: GLSL version doesn't set alpha, leaving it undefined
    }
}
