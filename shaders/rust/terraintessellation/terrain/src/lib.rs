#![no_std]

use spirv_std::glam::{vec3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub modelview: Mat4,
    pub light_pos: Vec4,
    pub frustum_planes: [Vec4; 6],
    pub displacement_factor: f32,
    pub tessellation_factor: f32,
    pub viewport_dim: Vec2,
    pub tessellated_edge_size: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_position = Vec4::from((in_pos, 1.0));
    *out_uv = in_uv;
    *out_normal = in_normal;
}

fn screen_space_tess_factor(p0: Vec4, p1: Vec4, ubo: &UBO) -> f32 {
    let mid_point = 0.5 * (p0 + p1);
    let radius = p0.distance(p1) / 2.0;

    let v0 = ubo.modelview * mid_point;

    let clip0 = ubo.projection * (v0 - Vec4::new(radius, 0.0, 0.0, 0.0));
    let clip1 = ubo.projection * (v0 + Vec4::new(radius, 0.0, 0.0, 0.0));

    let clip0 = clip0 / clip0.w;
    let clip1 = clip1 / clip1.w;

    let clip0_vp = clip0.xy() * ubo.viewport_dim;
    let clip1_vp = clip1.xy() * ubo.viewport_dim;

    (clip0_vp.distance(clip1_vp) / ubo.tessellated_edge_size * ubo.tessellation_factor)
        .clamp(1.0, 64.0)
}

fn frustum_check(
    pos: Vec4,
    uv: Vec2,
    ubo: &UBO,
    sampler_height: &Image!(2D, type=f32, sampled),
    sampler: &Sampler,
) -> bool {
    const RADIUS: f32 = 8.0;
    let mut pos = pos;
    pos.y -= sampler_height.sample_by_lod(*sampler, uv, 0.0).x * ubo.displacement_factor;

    for i in 0..6 {
        if pos.dot(ubo.frustum_planes[i]) + RADIUS < 0.0 {
            return false;
        }
    }
    true
}

#[spirv(tessellation_control(output_vertices = 4))]
pub fn main_tcs(
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(position)] in_position: [Vec4; 4],
    in_normal: [Vec3; 4],
    in_uv: [Vec2; 4],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_height: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(position)] out_position: &mut [Vec4; 4],
    out_normal: &mut [Vec3; 4],
    out_uv: &mut [Vec2; 4],
    #[spirv(tess_level_inner)] tess_level_inner: &mut [f32; 2],
    #[spirv(tess_level_outer)] tess_level_outer: &mut [f32; 4],
) {
    if invocation_id == 0 {
        if !frustum_check(
            in_position[invocation_id as usize],
            in_uv[0],
            ubo,
            sampler_height,
            sampler,
        ) {
            tess_level_inner[0] = 0.0;
            tess_level_inner[1] = 0.0;
            tess_level_outer[0] = 0.0;
            tess_level_outer[1] = 0.0;
            tess_level_outer[2] = 0.0;
            tess_level_outer[3] = 0.0;
        } else {
            if ubo.tessellation_factor > 0.0 {
                tess_level_outer[0] = screen_space_tess_factor(in_position[3], in_position[0], ubo);
                tess_level_outer[1] = screen_space_tess_factor(in_position[0], in_position[1], ubo);
                tess_level_outer[2] = screen_space_tess_factor(in_position[1], in_position[2], ubo);
                tess_level_outer[3] = screen_space_tess_factor(in_position[2], in_position[3], ubo);
                tess_level_inner[0] = (tess_level_outer[0] + tess_level_outer[3]) * 0.5;
                tess_level_inner[1] = (tess_level_outer[2] + tess_level_outer[1]) * 0.5;
            } else {
                tess_level_inner[0] = 1.0;
                tess_level_inner[1] = 1.0;
                tess_level_outer[0] = 1.0;
                tess_level_outer[1] = 1.0;
                tess_level_outer[2] = 1.0;
                tess_level_outer[3] = 1.0;
            }
        }
    }

    let idx = invocation_id as usize;
    out_position[idx] = in_position[idx];
    out_normal[idx] = in_normal[idx];
    out_uv[idx] = in_uv[idx];
}

