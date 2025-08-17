#![no_std]

use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = vec2(((vert_index << 1) & 2) as f32, (vert_index & 2) as f32);
    *out_uv = uv;
    *out_position = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}
