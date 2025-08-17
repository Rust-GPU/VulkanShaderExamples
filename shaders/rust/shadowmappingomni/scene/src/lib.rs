#![no_std]

use spirv_std::glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_world_pos: &mut Vec3,
    out_light_pos: &mut Vec3,
) {
    *out_color = in_color;
    *out_normal = in_normal;

    *out_position =
        ubo.projection * ubo.view * ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_eye_pos = (ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0)).xyz();
    *out_light_vec = (ubo.light_pos.xyz() - in_pos).normalize();
    *out_world_pos = in_pos;

    *out_light_pos = ubo.light_pos.xyz();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    _in_eye_pos: Vec3,
    in_light_vec: Vec3,
    in_world_pos: Vec3,
    in_light_pos: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] shadow_cube_map: &spirv_std::Image!(cube, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &spirv_std::Sampler,
    out_frag_color: &mut Vec4,
) {
    const EPSILON: f32 = 0.15;
    const SHADOW_OPACITY: f32 = 0.5;

    // Lighting
    let i_ambient = Vec4::new(0.05, 0.05, 0.05, 1.0);
    let i_diffuse = Vec4::ONE * in_normal.dot(in_light_vec).max(0.0);

    *out_frag_color = i_ambient + i_diffuse * Vec4::new(in_color.x, in_color.y, in_color.z, 1.0);

    // Shadow
    let light_vec = in_world_pos - in_light_pos;
    let sampled_dist: Vec4 = shadow_cube_map.sample_by_lod(*sampler, light_vec, 0.0);
    let dist = light_vec.length();

    // Check if fragment is in shadow
    let shadow = if dist <= sampled_dist.x + EPSILON {
        1.0
    } else {
        SHADOW_OPACITY
    };

    out_frag_color.x *= shadow;
    out_frag_color.y *= shadow;
    out_frag_color.z *= shadow;
}
