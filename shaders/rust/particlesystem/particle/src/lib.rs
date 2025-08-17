#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat4, Vec2, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub viewport_dim: Vec2,
    pub point_size: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_color: Vec4,
    in_alpha: f32,
    in_size: f32,
    in_rotation: f32,
    in_type: i32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
    out_color: &mut Vec4,
    out_alpha: &mut f32,
    #[spirv(flat)] out_type: &mut i32,
    out_rotation: &mut f32,
) {
    *out_color = in_color;
    *out_alpha = in_alpha;
    *out_type = in_type;
    *out_rotation = in_rotation;

    *out_position = ubo.projection * ubo.modelview * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    // Base size of the point sprites
    let sprite_size = 8.0 * in_size;

    // Scale particle size depending on camera projection
    let eye_pos = ubo.modelview * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let projected_corner =
        ubo.projection * vec4(0.5 * sprite_size, 0.5 * sprite_size, eye_pos.z, eye_pos.w);
    *out_point_size = ubo.viewport_dim.x * projected_corner.x / projected_corner.w;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(point_coord)] point_coord: Vec2,
    in_color: Vec4,
    in_alpha: f32,
    #[spirv(flat)] in_type: i32,
    in_rotation: f32,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_smoke: &spirv_std::image::SampledImage<
        spirv_std::Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_fire: &spirv_std::image::SampledImage<
        spirv_std::Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let alpha = if in_alpha <= 1.0 {
        in_alpha
    } else {
        2.0 - in_alpha
    };

    // Rotate texture coordinates
    let rot_center = 0.5;
    let rot_cos = in_rotation.cos();
    let rot_sin = in_rotation.sin();
    let rot_uv = Vec2::new(
        rot_cos * (point_coord.x - rot_center)
            + rot_sin * (point_coord.y - rot_center)
            + rot_center,
        rot_cos * (point_coord.y - rot_center) - rot_sin * (point_coord.x - rot_center)
            + rot_center,
    );

    let color = if in_type == 0 {
        // Flame
        let c = sampler_fire.sample(rot_uv);
        out_frag_color.w = 0.0;
        c
    } else {
        // Smoke
        let c = sampler_smoke.sample(rot_uv);
        out_frag_color.w = c.w * alpha;
        c
    };

    *out_frag_color = vec4(
        color.x * in_color.x * alpha,
        color.y * in_color.y * alpha,
        color.z * in_color.z * alpha,
        out_frag_color.w,
    );
}
