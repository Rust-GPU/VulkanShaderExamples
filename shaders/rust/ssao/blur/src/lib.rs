#![no_std]

use spirv_std::glam::{vec2, Vec2};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub texel_size: Vec2,
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] sampler_ssao: &Sampler,
    #[spirv(descriptor_set = 0, binding = 0)] texture_ssao: &Image!(2D, type=f32, sampled),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo: &UBO,
    out_frag_color: &mut f32,
) {
    const BLUR_RANGE: i32 = 2;
    let mut n = 0;
    let texel_size = ubo.texel_size;
    let mut result = 0.0;

    for x in -BLUR_RANGE..=BLUR_RANGE {
        for y in -BLUR_RANGE..=BLUR_RANGE {
            let offset = vec2(x as f32, y as f32) * texel_size;
            result += texture_ssao.sample(*sampler_ssao, in_uv + offset).x;
            n += 1;
        }
    }

    *out_frag_color = result / n as f32;
}
