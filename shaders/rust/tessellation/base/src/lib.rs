#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv, Image, Sampler,
};

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_position = Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_normal = in_normal;
    *out_uv = in_uv;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(descriptor_set = 1, binding = 0)] color_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0)] color_texture: &Image!(2D, type=f32, sampled),
    in_normal: Vec3,
    in_uv: Vec2,
    out_frag_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = Vec3::new(-4.0, -4.0, 0.0).normalize();

    let color: Vec4 = color_texture.sample(*color_sampler, in_uv);

    let lighting = n.dot(l).max(0.0).clamp(0.2, 1.0);
    *out_frag_color = Vec4::new(
        lighting * color.x,
        lighting * color.y,
        lighting * color.z,
        color.w,
    );
}
