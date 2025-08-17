#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{mat3, vec4, Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    #[spirv(flat)] out_flat_normal: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    let pos = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    *out_position = ubo.projection * ubo.model * pos;

    let world_pos = ubo.model * pos;
    let model_mat3 = mat3(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    *out_normal = model_mat3 * in_normal;
    let l_pos = ubo.light_pos.truncate();
    *out_light_vec = l_pos - world_pos.truncate();
    *out_view_vec = -world_pos.truncate();

    // Flat shading normal is not interpolated
    *out_flat_normal = *out_normal;
}
