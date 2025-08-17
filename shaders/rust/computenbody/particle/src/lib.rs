#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec2, vec4, Mat4, Vec2, Vec4, Vec4Swizzles},
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub screendim: Vec2,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4, // particle position + mass
    in_vel: Vec4, // particle velocity + gradient pos
    #[spirv(uniform, descriptor_set = 0, binding = 2)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
    out_gradient_pos: &mut f32,
) {
    // Point size influenced by mass (stored in inPos.w)
    let sprite_size = 0.005 * in_pos.w;

    // Transform to eye space
    let eye_pos = ubo.modelview * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    // Calculate point size in screen space
    let projected_corner =
        ubo.projection * vec4(0.5 * sprite_size, 0.5 * sprite_size, eye_pos.z, eye_pos.w);
    *out_point_size = (ubo.screendim.x * projected_corner.x / projected_corner.w).clamp(1.0, 128.0);

    // Final position
    *out_position = ubo.projection * eye_pos;

    // Pass gradient position for coloring
    *out_gradient_pos = in_vel.w;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(descriptor_set = 0, binding = 0)] sampler_color_map: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_gradient_ramp: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(point_coord)] point_coord: Vec2,
    in_gradient_pos: f32,
    out_frag_color: &mut Vec4,
) {
    // Sample gradient color based on particle's gradient position
    let color = sampler_gradient_ramp
        .sample(vec2(in_gradient_pos, 0.0))
        .xyz();

    // Sample particle texture and modulate with gradient color
    let particle_color = sampler_color_map.sample(point_coord).xyz();

    *out_frag_color = vec4(
        particle_color.x * color.x,
        particle_color.y * color.y,
        particle_color.z * color.z,
        1.0,
    );
}
