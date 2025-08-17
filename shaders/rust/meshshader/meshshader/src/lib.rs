#![no_std]

use spirv_std::arch::{emit_mesh_tasks_ext, set_mesh_outputs_ext};
use spirv_std::glam::{vec4, Mat4, UVec3, Vec3, Vec4};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
}

#[spirv(task_ext(threads(1)))]
pub fn main_task() {
    unsafe {
        emit_mesh_tasks_ext(3, 1, 1);
    }
}

#[spirv(mesh_ext(
    threads(1),
    output_vertices = 3,
    output_primitives_ext = 1,
    output_triangles_ext
))]
pub fn main_mesh(
    #[spirv(local_invocation_id)] local_invocation_id: UVec3,
    #[spirv(global_invocation_id)] global_invocation_id: UVec3,
    #[spirv(local_invocation_index)] local_invocation_index: u32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] positions: &mut [Vec4; 3],
    #[spirv(primitive_triangle_indices_ext)] indices: &mut [UVec3; 1],
    out_colors: &mut [Vec3; 3],
) {
    const POSITIONS: [Vec4; 3] = [
        vec4(0.0, -1.0, 0.0, 1.0),
        vec4(-1.0, 1.0, 0.0, 1.0),
        vec4(1.0, 1.0, 0.0, 1.0),
    ];

    const COLORS: [Vec3; 3] = [
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
    ];

    let _iid = local_invocation_id.x;
    let offset = vec4(0.0, 0.0, global_invocation_id.x as f32, 0.0);

    unsafe {
        set_mesh_outputs_ext(3, 1);
    }

    let mvp = ubo.projection * ubo.view * ubo.model;
    positions[0] = mvp * (POSITIONS[0] + offset);
    positions[1] = mvp * (POSITIONS[1] + offset);
    positions[2] = mvp * (POSITIONS[2] + offset);
    out_colors[0] = COLORS[0];
    out_colors[1] = COLORS[1];
    out_colors[2] = COLORS[2];
    indices[local_invocation_index as usize] = UVec3::new(0, 1, 2);
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, out_frag_color: &mut Vec4) {
    *out_frag_color = vec4(in_color.x, in_color.y, in_color.z, 1.0);
}
