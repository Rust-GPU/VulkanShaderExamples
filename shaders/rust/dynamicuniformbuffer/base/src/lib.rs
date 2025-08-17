#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UboView {
    pub projection: Mat4,
    pub view: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UboInstance {
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo_view: &UboView,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo_instance: &UboInstance,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
) {
    *out_color = in_color;
    let model_view = ubo_view.view * ubo_instance.model;
    *out_position = ubo_view.projection * model_view * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, out_frag_color: &mut Vec4) {
    *out_frag_color = vec4(in_color.x, in_color.y, in_color.z, 1.0);
}
