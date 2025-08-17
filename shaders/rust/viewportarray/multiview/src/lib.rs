#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat3, Mat4, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: [Mat4; 2],
    pub modelview: [Mat4; 2],
    pub light_pos: Vec4,
}

#[spirv(geometry(triangles = 3, output_triangle_strip = 3, invocations = 2))]
pub fn main_gs(
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_color: [Vec3; 3],
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(primitive_id)] primitive_id_in: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    #[spirv(viewport_index, flat)] out_viewport_index: &mut u32,
    #[spirv(primitive_id, flat)] out_primitive_id: &mut u32,
) {
    let inv_id = invocation_id as usize;

    for i in 0..3 {
        *out_normal = Mat3::from_mat4(ubo.modelview[inv_id]) * in_normal[i];
        *out_color = in_color[i];

        let pos = in_position[i];
        let world_pos = ubo.modelview[inv_id] * pos;

        let l_pos = (ubo.modelview[inv_id] * ubo.light_pos).truncate();
        *out_light_vec = l_pos - world_pos.truncate();
        *out_view_vec = -world_pos.truncate();

        *out_position = ubo.projection[inv_id] * world_pos;

        // Set the viewport index that the vertex will be emitted to
        *out_viewport_index = invocation_id;
        *out_primitive_id = primitive_id_in;

        unsafe { spirv_std::arch::emit_vertex() };
    }

    unsafe { spirv_std::arch::end_primitive() };
}
