#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
pub struct UBO {
    projection: Mat4,
    modelview: Mat4,
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
    let s = instance_rot.x.sin();
    let c = instance_rot.x.cos();

    let mx = Mat4::from_cols(
        vec4(c, s, 0.0, 0.0),
        vec4(-s, c, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),
    );

    // rotate around y
    let s = instance_rot.y.sin();
    let c = instance_rot.y.cos();

    let my = Mat4::from_cols(
        vec4(c, 0.0, s, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(-s, 0.0, c, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),
    );

    // rotate around z
    let s = instance_rot.z.sin();
    let c = instance_rot.z.cos();

    let mz = Mat4::from_cols(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, c, s, 0.0),
        vec4(0.0, -s, c, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0),
    );

    let rot_mat = mz * my * mx;

    *out_normal = (rot_mat * vec4(in_normal.x, in_normal.y, in_normal.z, 0.0)).truncate();

    let scaled_pos = in_pos * instance_scale + instance_pos;
    let pos = rot_mat * vec4(scaled_pos.x, scaled_pos.y, scaled_pos.z, 1.0);

    *out_position = ubo.projection * ubo.modelview * pos;

    let l_pos = vec4(0.0, -5.0, 0.0, 1.0);
    *out_light_vec = l_pos.truncate() - pos.truncate();
    *out_view_vec = -pos.truncate();
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
    let color = sampler_array.sample(vec3(in_uv.x, in_uv.y, in_uv.z));

    if color.w < 0.5 {
        spirv_std::arch::kill();
    }

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let ambient = vec3(0.65, 0.65, 0.65);
    let diffuse = n.dot(l).max(0.0) * in_color;

    let final_color = (ambient + diffuse) * color.truncate();
    *out_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
