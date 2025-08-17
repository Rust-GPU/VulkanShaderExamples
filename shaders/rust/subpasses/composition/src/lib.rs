#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::num_traits::Float;
use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec4,
    pub color: [f32; 3],
    pub radius: f32,
}

const AMBIENT: f32 = 0.05;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = Vec2::new(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_uv = uv;
    *out_position = Vec4::new(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(input_attachment_index = 0, descriptor_set = 0, binding = 0)] input_position: &spirv_std::Image!(subpass, type=f32, sampled=false),
    #[spirv(input_attachment_index = 1, descriptor_set = 0, binding = 1)] input_normal: &spirv_std::Image!(subpass, type=f32, sampled=false),
    #[spirv(input_attachment_index = 2, descriptor_set = 0, binding = 2)] input_albedo: &spirv_std::Image!(subpass, type=f32, sampled=false),
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] lights: &[Light],
    _in_uv: Vec2,
    out_color: &mut Vec4,
) {
    // Read G-Buffer values from previous sub pass
    let coord = spirv_std::glam::IVec2::new(0, 0); // Subpass reads at current fragment location
    let frag_pos = input_position.read_subpass(coord).truncate();
    let normal = input_normal.read_subpass(coord).truncate();
    let albedo = input_albedo.read_subpass(coord);

    // Ambient part
    let mut frag_color = albedo.truncate() * AMBIENT;

    for i in 0..64 {
        let light = &lights[i];
        let l = light.position.truncate() - frag_pos;
        let dist = l.length();

        let l = l.normalize();
        let atten = light.radius / (dist.powf(3.0) + 1.0);

        let n = normal.normalize();
        let n_dot_l = n.dot(l).max(0.0);
        let color = Vec3::new(light.color[0], light.color[1], light.color[2]);
        let diff = color * albedo.truncate() * n_dot_l * atten;

        frag_color += diff;
    }

    *out_color = Vec4::new(frag_color.x, frag_color.y, frag_color.z, 1.0);
}
