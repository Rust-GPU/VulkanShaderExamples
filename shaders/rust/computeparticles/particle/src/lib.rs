#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{vec2, vec4, UVec3, Vec2, Vec4},
    num_traits::Float,
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub gradient_pos: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub delta_t: f32,
    pub dest_x: f32,
    pub dest_y: f32,
    pub particle_count: i32,
}

fn attraction(pos: Vec2, attract_pos: Vec2) -> Vec2 {
    let delta = attract_pos - pos;
    const DAMP: f32 = 0.5;
    let d_damped_dot = delta.dot(delta) + DAMP;
    let inv_dist = 1.0 / d_damped_dot.sqrt();
    let inv_dist_cubed = inv_dist * inv_dist * inv_dist;
    delta * inv_dist_cubed * 0.0035
}

fn repulsion(pos: Vec2, attract_pos: Vec2) -> Vec2 {
    let delta = attract_pos - pos;
    let target_distance = delta.dot(delta).sqrt();
    delta * (1.0 / (target_distance * target_distance * target_distance)) * -0.000035
}

#[spirv(compute(threads(256, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles_in: &[Particle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particles_out: &mut [Particle],
    #[spirv(uniform, descriptor_set = 0, binding = 2)] ubo: &Ubo,
) {
    let index = global_id.x;
    if index >= ubo.particle_count as u32 {
        return;
    }

    let idx = index as usize;
    let mut vel = particles_in[idx].vel;
    let mut pos = particles_in[idx].pos;
    let g_pos = particles_in[idx].gradient_pos;

    let dest_pos = vec2(ubo.dest_x, ubo.dest_y);

    let delta = dest_pos - pos;
    let target_distance = delta.dot(delta).sqrt();
    vel += repulsion(pos, dest_pos) * 0.05;

    pos += vel * ubo.delta_t;

    if pos.x < -1.0 || pos.x > 1.0 || pos.y < -1.0 || pos.y > 1.0 {
        vel = (-vel * 0.1) + attraction(pos, dest_pos) * 12.0;
    } else {
        particles_out[idx].pos = pos;
    }

    particles_out[idx].vel = vel;
    particles_out[idx].gradient_pos.x = g_pos.x + 0.02 * ubo.delta_t;
    if particles_out[idx].gradient_pos.x > 1.0 {
        particles_out[idx].gradient_pos.x -= 1.0;
    }
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec2,
    in_gradient_pos: Vec4,
    #[spirv(position)] out_position: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
    out_color: &mut Vec4,
    out_gradient_pos: &mut f32,
) {
    *out_point_size = 8.0;
    *out_color = vec4(0.035, 0.035, 0.035, 1.0);
    *out_gradient_pos = in_gradient_pos.x;
    *out_position = vec4(in_pos.x, in_pos.y, 1.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    _in_color: Vec4,
    in_gradient_pos: f32,
    #[spirv(point_coord)] point_coord: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_color_map: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_gradient_ramp: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_frag_color: &mut Vec4,
) {
    let color = sampler_gradient_ramp
        .sample(vec2(in_gradient_pos, 0.0))
        .truncate();
    let tex_color = sampler_color_map.sample(point_coord).truncate();
    *out_frag_color = vec4(
        tex_color.x * color.x,
        tex_color.y * color.y,
        tex_color.z * color.z,
        1.0,
    );
}
