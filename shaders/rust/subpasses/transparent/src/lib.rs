#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    spirv, Image, Sampler,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_color: Vec3,
    _in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_color = in_color;
    *out_uv = in_uv;

    *out_position =
        ubo.projection * ubo.view * ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(input_attachment_index = 0, descriptor_set = 0, binding = 1)] sampler_position_depth: &spirv_std::Image!(subpass, type=f32, sampled=false),
    #[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] texture: &Image!(2D, type=f32, sampled),
    _in_color: Vec3,
    in_uv: Vec2,
    #[spirv(spec_constant(id = 0))] near_plane_bits: u32,
    #[spirv(spec_constant(id = 1))] far_plane_bits: u32,
    out_color: &mut Vec4,
) {
    // Sample depth from deferred depth buffer and discard if obscured
    let coord = spirv_std::glam::IVec2::new(0, 0); // Subpass reads at current fragment location
    let depth = sampler_position_depth.read_subpass(coord).w;

    // Save the sampled texture color before discarding.
    // This is to avoid implicit derivatives in non-uniform control flow.
    let sampled_color: Vec4 = texture.sample(*sampler, in_uv);

    // Linearize depth
    let near_plane = f32::from_bits(near_plane_bits);
    let far_plane = f32::from_bits(far_plane_bits);
    let z = frag_coord.z * 2.0 - 1.0;
    let linear_depth =
        (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane));

    if depth != 0.0 && linear_depth > depth {
        spirv_std::arch::kill();
    }

    *out_color = sampled_color;
}
