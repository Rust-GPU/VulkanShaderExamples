#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec4, Vec2, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub brightness_contrast: Vec2,
    pub range: Vec2,
    pub attachment_index: i32,
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] out_position: &mut Vec4) {
    let x = ((vert_idx << 1) & 2) as f32;
    let y = (vert_idx & 2) as f32;
    *out_position = vec4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

fn brightness_contrast(color: Vec3, brightness: f32, contrast: f32) -> Vec3 {
    (color - 0.5) * contrast + 0.5 + brightness
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(input_attachment_index = 0, descriptor_set = 0, binding = 0)] input_color: &spirv_std::Image!(subpass, type=f32, sampled=false),
    #[spirv(input_attachment_index = 1, descriptor_set = 0, binding = 1)] input_depth: &spirv_std::Image!(subpass, type=f32, sampled=false),
    #[spirv(uniform, descriptor_set = 0, binding = 2)] ubo: &Ubo,
    out_color: &mut Vec4,
) {
    let coord = spirv_std::glam::IVec2::new(frag_coord.x as i32, frag_coord.y as i32);

    // Apply brightness and contrast filter to color input
    if ubo.attachment_index == 0 {
        // Read color from previous color input attachment
        let color = input_color.read_subpass(coord).truncate();
        let adjusted =
            brightness_contrast(color, ubo.brightness_contrast.x, ubo.brightness_contrast.y);
        *out_color = vec4(adjusted.x, adjusted.y, adjusted.z, 1.0);
    }

    // Visualize depth input range
    if ubo.attachment_index == 1 {
        // Read depth from previous depth input attachment
        let depth = input_depth.read_subpass(coord).x;
        let normalized = (depth - ubo.range.x) * (1.0 / (ubo.range.y - ubo.range.x));
        *out_color = vec4(normalized, normalized, normalized, 1.0);
    }
}
