#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat3, Mat4, Vec3, Vec4},
    spirv,
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
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_world_pos: &mut Vec3,
    _out_tangent: &mut Vec3,
) {
    *out_position = ubo.projection * ubo.view * ubo.model * in_pos;

    // Vertex position in world space
    let world_pos = (ubo.model * in_pos).truncate();
    // GL to Vulkan coord space
    *out_world_pos = Vec3::new(world_pos.x, -world_pos.y, world_pos.z);

    // Normal in world space
    let m_normal = Mat3::from_mat4(ubo.model).transpose().inverse();
    *out_normal = m_normal * in_normal.normalize();

    // Currently just vertex color
    *out_color = in_color;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    in_normal: Vec3,
    in_color: Vec3,
    in_world_pos: Vec3,
    #[spirv(spec_constant(id = 0))] near_plane_bits: u32,
    #[spirv(spec_constant(id = 1))] far_plane_bits: u32,
    out_color: &mut Vec4,
    out_position: &mut Vec4,
    out_normal: &mut Vec4,
    out_albedo: &mut Vec4,
) {
    *out_position = Vec4::new(in_world_pos.x, in_world_pos.y, in_world_pos.z, 1.0);

    let mut n = in_normal.normalize();
    n.y = -n.y;
    *out_normal = Vec4::new(n.x, n.y, n.z, 1.0);

    out_albedo.x = in_color.x;
    out_albedo.y = in_color.y;
    out_albedo.z = in_color.z;

    // Store linearized depth in alpha component
    let near_plane = f32::from_bits(near_plane_bits);
    let far_plane = f32::from_bits(far_plane_bits);
    let z = frag_coord.z * 2.0 - 1.0;
    out_position.w =
        (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane));

    // Write color attachments to avoid undefined behaviour (validation error)
    *out_color = Vec4::ZERO;
}
