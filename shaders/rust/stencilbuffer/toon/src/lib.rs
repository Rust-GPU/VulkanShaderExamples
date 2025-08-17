#![no_std]

use spirv_std::glam::{Mat3, Mat4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    _in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_color = Vec3::new(1.0, 0.0, 0.0);
    *out_position = ubo.projection * ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let model_mat3 = Mat3::from_mat4(ubo.model);
    *out_normal = model_mat3 * in_normal;

    let pos = ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let l_pos = model_mat3 * ubo.light_pos.xyz();
    *out_light_vec = l_pos - pos.xyz();
}

#[spirv(fragment)]
pub fn main_fs(in_normal: Vec3, in_color: Vec3, in_light_vec: Vec3, out_frag_color: &mut Vec4) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let intensity = n.dot(l);

    let mut color = if intensity > 0.98 {
        in_color * 1.5
    } else if intensity > 0.9 {
        in_color * 1.0
    } else if intensity > 0.5 {
        in_color * 0.6
    } else if intensity > 0.25 {
        in_color * 0.4
    } else {
        in_color * 0.2
    };

    // Desaturate a bit
    let grayscale = Vec3::new(0.2126, 0.7152, 0.0722).dot(color);
    color = color.lerp(Vec3::splat(grayscale), 0.1);

    *out_frag_color = Vec4::new(color.x, color.y, color.z, 1.0);
}
