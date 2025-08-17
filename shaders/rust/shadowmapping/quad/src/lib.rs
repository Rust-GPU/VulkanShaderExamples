#![no_std]

use spirv_std::glam::{vec2, vec4, Mat4, Vec2, Vec4};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_space: Mat4,
    pub light_pos: Vec4,
    pub z_near: f32,
    pub z_far: f32,
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

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] texture_color: &Image!(2D, type=f32, sampled),
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    out_frag_color: &mut Vec4,
) {
    let depth = texture_color.sample(*sampler_color, in_uv).x;
    let linearized_depth = linearize_depth(depth, ubo.z_near, ubo.z_far);
    let color_value = 1.0 - linearized_depth;
    *out_frag_color = vec4(color_value, color_value, color_value, 1.0);
}

fn linearize_depth(depth: f32, z_near: f32, z_far: f32) -> f32 {
    let z = depth;
    (2.0 * z_near) / (z_far + z_near - z * (z_far - z_near))
}
