#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::image::SampledImage;
use spirv_std::{
    glam::{mat3, vec3, vec4, Mat4, Vec2, Vec3, Vec4},
    num_traits::Float,
    spirv, Image,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let model_mat3 = mat3(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    *out_normal = model_mat3 * in_normal;
    let l_pos = model_mat3 * ubo.light_pos.truncate();
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = -pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_uv: Vec2,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] colormap_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 2)] _discard_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(spec_constant(id = 0))] lighting_model: u32,
    #[spirv(spec_constant(id = 1))] toon_desaturation_bits: u32,
    out_frag_color: &mut Vec4,
) {
    match lighting_model {
        0 => {
            // Phong
            let ambient = in_color * vec3(0.25, 0.25, 0.25);
            let n = in_normal.normalize();
            let l = in_light_vec.normalize();
            let v = in_view_vec.normalize();
            let r = (-l).reflect(n);
            let diffuse = n.dot(l).max(0.0) * in_color;
            let specular = r.dot(v).max(0.0).powf(32.0) * vec3(0.75, 0.75, 0.75);
            *out_frag_color = vec4(
                (ambient + diffuse * 1.75 + specular).x,
                (ambient + diffuse * 1.75 + specular).y,
                (ambient + diffuse * 1.75 + specular).z,
                1.0,
            );
        }
        1 => {
            // Toon
            let n = in_normal.normalize();
            let l = in_light_vec.normalize();
            let intensity = n.dot(l);
            let color = if intensity > 0.98 {
                in_color * 1.5
            } else if intensity > 0.9 {
                in_color * 1.0
            } else if intensity > 0.5 {
                in_color * 0.6
            } else if intensity > 0.25 {
                in_color * 0.4
            } else {
                in_color * 0.2
            };
            // Desaturate a bit - convert u32 bits to f32
            let toon_desaturation = f32::from_bits(toon_desaturation_bits);
            let desaturated = vec3(0.2126, 0.7152, 0.0722).dot(color);
            let final_color = color.lerp(
                vec3(desaturated, desaturated, desaturated),
                toon_desaturation,
            );
            *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
        }
        2 => {
            // Textured
            let color = colormap_sampler.sample(in_uv);
            let texture_color = vec4(color.x, color.x, color.x, color.w); // .rrra equivalent
            let ambient = texture_color.truncate() * vec3(0.25, 0.25, 0.25) * in_color;
            let n = in_normal.normalize();
            let l = in_light_vec.normalize();
            let v = in_view_vec.normalize();
            let r = (-l).reflect(n);
            let diffuse = n.dot(l).max(0.0) * texture_color.truncate();
            let specular = r.dot(v).max(0.0).powf(32.0) * texture_color.w;
            let final_color = ambient + diffuse + vec3(specular, specular, specular);
            *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
        }
        _ => {
            // Default fallback
            *out_frag_color = vec4(1.0, 0.0, 1.0, 1.0); // Magenta for debugging
        }
    }
}
