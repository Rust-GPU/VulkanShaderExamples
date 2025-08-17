#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use spirv_std::image::{Cubemap, SampledImage};
use spirv_std::{num_traits::Float, spirv};

// UBO structure for camera matrices
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UBO {
    projection: Mat4,
    model: Mat4,
    view: Mat4,
    cam_pos: Vec3,
    _padding: f32,
}

// UBO structure for lighting parameters
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UBOParams {
    lights: [Vec4; 4], // offset 0, size 64
    exposure: f32,     // offset 64
    gamma: f32,        // offset 68
}

// Push constants for object position
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PushConstsVertex {
    obj_pos: Vec3,
}

// Push constants for material parameters
// Fragment shader needs to match GLSL layout with explicit offsets
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PushConstsMaterial {
    _padding: [f32; 3], // offset 0-11 to match GLSL offset 12
    roughness: f32,     // offset 12
    metallic: f32,      // offset 16
    specular: f32,      // offset 20
    r: f32,             // offset 24
    g: f32,             // offset 28
    b: f32,             // offset 32
}

use core::f32::consts::PI;

// Reflect function
fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * n.dot(i) * n
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(push_constant)] push_consts: &PushConstsVertex,
    #[spirv(position)] out_pos: &mut Vec4,
    out_world_pos: &mut Vec3,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    let loc_pos = (ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0)).truncate();
    *out_world_pos = loc_pos + push_consts.obj_pos;
    *out_normal = ubo.model.transform_vector3(in_normal);
    *out_uv = Vec2::new(in_uv.x, 1.0 - in_uv.y);
    *out_pos =
        ubo.projection * ubo.view * vec4(out_world_pos.x, out_world_pos.y, out_world_pos.z, 1.0);
}

// Uncharted 2 tone mapping
fn uncharted2_tonemap(x: Vec3) -> Vec3 {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;

    ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f
}

// Normal Distribution function (GGX)
fn d_ggx(dot_nh: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let denom = dot_nh * dot_nh * (alpha2 - 1.0) + 1.0;
    alpha2 / (PI * denom * denom)
}

// Geometric Shadowing function (Schlick-Smith GGX)
fn g_schlicksmith_ggx(dot_nl: f32, dot_nv: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let gl = dot_nl / (dot_nl * (1.0 - k) + k);
    let gv = dot_nv / (dot_nv * (1.0 - k) + k);
    gl * gv
}

// Fresnel function (Schlick approximation)
fn f_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    let one_minus_cos = 1.0 - cos_theta;
    let one_minus_cos_5 =
        one_minus_cos * one_minus_cos * one_minus_cos * one_minus_cos * one_minus_cos;
    f0 + (Vec3::splat(1.0) - f0) * one_minus_cos_5
}

// Fresnel function with roughness
fn f_schlick_r(cos_theta: f32, f0: Vec3, roughness: f32) -> Vec3 {
    let one_minus_roughness = Vec3::splat(1.0 - roughness);
    let one_minus_cos = 1.0 - cos_theta;
    let one_minus_cos_5 =
        one_minus_cos * one_minus_cos * one_minus_cos * one_minus_cos * one_minus_cos;
    f0 + (one_minus_roughness.max(f0) - f0) * one_minus_cos_5
}

// Sample prefiltered environment map
fn prefiltered_reflection(
    r: Vec3,
    roughness: f32,
    prefiltered_map: &SampledImage<Cubemap>,
) -> Vec3 {
    const MAX_REFLECTION_LOD: f32 = 9.0;
    let lod = roughness * MAX_REFLECTION_LOD;
    let lod_f = lod.floor();
    let lod_c = lod.ceil();

    let a = prefiltered_map.sample_by_lod(r, lod_f).truncate();
    let b = prefiltered_map.sample_by_lod(r, lod_c).truncate();

    a * (1.0 - (lod - lod_f)) + b * (lod - lod_f)
}

// Calculate specular contribution from a light
fn specular_contribution(
    l: Vec3,
    v: Vec3,
    n: Vec3,
    f0: Vec3,
    metallic: f32,
    roughness: f32,
    albedo: Vec3,
) -> Vec3 {
    let h = (v + l).normalize();
    let dot_nh = n.dot(h).clamp(0.0, 1.0);
    let dot_nv = n.dot(v).clamp(0.0, 1.0);
    let dot_nl = n.dot(l).clamp(0.0, 1.0);

    let mut color = Vec3::ZERO;

    if dot_nl > 0.0 {
        let d = d_ggx(dot_nh, roughness);
        let g = g_schlicksmith_ggx(dot_nl, dot_nv, roughness);
        let f = f_schlick(dot_nv, f0);

        let spec = (d * f * g) / (4.0 * dot_nl * dot_nv + 0.001);
        let kd = (Vec3::ONE - f) * (1.0 - metallic);

        color += (kd * albedo / PI + spec) * dot_nl;
    }

    color
}

#[spirv(fragment)]
pub fn main_fs(
    in_world_pos: Vec3,
    in_normal: Vec3,
    _in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo_params: &UBOParams,
    #[spirv(push_constant)] material: &PushConstsMaterial,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_irradiance: &SampledImage<Cubemap>,
    #[spirv(descriptor_set = 0, binding = 3)] sampler_brdf_lut: &SampledImage<
        spirv_std::image::Image2d,
    >,
    #[spirv(descriptor_set = 0, binding = 4)] prefiltered_map: &SampledImage<Cubemap>,
    out_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let v = (ubo.cam_pos - in_world_pos).normalize();
    let r = reflect(-v, n);

    let metallic = material.metallic;
    let roughness = material.roughness;
    let albedo = vec3(material.r, material.g, material.b);

    let mut f0 = Vec3::splat(0.04);
    f0 = f0 * (1.0 - metallic) + albedo * metallic;

    // Direct lighting contribution
    let mut lo = Vec3::ZERO;
    for i in 0..4 {
        let l = (ubo_params.lights[i].truncate() - in_world_pos).normalize();
        lo += specular_contribution(l, v, n, f0, metallic, roughness, albedo);
    }

    // Sample BRDF LUT
    let n_dot_v = n.dot(v).max(0.0);
    let brdf = sampler_brdf_lut
        .sample(vec2(n_dot_v, roughness))
        .truncate()
        .truncate();

    // Sample prefiltered reflection
    let reflection = prefiltered_reflection(r, roughness, prefiltered_map);

    // Sample irradiance
    let irradiance = sampler_irradiance.sample(n).truncate();

    // Calculate diffuse
    let diffuse = irradiance * albedo;

    // Calculate Fresnel
    let f = f_schlick_r(n_dot_v, f0, roughness);

    // Specular reflectance
    let specular = reflection * (f * brdf.x + brdf.y);

    // Ambient part
    let mut kd = Vec3::ONE - f;
    kd *= 1.0 - metallic;
    let ambient = kd * diffuse + specular;

    // Final color
    let mut color = ambient + lo;

    // Tone mapping
    color = uncharted2_tonemap(color * ubo_params.exposure);
    let white_scale = Vec3::ONE / uncharted2_tonemap(Vec3::splat(11.2));
    color = color * white_scale;

    // Gamma correction
    let inv_gamma = 1.0 / ubo_params.gamma;
    color = vec3(
        color.x.powf(inv_gamma),
        color.y.powf(inv_gamma),
        color.z.powf(inv_gamma),
    );

    *out_color = vec4(color.x, color.y, color.z, 1.0);
}
