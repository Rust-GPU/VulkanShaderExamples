#![no_std]

use spirv_std::image::SampledImage;
use spirv_std::spirv;
use spirv_std::{
    glam::{vec4, Mat4, Vec2, Vec3, Vec4},
    Image, RuntimeArray,
};

#[repr(C)]
pub struct Matrices {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_uv: Vec2,
    in_texture_index: i32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] matrices: &Matrices,
    #[spirv(position)] out_pos: &mut Vec4,
    out_uv: &mut Vec2,
    #[spirv(flat)] out_tex_index: &mut i32,
) {
    *out_uv = in_uv;
    *out_tex_index = in_texture_index;
    let pos = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_pos = matrices.projection * matrices.view * matrices.model * pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(flat)] in_tex_index: i32,
    #[spirv(descriptor_set = 0, binding = 1)] textures: &RuntimeArray<
        SampledImage<Image!(2D, type=f32, sampled)>,
    >,
    out_frag_color: &mut Vec4,
) {
    unsafe {
        let texture = textures.index(in_tex_index as usize);
        *out_frag_color = texture.sample(in_uv);
    }
}
