#![no_std]

use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = Vec2::new(((vertex_index << 1) & 2) as f32, (vertex_index & 2) as f32);
    *out_uv = uv;
    *out_position = Vec4::new(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] shadow_cube_map: &spirv_std::Image!(cube, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &spirv_std::Sampler,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = Vec4::new(0.05, 0.05, 0.05, 1.0);

    let mut sample_pos = Vec3::ZERO;

    // Crude statement to visualize different cube map faces based on UV coordinates
    let x = (in_uv.x / 0.25).floor() as i32;
    let y = (in_uv.y / (1.0 / 3.0)).floor() as i32;

    if y == 1 {
        let mut uv = Vec2::new(in_uv.x * 4.0, (in_uv.y - 1.0 / 3.0) * 3.0);
        uv = 2.0 * Vec2::new(uv.x - (x as f32) * 1.0, uv.y) - Vec2::ONE;
        match x {
            0 => {
                // NEGATIVE_X
                sample_pos = Vec3::new(-1.0, uv.y, uv.x);
            }
            1 => {
                // POSITIVE_Z
                sample_pos = Vec3::new(uv.x, uv.y, 1.0);
            }
            2 => {
                // POSITIVE_X
                sample_pos = Vec3::new(1.0, uv.y, -uv.x);
            }
            3 => {
                // NEGATIVE_Z
                sample_pos = Vec3::new(-uv.x, uv.y, -1.0);
            }
            _ => {}
        }
    } else {
        if x == 1 {
            let uv = 2.0 * Vec2::new((in_uv.x - 0.25) * 4.0, (in_uv.y - (y as f32) / 3.0) * 3.0)
                - Vec2::ONE;
            match y {
                0 => {
                    // NEGATIVE_Y
                    sample_pos = Vec3::new(uv.x, -1.0, uv.y);
                }
                2 => {
                    // POSITIVE_Y
                    sample_pos = Vec3::new(uv.x, 1.0, -uv.y);
                }
                _ => {}
            }
        }
    }

    if sample_pos.x != 0.0 && sample_pos.y != 0.0 {
        let sampled: Vec4 = shadow_cube_map.sample_by_lod(*sampler, sample_pos, 0.0);
        let dist = sampled.xyz().length() * 0.005;
        *out_frag_color = Vec4::new(dist, dist, dist, 1.0);
    }
}
