#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
pub struct UBO {
    projection: Mat4,
    model: Mat4,
    view_pos: Vec4,
    depth: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec3,
    out_lod_bias: &mut f32,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_uv = vec3(in_uv.x, in_uv.y, ubo.depth);

    let world_pos = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).truncate();

    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    // Transform normal by inverse transpose of model matrix
    let normal_matrix = ubo.model.inverse().transpose();
    *out_normal = normal_matrix.transform_vector3(in_normal);

    let light_pos = Vec3::ZERO;
    let l_pos = ubo.model.transform_vector3(light_pos);
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = ubo.view_pos.truncate() - pos.truncate();

    // Note: out_lod_bias is not used in the GLSL shader, leaving uninitialized
    *out_lod_bias = 0.0;
}

// Reflect function
fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * n.dot(i) * n
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec3,
    _in_lod_bias: f32,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &spirv_std::image::SampledImage<
        spirv_std::image::Image3d,
    >,
    out_color: &mut Vec4,
) {
    let color = sampler_color.sample(in_uv);

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = reflect(-l, n);

    let diffuse = n.dot(l).max(0.0) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * color.x;

    *out_color = vec4(
        diffuse.x * color.x + specular,
        diffuse.y * color.x + specular,
        diffuse.z * color.x + specular,
        1.0,
    );
}
