#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, IVec2, UVec3, Vec3Swizzles, Vec4},
    spirv, Image,
};

fn conv(kernel: &[f32; 9], data: &[f32; 9], denom: f32, offset: f32) -> f32 {
    let mut res = 0.0;
    for i in 0..9 {
        res += kernel[i] * data[i];
    }
    (res / denom + offset).clamp(0.0, 1.0)
}

#[spirv(compute(threads(16, 16)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] input_image: &Image!(
        2D,
        format = rgba8,
        sampled = false
    ),
    #[spirv(descriptor_set = 0, binding = 1)] result_image: &Image!(
        2D,
        format = rgba8,
        sampled = false
    ),
) {
    // Fetch neighbouring texels
    let mut avg = [0.0; 9];
    let mut n = 0;

    for i in -1..=1 {
        for j in -1..=1 {
            let coord = IVec2::new((global_id.x as i32) + i, (global_id.y as i32) + j);
            let rgb: Vec4 = input_image.read(coord);
            avg[n] = (rgb.x + rgb.y + rgb.z) / 3.0;
            n += 1;
        }
    }

    // Edge detection kernel
    let kernel = [
        -1.0 / 8.0,
        -1.0 / 8.0,
        -1.0 / 8.0,
        -1.0 / 8.0,
        1.0,
        -1.0 / 8.0,
        -1.0 / 8.0,
        -1.0 / 8.0,
        -1.0 / 8.0,
    ];

    let gray = conv(&kernel, &avg, 0.1, 0.0);
    let res = vec4(gray, gray, gray, 1.0);

    unsafe {
        result_image.write(IVec2::new(global_id.x as i32, global_id.y as i32), res);
    }
}
