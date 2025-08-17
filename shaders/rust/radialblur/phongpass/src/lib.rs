#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    num_traits::Float,
    spirv, Image, Sampler,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub model: Mat4,
    pub gradient_pos: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    pos: Vec4,
    _in_uv: Vec2, // Location 1 - unused but needed to match GLSL layout
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = Vec2::new(ubo.gradient_pos, 0.0);
    *out_position = ubo.projection * ubo.model * pos;

    let eye_pos = ubo.model * pos;
    *out_eye_pos = Vec3::new(eye_pos.x, eye_pos.y, eye_pos.z);

    let light_pos = Vec4::new(0.0, 0.0, -5.0, 1.0);
    *out_light_vec = (Vec3::new(light_pos.x, light_pos.y, light_pos.z)
        - Vec3::new(pos.x, pos.y, pos.z))
    .normalize();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_eye_pos: Vec3,
    in_light_vec: Vec3,
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_gradient_ramp: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image_gradient_ramp: &Image!(2D, type=f32, sampled),
    out_frag_color: &mut Vec4,
) {
    // No light calculations for glow color
    // Use max. color channel value
    // to detect bright glow emitters
    if in_color.x >= 0.9 || in_color.y >= 0.9 || in_color.z >= 0.9 {
        let gradient_color = image_gradient_ramp.sample(*sampler_gradient_ramp, in_uv);
        out_frag_color.x = gradient_color.x;
        out_frag_color.y = gradient_color.y;
        out_frag_color.z = gradient_color.z;
        // Note: GLSL version doesn't set alpha for glow emitters
    } else {
        let eye = (-in_eye_pos).normalize();
        let reflected = (-in_light_vec).reflect(in_normal).normalize();

        let ambient = Vec4::new(0.2, 0.2, 0.2, 1.0);
        let diffuse = Vec4::new(0.5, 0.5, 0.5, 0.5) * in_normal.dot(in_light_vec).max(0.0);
        let specular_strength = 0.25;
        let specular = Vec4::new(0.5, 0.5, 0.5, 1.0)
            * reflected.dot(eye).max(0.0).powf(4.0)
            * specular_strength;

        let result =
            (ambient + diffuse) * Vec4::new(in_color.x, in_color.y, in_color.z, 1.0) + specular;
        *out_frag_color = result;
    }
}
