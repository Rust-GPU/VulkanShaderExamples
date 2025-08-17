#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles},
    image::SampledImage,
    num_traits::Float,
    spirv, Image,
};

const LIGHT_COUNT: usize = 3;
const SHADOW_FACTOR: f32 = 0.25;
const AMBIENT_LIGHT: f32 = 0.1;
const USE_PCF: bool = true;
const SHADOW_MAP_SIZE: f32 = 2048.0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec4,
    pub target: Vec4,
    pub color: Vec4,
    pub view_matrix: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub view_pos: Vec4,
    pub lights: [Light; LIGHT_COUNT],
    pub use_shadows: i32,
    pub display_debug_target: i32,
}

fn texture_proj(
    shadow_map: &SampledImage<Image!(2D, type=f32, sampled, arrayed)>,
    p: Vec4,
    layer: f32,
    offset: Vec2,
) -> f32 {
    let mut shadow = 1.0;
    let shadow_coord = p / p.w;
    let shadow_coord_xy = shadow_coord.xy() * 0.5 + 0.5;

    if shadow_coord.z > -1.0 && shadow_coord.z < 1.0 {
        let sample_coord = Vec3::new(
            shadow_coord_xy.x + offset.x,
            shadow_coord_xy.y + offset.y,
            layer,
        );
        let dist = shadow_map.sample(sample_coord).x;
        if shadow_coord.w > 0.0 && dist < shadow_coord.z {
            shadow = SHADOW_FACTOR;
        }
    }
    shadow
}

fn filter_pcf(
    shadow_map: &SampledImage<Image!(2D, type=f32, sampled, arrayed)>,
    sc: Vec4,
    layer: f32,
) -> f32 {
    let scale = 1.5;
    let dx = scale * 1.0 / SHADOW_MAP_SIZE;
    let dy = scale * 1.0 / SHADOW_MAP_SIZE;

    let mut shadow_factor = 0.0;
    let mut count = 0;
    let range = 1;

    for x in -range..=range {
        for y in -range..=range {
            shadow_factor += texture_proj(
                shadow_map,
                sc,
                layer,
                Vec2::new(dx * x as f32, dy * y as f32),
            );
            count += 1;
        }
    }
    shadow_factor / count as f32
}

fn shadow(
    frag_color: Vec3,
    frag_pos: Vec3,
    ubo: &UBO,
    shadow_map: &SampledImage<Image!(2D, type=f32, sampled, arrayed)>,
) -> Vec3 {
    let mut result = frag_color;
    for i in 0..LIGHT_COUNT {
        let shadow_clip =
            ubo.lights[i].view_matrix * Vec4::new(frag_pos.x, frag_pos.y, frag_pos.z, 1.0);

        let shadow_factor = if USE_PCF {
            filter_pcf(shadow_map, shadow_clip, i as f32)
        } else {
            texture_proj(shadow_map, shadow_clip, i as f32, Vec2::ZERO)
        };

        result *= shadow_factor;
    }
    result
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: u32,
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
    #[spirv(descriptor_set = 0, binding = 1)] position_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 2)] normal_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 3)] albedo_sampler: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(uniform, descriptor_set = 0, binding = 4)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 5)] shadow_map: &SampledImage<
        Image!(2D, type=f32, sampled, arrayed),
    >,
    out_frag_color: &mut Vec4,
) {
    // Get G-Buffer values
    let frag_pos = position_sampler.sample(in_uv).xyz();
    let normal = normal_sampler.sample(in_uv).xyz();
    let albedo = albedo_sampler.sample(in_uv);

    let mut frag_color;

    // Debug display
    if ubo.display_debug_target > 0 {
        frag_color = match ubo.display_debug_target {
            1 => shadow(Vec3::ONE, frag_pos, ubo, shadow_map),
            2 => frag_pos,
            3 => normal,
            4 => albedo.xyz(),
            5 => Vec3::splat(albedo.w),
            _ => Vec3::ZERO,
        };
        *out_frag_color = Vec4::new(frag_color.x, frag_color.y, frag_color.z, 1.0);
        return;
    }

    // Ambient part
    frag_color = albedo.xyz() * AMBIENT_LIGHT;

    let n = normal.normalize();

    for i in 0..LIGHT_COUNT {
        // Vector to light
        let mut l = ubo.lights[i].position.xyz() - frag_pos;
        let dist = l.length();
        l = l.normalize();

        // Viewer to fragment
        let v = (ubo.view_pos.xyz() - frag_pos).normalize();

        let light_cos_inner_angle = 15.0f32.to_radians().cos();
        let light_cos_outer_angle = 25.0f32.to_radians().cos();
        let light_range = 100.0;

        // Direction vector from source to target
        let dir = (ubo.lights[i].position.xyz() - ubo.lights[i].target.xyz()).normalize();

        // Dual cone spot light with smooth transition between inner and outer angle
        let cos_dir = l.dot(dir);
        let spot_effect = smoothstep(light_cos_outer_angle, light_cos_inner_angle, cos_dir);
        let height_attenuation = smoothstep(light_range, 0.0, dist);

        // Diffuse lighting
        let ndot_l = n.dot(l).max(0.0);
        let diff = Vec3::splat(ndot_l);

        // Specular lighting
        let r = reflect(-l, n);
        let ndot_r = r.dot(v).max(0.0);
        let spec = Vec3::splat(ndot_r.powf(16.0) * albedo.w * 2.5);

        frag_color += (diff + spec)
            * spot_effect
            * height_attenuation
            * ubo.lights[i].color.xyz()
            * albedo.xyz();
    }

    // Shadow calculations in a separate pass
    if ubo.use_shadows > 0 {
        frag_color = shadow(frag_color, frag_pos, ubo, shadow_map);
    }

    *out_frag_color = Vec4::new(frag_color.x, frag_color.y, frag_color.z, 1.0);
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * n.dot(i) * n
}
