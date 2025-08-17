#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{mat3, vec3, vec4, Mat4, Vec2, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub normal: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    in_normal: Vec3,
    in_tangent: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_light_vec: &mut Vec3,
    out_light_vec_b: &mut Vec3,
    out_light_dir: &mut Vec3,
    out_view_vec: &mut Vec3,
) {
    let vertex_position = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).truncate();
    *out_light_dir = (ubo.light_pos.truncate() - vertex_position).normalize();

    let bi_tangent = in_normal.cross(in_tangent.truncate());

    // Setup (t)angent-(b)inormal-(n)ormal matrix for converting
    // object coordinates into tangent space
    let normal_mat3 = mat3(
        ubo.normal.x_axis.truncate(),
        ubo.normal.y_axis.truncate(),
        ubo.normal.z_axis.truncate(),
    );
    let tbn_matrix = mat3(
        normal_mat3 * in_tangent.truncate(),
        normal_mat3 * bi_tangent,
        normal_mat3 * in_normal,
    );

    *out_light_vec = tbn_matrix.transpose() * (ubo.light_pos.truncate() - vertex_position);

    let light_dist = ubo.light_pos.truncate() - in_pos;
    *out_light_vec_b = vec3(
        in_tangent.truncate().dot(light_dist),
        bi_tangent.dot(light_dist),
        in_normal.dot(light_dist),
    );

    *out_view_vec = vec3(
        in_tangent.truncate().dot(in_pos),
        bi_tangent.dot(in_pos),
        in_normal.dot(in_pos),
    );

    *out_uv = in_uv;

    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

const LIGHT_RADIUS: f32 = 45.0;

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_light_vec: Vec3,
    in_light_vec_b: Vec3,
    _in_light_dir: Vec3,
    in_view_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] s_color_map: &spirv_std::image::SampledImage<
        spirv_std::Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 2)] s_normal_height_map: &spirv_std::image::SampledImage<
        spirv_std::Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let specular_color = vec3(0.85, 0.5, 0.0);

    let inv_radius = 1.0 / LIGHT_RADIUS;
    let ambient = 0.25;

    let rgb = s_color_map.sample(in_uv).truncate();
    let normal = ((s_normal_height_map.sample(in_uv).truncate() - 0.5) * 2.0).normalize();

    let dist_sqr = in_light_vec_b.dot(in_light_vec_b);
    let l_vec = in_light_vec_b * (1.0 / dist_sqr.sqrt());

    let atten = (1.0 - inv_radius * dist_sqr.sqrt())
        .clamp(0.0, 1.0)
        .max(ambient);
    let diffuse = l_vec.dot(normal).clamp(0.0, 1.0);

    let light = (-in_light_vec).normalize();
    let view = in_view_vec.normalize();
    let reflect_dir = (-light).reflect(normal);

    let specular = view.dot(reflect_dir).max(0.0).powf(4.0);

    let final_color = (rgb * atten + (diffuse * rgb + 0.5 * specular * specular_color)) * atten;
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
