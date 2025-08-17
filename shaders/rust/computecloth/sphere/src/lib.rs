#![no_std]

use spirv_std::glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    let eye_pos = ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_position = ubo.projection * eye_pos;
    let pos = Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let l_pos = ubo.light_pos.xyz();
    *out_light_vec = l_pos - pos.xyz();
    *out_view_vec = -pos.xyz();
    *out_normal = in_normal;
}

#[spirv(fragment)]
pub fn main_fs(in_normal: Vec3, in_view_vec: Vec3, in_light_vec: Vec3, out_frag_color: &mut Vec4) {
    let color = Vec3::splat(0.5);
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = l.reflect(n);
    let diffuse = n.dot(l).max(0.15) * Vec3::ONE;
    let specular = r.dot(v).max(0.0).powf(32.0) * Vec3::ONE;
    *out_frag_color = Vec4::new(
        diffuse.x * color.x + specular.x,
        diffuse.y * color.y + specular.y,
        diffuse.z * color.z + specular.z,
        1.0,
    );
}
