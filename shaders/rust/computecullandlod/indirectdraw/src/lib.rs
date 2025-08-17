#![no_std]

use spirv_std::glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec4,
    in_normal: Vec3,
    in_color: Vec3,
    instance_pos: Vec3,
    instance_scale: f32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_color = in_color;
    *out_normal = in_normal;

    let pos = Vec4::new(
        (in_pos.x * instance_scale) + instance_pos.x,
        (in_pos.y * instance_scale) + instance_pos.y,
        (in_pos.z * instance_scale) + instance_pos.z,
        1.0,
    );

    *out_position = ubo.projection * ubo.modelview * pos;

    let l_pos = Vec4::new(0.0, 10.0, 50.0, 1.0);
    *out_light_vec = l_pos.xyz() - pos.xyz();
    *out_view_vec = -pos.xyz();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    _in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let ambient = Vec3::splat(0.25);
    let diffuse = Vec3::splat(n.dot(l).max(0.0));
    *out_frag_color = Vec4::new(
        (ambient.x + diffuse.x) * in_color.x,
        (ambient.y + diffuse.y) * in_color.y,
        (ambient.z + diffuse.z) * in_color.z,
        1.0,
    );
}
