#![no_std]

use spirv_std::glam::{vec3, vec4, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
    pub near_plane: f32,
    pub far_plane: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_uv: Vec2,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_color: &mut Vec3,
    out_pos: &mut Vec3,
) {
    *out_position = ubo.projection * ubo.view * ubo.model * in_pos;
    *out_uv = in_uv;

    // Vertex position in view space
    *out_pos = (ubo.view * ubo.model * in_pos).truncate();

    // Normal in view space
    let normal_matrix = Mat3::from_mat4(ubo.view * ubo.model);
    *out_normal = normal_matrix * in_normal;

    *out_color = in_color;
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    in_pos: Vec3,
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 1, binding = 0)] sampler_colormap: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0)] texture_colormap: &Image!(2D, type=f32, sampled),
    out_position: &mut Vec4,
    out_normal: &mut Vec4,
    out_albedo: &mut Vec4,
) {
    let depth = linear_depth(frag_coord.z, ubo.near_plane, ubo.far_plane);
    *out_position = vec4(in_pos.x, in_pos.y, in_pos.z, depth);
    let normalized_normal = in_normal.normalize() * 0.5 + 0.5;
    *out_normal = vec4(
        normalized_normal.x,
        normalized_normal.y,
        normalized_normal.z,
        1.0,
    );
    *out_albedo = texture_colormap.sample(*sampler_colormap, in_uv)
        * vec4(in_color.x, in_color.y, in_color.z, 1.0);
}

fn linear_depth(depth: f32, near_plane: f32, far_plane: f32) -> f32 {
    let z = depth * 2.0 - 1.0;
    (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane))
}
