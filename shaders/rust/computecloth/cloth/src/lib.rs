#![no_std]

use spirv_std::glam::{IVec2, Mat4, UVec3, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Particle {
    pub pos: Vec4,
    pub vel: Vec4,
    pub uv: Vec4,
    pub normal: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub delta_t: f32,
    pub particle_mass: f32,
    pub spring_stiffness: f32,
    pub damping: f32,
    pub rest_dist_h: f32,
    pub rest_dist_v: f32,
    pub rest_dist_d: f32,
    pub sphere_radius: f32,
    pub sphere_pos: Vec4,
    pub gravity: Vec4,
    pub particle_count: IVec2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub calculate_normals: u32,
}

fn spring_force(p0: Vec3, p1: Vec3, rest_dist: f32, spring_stiffness: f32) -> Vec3 {
    let dist = p0 - p1;
    let length = dist.length();
    if length > 0.0 {
        dist.normalize() * spring_stiffness * (length - rest_dist)
    } else {
        Vec3::ZERO
    }
}

#[spirv(compute(threads(10, 10)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particle_in: &[Particle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] particle_out: &mut [Particle],
    #[spirv(uniform, descriptor_set = 0, binding = 2)] ubo: &UBO,
    #[spirv(push_constant)] push_consts: &PushConsts,
) {
    let particle_count_x = ubo.particle_count.x as u32;
    let particle_count_y = ubo.particle_count.y as u32;
    let index = id.y * particle_count_x + id.x;
    if index >= particle_count_x * particle_count_y {
        return;
    }

    // Initial force from gravity
    let mut force = ubo.gravity.xyz() * ubo.particle_mass;

    let idx = index as usize;
    let pos = particle_in[idx].pos.xyz();
    let vel = particle_in[idx].vel.xyz();

    // Spring forces from neighboring particles
    // left
    if id.x > 0 {
        force += spring_force(
            particle_in[idx - 1].pos.xyz(),
            pos,
            ubo.rest_dist_h,
            ubo.spring_stiffness,
        );
    }
    // right
    if id.x < particle_count_x - 1 {
        force += spring_force(
            particle_in[idx + 1].pos.xyz(),
            pos,
            ubo.rest_dist_h,
            ubo.spring_stiffness,
        );
    }
    // upper
    if id.y < particle_count_y - 1 {
        force += spring_force(
            particle_in[idx + particle_count_x as usize].pos.xyz(),
            pos,
            ubo.rest_dist_v,
            ubo.spring_stiffness,
        );
    }
    // lower
    if id.y > 0 {
        force += spring_force(
            particle_in[idx - particle_count_x as usize].pos.xyz(),
            pos,
            ubo.rest_dist_v,
            ubo.spring_stiffness,
        );
    }
    // upper-left
    if id.x > 0 && id.y < particle_count_y - 1 {
        force += spring_force(
            particle_in[idx + particle_count_x as usize - 1].pos.xyz(),
            pos,
            ubo.rest_dist_d,
            ubo.spring_stiffness,
        );
    }
    // lower-left
    if id.x > 0 && id.y > 0 {
        force += spring_force(
            particle_in[idx - particle_count_x as usize - 1].pos.xyz(),
            pos,
            ubo.rest_dist_d,
            ubo.spring_stiffness,
        );
    }
    // upper-right
    if id.x < particle_count_x - 1 && id.y < particle_count_y - 1 {
        force += spring_force(
            particle_in[idx + particle_count_x as usize + 1].pos.xyz(),
            pos,
            ubo.rest_dist_d,
            ubo.spring_stiffness,
        );
    }
    // lower-right
    if id.x < particle_count_x - 1 && id.y > 0 {
        force += spring_force(
            particle_in[idx - particle_count_x as usize + 1].pos.xyz(),
            pos,
            ubo.rest_dist_d,
            ubo.spring_stiffness,
        );
    }

    // Damping
    force += -ubo.damping * vel;

    // Integrate
    let f = force * (1.0 / ubo.particle_mass);
    let new_pos = pos + vel * ubo.delta_t + 0.5 * f * ubo.delta_t * ubo.delta_t;
    let new_vel = vel + f * ubo.delta_t;

    particle_out[idx].pos = Vec4::new(new_pos.x, new_pos.y, new_pos.z, 1.0);
    particle_out[idx].vel = Vec4::new(new_vel.x, new_vel.y, new_vel.z, 0.0);

    // Sphere collision
    let sphere_dist = new_pos - ubo.sphere_pos.xyz();
    if sphere_dist.length() < ubo.sphere_radius + 0.01 {
        // If the particle is inside the sphere, push it to the outer radius
        let push_out = ubo.sphere_pos.xyz() + sphere_dist.normalize() * (ubo.sphere_radius + 0.01);
        particle_out[idx].pos = Vec4::new(push_out.x, push_out.y, push_out.z, 1.0);
        // Cancel out velocity
        particle_out[idx].vel = Vec4::ZERO;
    }

    // Calculate normals
    if push_consts.calculate_normals == 1 {
        let mut normal = Vec3::ZERO;

        let stride = particle_count_x as usize;
        if id.y > 0 {
            if id.x > 0 {
                let a = particle_in[idx - 1].pos.xyz() - pos;
                let b = particle_in[idx - stride - 1].pos.xyz() - pos;
                let c = particle_in[idx - stride].pos.xyz() - pos;
                normal += a.cross(b) + b.cross(c);
            }
            if id.x < particle_count_x - 1 {
                let a = particle_in[idx - stride].pos.xyz() - pos;
                let b = particle_in[idx - stride + 1].pos.xyz() - pos;
                let c = particle_in[idx + 1].pos.xyz() - pos;
                normal += a.cross(b) + b.cross(c);
            }
        }
        if id.y < particle_count_y - 1 {
            if id.x > 0 {
                let a = particle_in[idx + stride].pos.xyz() - pos;
                let b = particle_in[idx + stride - 1].pos.xyz() - pos;
                let c = particle_in[idx - 1].pos.xyz() - pos;
                normal += a.cross(b) + b.cross(c);
            }
            if id.x < particle_count_x - 1 {
                let a = particle_in[idx + 1].pos.xyz() - pos;
                let b = particle_in[idx + stride + 1].pos.xyz() - pos;
                let c = particle_in[idx + stride].pos.xyz() - pos;
                normal += a.cross(b) + b.cross(c);
            }
        }

        if normal.length() > 0.0 {
            normal = normal.normalize();
        }
        particle_out[idx].normal = Vec4::new(normal.x, normal.y, normal.z, 0.0);
    }

    // Copy UV coordinates
    particle_out[idx].uv = particle_in[idx].uv;
}

// Vertex shader for rendering the cloth
#[repr(C)]
#[derive(Copy, Clone)]
pub struct VertexUBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_uv: Vec2,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &VertexUBO,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_uv = in_uv;
    *out_normal = in_normal;
    let eye_pos = ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_position = ubo.projection * eye_pos;
    let pos = Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let l_pos = ubo.light_pos.xyz();
    *out_light_vec = l_pos - pos.xyz();
    *out_view_vec = -pos.xyz();
}

// Fragment shader for rendering the cloth
#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] color_texture: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] color_sampler: &Sampler,
    out_frag_color: &mut Vec4,
) {
    let color = color_texture.sample(*color_sampler, in_uv).xyz();
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = l.reflect(n);
    let diffuse = n.dot(l).max(0.15) * Vec3::ONE;
    let specular = r.dot(v).max(0.0).powf(8.0) * Vec3::splat(0.2);
    *out_frag_color = Vec4::new(
        diffuse.x * color.x + specular.x,
        diffuse.y * color.y + specular.y,
        diffuse.z * color.z + specular.z,
        1.0,
    );
}
