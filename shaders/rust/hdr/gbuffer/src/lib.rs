#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Mat3, Mat4, Vec3, Vec4},
    num_traits::Float,
    spirv, Image, Sampler,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub inverse_modelview: Mat4,
    pub exposure: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(spec_constant(id = 0, default = 0))] type_id: u32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uvw: &mut Vec3,
    out_pos: &mut Vec3,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_uvw = in_pos;

    match type_id as i32 {
        0 => {
            // Skybox
            let pos = Mat3::from_mat4(ubo.modelview) * in_pos;
            *out_pos = pos;
            *out_position = ubo.projection * Vec4::new(pos.x, pos.y, pos.z, 1.0);
        }
        1 => {
            // Object
            let pos = ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
            *out_pos = Vec3::new(pos.x, pos.y, pos.z);
            *out_position =
                ubo.projection * ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
        }
        _ => {
            let pos = ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
            *out_pos = Vec3::new(pos.x, pos.y, pos.z);
            *out_position =
                ubo.projection * ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
        }
    }

    let pos = ubo.modelview * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    *out_pos = Vec3::new(pos.x, pos.y, pos.z);
    *out_normal = Mat3::from_mat4(ubo.modelview) * in_normal;

    let light_pos = Vec3::new(0.0, -5.0, 5.0);
    *out_light_vec = light_pos - *out_pos;
    *out_view_vec = -*out_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_uvw: Vec3,
    _in_pos: Vec3,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_env_map: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image_env_map: &Image!(cube, type=f32, sampled),
    #[spirv(spec_constant(id = 0, default = 0))] type_id: u32,
    out_color0: &mut Vec4,
    out_color1: &mut Vec4,
) {
    let color = match type_id as i32 {
        0 => {
            // Skybox
            let normal = in_uvw.normalize();
            image_env_map.sample(*sampler_env_map, normal)
        }
        1 => {
            // Reflect
            let w_view_vec = Mat3::from_mat4(ubo.inverse_modelview) * in_view_vec.normalize();
            let normal = in_normal.normalize();
            let w_normal = Mat3::from_mat4(ubo.inverse_modelview) * normal;

            let n_dot_l = normal.dot(in_light_vec).max(0.0);

            let eye_dir = in_view_vec.normalize();
            let half_vec = (in_light_vec + eye_dir).normalize();
            let n_dot_h = normal.dot(half_vec).max(0.0);
            let n_dot_v = normal.dot(eye_dir).max(0.0);
            let v_dot_h = eye_dir.dot(half_vec).max(0.0);

            // Geometric attenuation
            let nh2 = 2.0 * n_dot_h;
            let g1 = (nh2 * n_dot_v) / v_dot_h;
            let g2 = (nh2 * n_dot_l) / v_dot_h;
            let geo_att = 1.0_f32.min(g1.min(g2));

            const F0: f32 = 0.6;
            const K: f32 = 0.2;

            // Fresnel (schlick approximation)
            let mut fresnel = (1.0 - v_dot_h).powf(5.0);
            fresnel *= 1.0 - F0;
            fresnel += F0;

            let spec = (fresnel * geo_att) / (n_dot_v * n_dot_l * 3.14);

            let reflect_vec = -w_view_vec.reflect(w_normal);
            let env_color = image_env_map.sample(*sampler_env_map, reflect_vec);

            Vec4::new(
                env_color.x * n_dot_l * (K + spec * (1.0 - K)),
                env_color.y * n_dot_l * (K + spec * (1.0 - K)),
                env_color.z * n_dot_l * (K + spec * (1.0 - K)),
                1.0,
            )
        }
        2 => {
            // Refract
            let w_view_vec = Mat3::from_mat4(ubo.inverse_modelview) * in_view_vec.normalize();
            let w_normal = Mat3::from_mat4(ubo.inverse_modelview) * in_normal;
            let refract_vec = -w_view_vec.refract(w_normal, 1.0 / 1.6);
            image_env_map.sample(*sampler_env_map, refract_vec)
        }
        _ => Vec4::new(1.0, 0.0, 1.0, 1.0),
    };

    // Color with manual exposure into attachment 0
    let exposed = Vec3::ONE - (-Vec3::new(color.x, color.y, color.z) * ubo.exposure).exp();
    out_color0.x = exposed.x;
    out_color0.y = exposed.y;
    out_color0.z = exposed.z;
    out_color0.w = 1.0;

    // Bright parts for bloom into attachment 1
    let l = exposed.dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let threshold = 0.75;
    if l > threshold {
        out_color1.x = exposed.x;
        out_color1.y = exposed.y;
        out_color1.z = exposed.z;
    } else {
        out_color1.x = 0.0;
        out_color1.y = 0.0;
        out_color1.z = 0.0;
    }
    out_color1.w = 1.0;
}
