#![no_std]

use spirv_std::arch::atomic_i_add;
use spirv_std::glam::{Mat4, UVec3, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceData {
    pub pos: [f32; 3],
    pub scale: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: u32,
    pub first_instance: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub camera_pos: Vec4,
    pub frustum_planes: [Vec4; 6],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBOOut {
    pub draw_count: i32,
    pub lod_count: [i32; 6], // MAX_LOD_LEVEL + 1
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LOD {
    pub first_index: u32,
    pub index_count: u32,
    pub distance: f32,
    pub _pad0: f32,
}

fn frustum_check(pos: Vec4, radius: f32, frustum_planes: &[Vec4; 6]) -> bool {
    for i in 0..6 {
        if pos.dot(frustum_planes[i]) + radius < 0.0 {
            return false;
        }
    }
    true
}

#[spirv(compute(threads(16)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] instances: &[InstanceData],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    indirect_draws: &mut [IndexedIndirectCommand],
    #[spirv(uniform, descriptor_set = 0, binding = 2)] ubo: &UBO,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] ubo_out: &mut UBOOut,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] lods: &[LOD],
    #[spirv(spec_constant(id = 0, default = 5))] max_lod_level: u32,
) {
    let idx = global_id.x as usize;

    // Bounds check - important!
    if idx >= instances.len() || idx >= indirect_draws.len() {
        return;
    }

    let pos = Vec4::new(
        instances[idx].pos[0],
        instances[idx].pos[1],
        instances[idx].pos[2],
        1.0,
    );

    // Check if object is within current viewing frustum
    if frustum_check(pos, 1.0, &ubo.frustum_planes) {
        indirect_draws[idx].instance_count = 1;

        // Increase number of indirect draw counts
        unsafe {
            atomic_i_add::<
                i32,
                { spirv_std::memory::Scope::Device as u32 },
                { spirv_std::memory::Semantics::NONE.bits() },
            >(&mut ubo_out.draw_count, 1);
        }

        // Select appropriate LOD level based on distance to camera
        let mut lod_level = max_lod_level;
        let camera_pos_vec3 = ubo.camera_pos.xyz();
        let instance_pos = Vec3::new(
            instances[idx].pos[0],
            instances[idx].pos[1],
            instances[idx].pos[2],
        );
        let dist = instance_pos.distance(camera_pos_vec3);
        for i in 0..max_lod_level {
            if dist < lods[i as usize].distance {
                lod_level = i;
                break;
            }
        }
        indirect_draws[idx].first_index = lods[lod_level as usize].first_index;
        indirect_draws[idx].index_count = lods[lod_level as usize].index_count;

        // Update stats
        unsafe {
            atomic_i_add::<
                i32,
                { spirv_std::memory::Scope::Device as u32 },
                { spirv_std::memory::Semantics::NONE.bits() },
            >(&mut ubo_out.lod_count[lod_level as usize], 1);
        }
    } else {
        indirect_draws[idx].instance_count = 0;
    }
}
