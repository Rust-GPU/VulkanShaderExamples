#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use core::f32::consts::PI;
use spirv_std::{
    glam::{mat3, vec3, vec4, Mat4, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub view: Mat4,
    pub cam_pos: Vec3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub obj_pos: Vec3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UniformInline {
    pub roughness: f32,
    pub metallic: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub ambient: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_world_pos: &mut Vec3,
    out_normal: &mut Vec3,
) {
    let loc_pos = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).truncate();
    *out_world_pos = loc_pos + push_consts.obj_pos;
    let model_mat3 = mat3(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    *out_normal = model_mat3 * in_normal;
    *out_position =
        ubo.projection * ubo.view * vec4(out_world_pos.x, out_world_pos.y, out_world_pos.z, 1.0);
}

// Normal Distribution function --------------------------------------
fn d_ggx(dot_nh: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let denom = dot_nh * dot_nh * (alpha2 - 1.0) + 1.0;
    alpha2 / (PI * denom * denom)
}

// Geometric Shadowing function --------------------------------------
fn g_schlicksmith_ggx(dot_nl: f32, dot_nv: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let gl = dot_nl / (dot_nl * (1.0 - k) + k);
    let gv = dot_nv / (dot_nv * (1.0 - k) + k);
    gl * gv
}

// Fresnel function ----------------------------------------------------
fn f_schlick(cos_theta: f32, metallic: f32, material_color: Vec3) -> Vec3 {
    let f0 = vec3(0.04, 0.04, 0.04).lerp(material_color, metallic);
    f0 + (vec3(1.0, 1.0, 1.0) - f0) * (1.0 - cos_theta).powf(5.0)
}

// Specular BRDF composition --------------------------------------------
fn brdf(l: Vec3, v: Vec3, n: Vec3, metallic: f32, roughness: f32, material_color: Vec3) -> Vec3 {
    // Precalculate vectors and dot products
    let h = (v + l).normalize();
    let dot_nv = n.dot(v).clamp(0.0, 1.0);
    let dot_nl = n.dot(l).clamp(0.0, 1.0);
    let _dot_lh = l.dot(h).clamp(0.0, 1.0);
    let dot_nh = n.dot(h).clamp(0.0, 1.0);

    // Light color fixed
    let light_color = vec3(1.0, 1.0, 1.0);

    let mut color = vec3(0.0, 0.0, 0.0);

    if dot_nl > 0.0 {
        let rroughness = roughness.max(0.05);
        // D = Normal distribution (Distribution of the microfacets)
        let d = d_ggx(dot_nh, rroughness);
        // G = Geometric shadowing term (Microfacets shadowing)
        let g = g_schlicksmith_ggx(dot_nl, dot_nv, rroughness);
        // F = Fresnel factor (Reflectance depending on angle of incidence)
        let f = f_schlick(dot_nv, metallic, material_color);

        let spec = d * f * g / (4.0 * dot_nl * dot_nv);

        color += spec * dot_nl * light_color;
    }

    color
}

#[spirv(fragment)]
pub fn main_fs(
    in_world_pos: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] material: &UniformInline,
    out_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let v = (ubo.cam_pos - in_world_pos).normalize();

    let roughness = material.roughness;
    let material_color = vec3(material.r, material.g, material.b);

    // Specular contribution
    let light_pos = vec3(0.0, 0.0, 10.0);
    let mut lo = vec3(0.0, 0.0, 0.0);
    let l = (light_pos - in_world_pos).normalize();
    lo += brdf(l, v, n, material.metallic, roughness, material_color);

    // Combine with ambient
    let mut color = material_color * material.ambient;
    color += lo;

    // Gamma correct
    color = color.powf(0.4545);

    *out_color = vec4(color.x, color.y, color.z, 1.0);
}
