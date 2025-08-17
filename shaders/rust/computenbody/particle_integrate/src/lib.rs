#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{vec4, Vec4},
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
}

#[spirv(compute(threads(256)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: spirv_std::glam::UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &mut [Particle],
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo: &UBO,
) {
    let index = global_id.x as usize;

    if index >= ubo.particle_count as usize {
        return;
    }

    let mut position = vec4(
        particles[index].pos[0],
        particles[index].pos[1],
        particles[index].pos[2],
        particles[index].pos[3],
    );
    let velocity = vec4(
        particles[index].vel[0],
        particles[index].vel[1],
        particles[index].vel[2],
        particles[index].vel[3],
    );

    // Euler integration: position += velocity * deltaTime
    position = position
        + vec4(
            velocity.x * ubo.delta_t,
            velocity.y * ubo.delta_t,
            velocity.z * ubo.delta_t,
            0.0,
        );

    particles[index].pos = [position.x, position.y, position.z, position.w];
}
