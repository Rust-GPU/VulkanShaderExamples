#![no_std]

use spirv_std::glam::{mat4, vec2, vec4, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_space: Mat4,
    pub light_pos: Vec4,
    pub z_near: f32,
    pub z_far: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    _in_uv: Vec2,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_shadow_coord: &mut Vec4,
) {
    let bias_mat = mat4(
        vec4(0.5, 0.0, 0.0, 0.0),
        vec4(0.0, 0.5, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.5, 0.5, 0.0, 1.0),
    );

    *out_color = in_color;
    *out_normal = in_normal;

    *out_position = ubo.projection * ubo.view * ubo.model * Vec4::from((in_pos, 1.0));

    let pos = ubo.model * Vec4::from((in_pos, 1.0));
    *out_normal = Mat3::from_mat4(ubo.model) * in_normal;
    *out_light_vec = (ubo.light_pos.xyz() - in_pos).normalize();
    *out_view_vec = -pos.xyz();

    *out_shadow_coord = (bias_mat * ubo.light_space * ubo.model) * Vec4::from((in_pos, 1.0));
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    in_shadow_coord: Vec4,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_shadow: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] texture_shadow: &Image!(2D, type=f32, sampled),
    #[spirv(spec_constant(id = 0, default = 0))] enable_pcf: u32,
    out_frag_color: &mut Vec4,
) {
    const AMBIENT: f32 = 0.1;

    let shadow_coord = in_shadow_coord / in_shadow_coord.w;
    let shadow = if enable_pcf == 1 {
        filter_pcf(shadow_coord, texture_shadow, sampler_shadow)
    } else {
        texture_proj(
            shadow_coord,
            vec2(0.0, 0.0),
            texture_shadow,
            sampler_shadow,
            AMBIENT,
        )
    };

    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let _r = (-l.reflect(n)).normalize();
    let diffuse = n.dot(l).max(AMBIENT) * in_color;

    let final_color = diffuse * shadow;
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}

fn texture_proj(
    shadow_coord: Vec4,
    off: Vec2,
    texture_shadow: &Image!(2D, type=f32, sampled),
    sampler_shadow: &Sampler,
    ambient: f32,
) -> f32 {
    let mut shadow = 1.0;
    if shadow_coord.z > -1.0 && shadow_coord.z < 1.0 {
        let dist = texture_shadow
            .sample(*sampler_shadow, shadow_coord.xy() + off)
            .x;
        if shadow_coord.w > 0.0 && dist < shadow_coord.z {
            shadow = ambient;
        }
    }
    shadow
}

fn filter_pcf(
    sc: Vec4,
    texture_shadow: &Image!(2D, type=f32, sampled),
    sampler_shadow: &Sampler,
) -> f32 {
    const AMBIENT: f32 = 0.1;

    // Since we can't query texture size in Rust GPU, we'll use a fixed scale
    let scale = 1.5;
    let texel_size = 1.0 / 2048.0; // Assuming 2048x2048 shadow map
    let dx = scale * texel_size;
    let dy = scale * texel_size;

    let mut shadow_factor = 0.0;
    let mut count = 0;
    let range = 1;

    for x in -range..=range {
        for y in -range..=range {
            shadow_factor += texture_proj(
                sc,
                vec2(dx * x as f32, dy * y as f32),
                texture_shadow,
                sampler_shadow,
                AMBIENT,
            );
            count += 1;
        }
    }
    shadow_factor / count as f32
}
