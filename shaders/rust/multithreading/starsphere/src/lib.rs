#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Mat4, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub mvp: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    *out_uvw = in_pos;
    *out_position = push_consts.mvp * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

const HASHSCALE3: Vec3 = vec3(443.897, 441.423, 437.195);
const STARFREQUENCY: f32 = 0.01;

// Hash function by Dave Hoskins (https://www.shadertoy.com/view/4djSRW)
fn hash33(p3_in: Vec3) -> f32 {
    let mut p3 = p3_in.fract() * HASHSCALE3;
    p3 += p3.dot(vec3(p3.y, p3.x, p3.z) + vec3(19.19, 19.19, 19.19));
    ((p3.x + p3.y) * p3.z + (p3.x + p3.z) * p3.y + (p3.y + p3.z) * p3.x).fract()
}

fn star_field(pos: Vec3) -> Vec3 {
    let mut color = vec3(0.0, 0.0, 0.0);
    let threshold = 1.0 - STARFREQUENCY;
    let rnd = hash33(pos);
    if rnd >= threshold {
        let star_col = ((rnd - threshold) / (1.0 - threshold)).powf(16.0);
        color += vec3(star_col, star_col, star_col);
    }
    color
}

#[spirv(fragment)]
pub fn main_fs(in_uvw: Vec3, out_frag_color: &mut Vec4) {
    // Fake atmosphere at the bottom
    let atmosphere = vec3(0.1, 0.15, 0.4) * (in_uvw.y + 0.25);
    let atmosphere = vec3(
        atmosphere.x.clamp(0.0, 1.0),
        atmosphere.y.clamp(0.0, 1.0),
        atmosphere.z.clamp(0.0, 1.0),
    );

    let color = star_field(in_uvw) + atmosphere;

    *out_frag_color = vec4(color.x, color.y, color.z, 1.0);
}
