#![no_std]

use spirv_std::glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub view: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_pos: &mut Vec4,
    out_light_pos: &mut Vec3,
) {
    *out_position = ubo.projection
        * push_consts.view
        * ubo.model
        * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_pos = Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_light_pos = ubo.light_pos.xyz();
}

#[spirv(fragment)]
pub fn main_fs(in_pos: Vec4, in_light_pos: Vec3, out_frag_color: &mut f32) {
    // Store distance to light as 32 bit float value
    let light_vec = in_pos.xyz() - in_light_pos;
    *out_frag_color = light_vec.length();
}
