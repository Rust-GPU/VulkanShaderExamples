#![no_std]

use spirv_std::glam::{vec4, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::image::SampledImage;
use spirv_std::spirv;
use spirv_std::Image;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
    pub instance_pos: [Vec4; 3],
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    in_uv: Vec2,
    in_color: Vec3,
    in_normal: Vec3,
    in_tangent: Vec3,
    #[spirv(instance_index)] instance_index: i32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_color: &mut Vec3,
    out_world_pos: &mut Vec3,
    out_tangent: &mut Vec3,
) {
    let tmp_pos = in_pos + ubo.instance_pos[instance_index as usize];

    *out_position = ubo.projection * ubo.view * ubo.model * tmp_pos;

    *out_uv = in_uv;

    // Vertex position in world space
    *out_world_pos = (ubo.model * tmp_pos).xyz();

    // Normal - just normalize, don't transform
    *out_normal = in_normal.normalize();
    *out_tangent = in_tangent.normalize();

    // Currently just vertex color
    *out_color = in_color;
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_uv: Vec2,
    _in_color: Vec3,
    in_world_pos: Vec3,
    in_tangent: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_normal: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    out_position: &mut Vec4,
    out_normal: &mut Vec4,
    out_albedo: &mut Vec4,
) {
    *out_position = vec4(in_world_pos.x, in_world_pos.y, in_world_pos.z, 1.0);

    // Calculate normal in tangent space
    let n = in_normal.normalize();
    let t = in_tangent.normalize();
    let b = n.cross(t);
    let tbn = Mat3::from_cols(t, b, n);
    let sampled_normal = sampler_normal.sample(in_uv).xyz() * 2.0 - 1.0;
    let tnorm = tbn * sampled_normal.normalize();
    *out_normal = vec4(tnorm.x, tnorm.y, tnorm.z, 1.0);

    *out_albedo = sampler_color.sample(in_uv);
}
