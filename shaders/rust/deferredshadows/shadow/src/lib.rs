#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat4, Vec4},
    spirv,
};

const LIGHT_COUNT: usize = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub mvp: [Mat4; LIGHT_COUNT],
    pub instance_pos: [Vec4; 3],
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    #[spirv(instance_index)] instance_index: u32,
    #[spirv(position)] out_position: &mut Vec4,
    out_instance_index: &mut u32,
) {
    *out_instance_index = instance_index;
    *out_position = in_pos;
}

#[spirv(geometry(triangles = 3, output_triangle_strip = 3, invocations = 3))]
pub fn main_gs(
    #[spirv(position)] in_position: [Vec4; 3],
    in_instance_index: [u32; 3],
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    #[spirv(layer, flat)] out_layer: &mut u32,
) {
    let instance_index = in_instance_index[0];
    let instanced_pos = ubo.instance_pos[instance_index as usize];

    for i in 0..3 {
        let tmp_pos = in_position[i] + instanced_pos;
        *out_position = ubo.mvp[invocation_id as usize] * tmp_pos;
        *out_layer = invocation_id;

        unsafe { spirv_std::arch::emit_vertex() };
    }

    unsafe { spirv_std::arch::end_primitive() };
}
