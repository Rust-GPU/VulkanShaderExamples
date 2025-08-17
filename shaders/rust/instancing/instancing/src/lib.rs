#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Mat3, Mat4, Vec2, Vec3, Vec4};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
pub struct UBO {
    projection: Mat4,
    modelview: Mat4,
    light_pos: Vec4,
    loc_speed: f32,
    glob_speed: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    instance_pos: Vec3,
    instance_rot: Vec3,
    instance_scale: f32,
    instance_tex_index: i32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_color = in_color;
    *out_uv = vec3(in_uv.x, in_uv.y, instance_tex_index as f32);

    // rotate around x
    let s = (instance_rot.x + ubo.loc_speed).sin();
    let c = (instance_rot.x + ubo.loc_speed).cos();

    let mx = Mat3::from_cols(vec3(c, s, 0.0), vec3(-s, c, 0.0), vec3(0.0, 0.0, 1.0));

    // rotate around y
    let s = (instance_rot.y + ubo.loc_speed).sin();
    let c = (instance_rot.y + ubo.loc_speed).cos();

    let my = Mat3::from_cols(vec3(c, 0.0, s), vec3(0.0, 1.0, 0.0), vec3(-s, 0.0, c));

    // rotate around z
    let s = (instance_rot.z + ubo.loc_speed).sin();
    let c = (instance_rot.z + ubo.loc_speed).cos();

    let mz = Mat3::from_cols(vec3(1.0, 0.0, 0.0), vec3(0.0, c, s), vec3(0.0, -s, c));

    let rot_mat = mz * my * mx;

    // Global rotation matrix
    let s = (instance_rot.y + ubo.glob_speed).sin();
    let c = (instance_rot.y + ubo.glob_speed).cos();

    let g_rot_mat = Mat4::from_cols(
        vec4(c, 0.0, s, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(-s, 0.0, c, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),
    );

    let loc_pos = rot_mat * in_pos;
    let pos = vec4(
        loc_pos.x * instance_scale + instance_pos.x,
        loc_pos.y * instance_scale + instance_pos.y,
        loc_pos.z * instance_scale + instance_pos.z,
        1.0,
    );

    *out_position = ubo.projection * ubo.modelview * g_rot_mat * pos;
    *out_normal = ubo
        .modelview
        .transform_vector3(g_rot_mat.transform_vector3(rot_mat.inverse() * in_normal));

    let pos = ubo.modelview
        * vec4(
            in_pos.x + instance_pos.x,
            in_pos.y + instance_pos.y,
            in_pos.z + instance_pos.z,
            1.0,
        );
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
    in_uv: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_array: &spirv_std::image::SampledImage<
        spirv_std::image::Image2dArray,
    >,
    out_color: &mut Vec4,
) {
    let color = sampler_array.sample(vec3(in_uv.x, in_uv.y, in_uv.z))
        * vec4(in_color.x, in_color.y, in_color.z, 1.0);
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = reflect(-l, n);

    let diffuse = n.dot(l).max(0.1) * in_color;
    let specular = if n.dot(l) > 0.0 {
        r.dot(v).max(0.0).powf(16.0) * vec3(0.75, 0.75, 0.75) * color.x
    } else {
        Vec3::ZERO
    };

    *out_color = vec4(
        diffuse.x * color.x + specular.x,
        diffuse.y * color.y + specular.y,
        diffuse.z * color.z + specular.z,
        1.0,
    );
}
