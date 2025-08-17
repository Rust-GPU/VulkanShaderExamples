#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec3, vec4, Vec3, Vec4};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uvw: &mut Vec3,
) {
    let vertex_index = vertex_index as u32;
    *out_uvw = vec3(
        ((vertex_index << 1) & 2) as f32,
        (vertex_index & 2) as f32,
        (vertex_index & 2) as f32,
    );
    *out_position = vec4(out_uvw.x * 2.0 - 1.0, out_uvw.y * 2.0 - 1.0, 0.0, 1.0);
}

const HASHSCALE3: Vec3 = Vec3::new(443.897, 441.423, 437.195);
const STARFREQUENCY: f32 = 0.01;

// Hash function by Dave Hoskins (https://www.shadertoy.com/view/4djSRW)
fn hash33(p3: Vec3) -> f32 {
    let mut p3 = vec3(
        (p3.x * HASHSCALE3.x).fract(),
        (p3.y * HASHSCALE3.y).fract(),
        (p3.z * HASHSCALE3.z).fract(),
    );
    p3 += p3.dot(vec3(p3.y, p3.x, p3.z) + vec3(19.19, 19.19, 19.19));
    ((p3.x + p3.y) * p3.z + (p3.x + p3.z) * p3.y + (p3.y + p3.z) * p3.x).fract()
}

fn star_field(pos: Vec3) -> Vec3 {
    let mut color = Vec3::ZERO;
    let threshold = 1.0 - STARFREQUENCY;
    let rnd = hash33(pos);
    if rnd >= threshold {
        let star_col = ((rnd - threshold) / (1.0 - threshold)).powf(16.0);
        color += vec3(star_col, star_col, star_col);
    }
    color
}

#[spirv(fragment)]
pub fn main_fs(in_uvw: Vec3, out_color: &mut Vec4) {
    let color = star_field(in_uvw);
    *out_color = vec4(color.x, color.y, color.z, 1.0);
}
