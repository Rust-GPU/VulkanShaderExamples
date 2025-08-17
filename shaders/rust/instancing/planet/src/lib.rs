#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
pub struct UBO {
    projection: Mat4,
    modelview: Mat4,
    light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo.projection * ubo.modelview * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.modelview * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_normal = ubo.modelview.transform_vector3(in_normal);
    let l_pos = ubo.modelview.transform_vector3(ubo.light_pos.truncate());
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = -pos.truncate();
}

// Reflect function
fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * n.dot(i) * n
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_uv: Vec2,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color_map: &spirv_std::image::SampledImage<
        spirv_std::image::Image2d,
    >,
    out_color: &mut Vec4,
) {
    let color =
        sampler_color_map.sample(in_uv) * vec4(in_color.x, in_color.y, in_color.z, 1.0) * 1.5;
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = reflect(-l, n);

    let diffuse = n.dot(l).max(0.0) * in_color;
    let specular = r.dot(v).max(0.0).powf(4.0) * vec3(0.5, 0.5, 0.5) * color.x;

    *out_color = vec4(
        diffuse.x * color.x + specular.x,
        diffuse.y * color.y + specular.y,
        diffuse.z * color.z + specular.z,
        1.0,
    );
}