#[spirv(tessellation_evaluation(quads, vertex_order_cw, spacing_equal))]
pub fn main_tes(
    #[spirv(tess_coord)] tess_coord: Vec3,
    #[spirv(position)] in_position: [Vec4; 4],
    in_normal: [Vec3; 4],
    in_uv: [Vec2; 4],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(descriptor_set = 0, binding = 1)] displacement_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
    out_eye_pos: &mut Vec3,
    out_world_pos: &mut Vec3,
) {
    let uv1 = in_uv[0].lerp(in_uv[1], tess_coord.x);
    let uv2 = in_uv[3].lerp(in_uv[2], tess_coord.x);
    *out_uv = uv1.lerp(uv2, tess_coord.y);

    let n1 = in_normal[0].lerp(in_normal[1], tess_coord.x);
    let n2 = in_normal[3].lerp(in_normal[2], tess_coord.x);
    *out_normal = n1.lerp(n2, tess_coord.y);

    let pos1 = in_position[0].lerp(in_position[1], tess_coord.x);
    let pos2 = in_position[3].lerp(in_position[2], tess_coord.x);
    let mut pos = pos1.lerp(pos2, tess_coord.y);

    pos.y -= displacement_map.sample_by_lod(*sampler, *out_uv, 0.0).x * ubo.displacement_factor;

    *out_position = ubo.projection * ubo.modelview * pos;

    *out_view_vec = -pos.xyz();
    *out_light_vec = (ubo.light_pos.xyz() + *out_view_vec).normalize();
    *out_world_pos = pos.xyz();
    *out_eye_pos = (ubo.modelview * pos).xyz();
}

fn sample_terrain_layer(
    uv: Vec2,
    sampler_height: &Image!(2D, type=f32, sampled),
    sampler_layers: &Image!(2D, type=f32, sampled, arrayed),
    sampler: &Sampler,
) -> Vec3 {
    let layers = [
        Vec2::new(-10.0, 10.0),
        Vec2::new(5.0, 45.0),
        Vec2::new(45.0, 80.0),
        Vec2::new(75.0, 100.0),
        Vec2::new(95.0, 140.0),
        Vec2::new(140.0, 190.0),
    ];

    let mut color = Vec3::ZERO;

    let height = sampler_height.sample_by_lod(*sampler, uv, 0.0).x * 255.0;

    for i in 0..6 {
        let range = layers[i].y - layers[i].x;
        let weight = ((range - (height - layers[i].y).abs()) / range).max(0.0);
        let tex_coord = uv * 16.0;
        color += weight
            * sampler_layers
                .sample(*sampler, vec3(tex_coord.x, tex_coord.y, i as f32))
                .xyz();
    }

    color
}

fn fog(depth: f32) -> f32 {
    const LOG2: f32 = -1.442695;
    let dist = depth * 0.1;
    let d = 0.25 * dist;
    1.0 - (d * d * LOG2).exp2().clamp(0.0, 1.0)
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    in_normal: Vec3,
    in_uv: Vec2,
    _in_view_vec: Vec3,
    in_light_vec: Vec3,
    _in_eye_pos: Vec3,
    _in_world_pos: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_height: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] sampler_layers: &Image!(2D, type=f32, sampled, arrayed),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    out_frag_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let ambient = Vec3::splat(0.5);
    let diffuse = n.dot(l).max(0.0) * Vec3::ONE;

    let color = Vec4::from((
        (ambient + diffuse) * sample_terrain_layer(in_uv, sampler_height, sampler_layers, sampler),
        1.0,
    ));

    const FOG_COLOR: Vec4 = Vec4::new(0.47, 0.5, 0.67, 0.0);
    *out_frag_color = color.lerp(FOG_COLOR, fog(frag_coord.z / frag_coord.w));
}
