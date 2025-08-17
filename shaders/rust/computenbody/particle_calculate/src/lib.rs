#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    arch::workgroup_memory_barrier_with_group_sync,
    glam::{vec3, vec4, Vec4, Vec4Swizzles},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Particle {
    pub pos: [f32; 4], // xyz = position, w = mass
    pub vel: [f32; 4], // xyz = velocity, w = gradient texture position
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub delta_t: f32,
    pub particle_count: u32,
    pub gravity: f32,
    pub power: f32,
    pub soften: f32,
}

const SHARED_DATA_SIZE: usize = 512;

#[spirv(compute(threads(256)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: spirv_std::glam::UVec3,
    #[spirv(local_invocation_id)] local_id: spirv_std::glam::UVec3,
    #[spirv(workgroup)] shared_data: &mut [Vec4; SHARED_DATA_SIZE],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &mut [Particle],
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo: &UBO,
) {
    let index = global_id.x as usize;
    let local_index = local_id.x as usize;

    if index >= ubo.particle_count as usize {
        return;
    }

    let position = vec4(
        particles[index].pos[0],
        particles[index].pos[1],
        particles[index].pos[2],
        particles[index].pos[3],
    );
    let mut velocity = vec4(
        particles[index].vel[0],
        particles[index].vel[1],
        particles[index].vel[2],
        particles[index].vel[3],
    );
    let mut acceleration = vec3(0.0, 0.0, 0.0);

    // Process particles in chunks of SHARED_DATA_SIZE
    let mut i = 0u32;
    while i < ubo.particle_count {
        // Load particle data into shared memory
        if i + (local_index as u32) < ubo.particle_count {
            let particle_idx = i as usize + local_index;
            shared_data[local_index] = vec4(
                particles[particle_idx].pos[0],
                particles[particle_idx].pos[1],
                particles[particle_idx].pos[2],
                particles[particle_idx].pos[3],
            );
        } else {
            shared_data[local_index] = vec4(0.0, 0.0, 0.0, 0.0);
        }

        // Ensure all threads have loaded their data
        unsafe {
            workgroup_memory_barrier_with_group_sync();
        }

        // Calculate forces from particles in shared memory
        for j in 0..256 {
            // gl_WorkGroupSize.x = 256
            let other = shared_data[j];
            let len = other.xyz() - position.xyz();
            let distance_sq = len.dot(len) + ubo.soften;
            acceleration += ubo.gravity * len * other.w / distance_sq.powf(ubo.power * 0.5);
        }

        // Synchronize before next iteration
        unsafe {
            workgroup_memory_barrier_with_group_sync();
        }

        i += SHARED_DATA_SIZE as u32;
    }

    // Update velocity with acceleration
    velocity = vec4(
        velocity.x + acceleration.x * ubo.delta_t,
        velocity.y + acceleration.y * ubo.delta_t,
        velocity.z + acceleration.z * ubo.delta_t,
        velocity.w,
    );

    // Update gradient texture position for visual effects
    velocity.w += 0.1 * ubo.delta_t;
    if velocity.w > 1.0 {
        velocity.w -= 1.0;
    }

    particles[index].vel = [velocity.x, velocity.y, velocity.z, velocity.w];
}
