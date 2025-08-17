#![no_std]

use spirv_std::num_traits::Float;
use spirv_std::spirv;
use spirv_std::{
    glam::{vec3, vec4, Mat3, Mat4, Vec3, Vec4},
    Image, Sampler,
};

#[repr(C)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub inv_model: Mat4,
    pub lod_bias: f32,
    pub cube_map_index: i32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_pos: &mut Vec4,
    out_world_pos: &mut Vec3,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_pos = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    *out_world_pos = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).truncate();
    *out_normal = Mat3::from_mat4(ubo.model) * in_normal;

    let light_pos = vec3(0.0, -5.0, 5.0);
    *out_light_vec = light_pos - *out_world_pos;
    *out_view_vec = -*out_world_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image: &Image!(cube, type=f32, sampled, arrayed),
    out_frag_color: &mut Vec4,
) {
    let ci = in_pos.normalize();
    let cr = ci.reflect(in_normal.normalize());

    let mut cr_transformed = (ubo.inv_model * vec4(cr.x, cr.y, cr.z, 0.0)).truncate();
    cr_transformed.y *= -1.0;
    cr_transformed.z *= -1.0;

    let coord = vec4(
        cr_transformed.x,
        cr_transformed.y,
        cr_transformed.z,
        ubo.cube_map_index as f32,
    );
    let color = image.sample_by_lod(*sampler, coord, ubo.lod_bias);

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);

    let ambient = vec3(0.5, 0.5, 0.5) * color.truncate();
    let diffuse = n.dot(l).max(0.0) * vec3(1.0, 1.0, 1.0);
    let specular = r.dot(v).max(0.0).powf(16.0) * vec3(0.5, 0.5, 0.5);

    let final_color = ambient + diffuse * color.truncate() + specular;
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
