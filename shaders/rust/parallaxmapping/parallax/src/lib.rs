#![no_std]

use spirv_std::glam::{Mat3, Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VertexUBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
    pub camera_pos: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FragmentUBO {
    pub height_scale: f32,
    pub parallax_bias: f32,
    pub num_layers: f32,
    pub mapping_mode: i32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_uv: Vec2,
    in_normal: Vec3,
    in_tangent: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &VertexUBO,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_tangent_light_pos: &mut Vec3,
    out_tangent_view_pos: &mut Vec3,
    out_tangent_frag_pos: &mut Vec3,
) {
    *out_position =
        ubo.projection * ubo.view * ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let frag_pos = (ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0)).xyz();
    *out_uv = in_uv;

    let model_mat3 = Mat3::from_mat4(ubo.model);
    let n = (model_mat3 * in_normal).normalize();
    let t = (model_mat3 * in_tangent.xyz()).normalize();
    let b = n.cross(t).normalize();
    let tbn = Mat3::from_cols(t, b, n).transpose();

    *out_tangent_light_pos = tbn * ubo.light_pos.xyz();
    *out_tangent_view_pos = tbn * ubo.camera_pos.xyz();
    *out_tangent_frag_pos = tbn * frag_pos;
}

fn parallax_mapping(
    uv: Vec2,
    view_dir: Vec3,
    normal_height_map: &Image!(2D, type=f32, sampled),
    sampler: &Sampler,
    ubo: &FragmentUBO,
) -> Vec2 {
    let height = 1.0 - normal_height_map.sample_by_lod(*sampler, uv, 0.0).w;
    let p = view_dir.xy() * (height * (ubo.height_scale * 0.5) + ubo.parallax_bias) / view_dir.z;
    uv - p
}

fn steep_parallax_mapping(
    uv: Vec2,
    view_dir: Vec3,
    normal_height_map: &Image!(2D, type=f32, sampled),
    sampler: &Sampler,
    ubo: &FragmentUBO,
) -> Vec2 {
    let layer_depth = 1.0 / ubo.num_layers;
    let mut curr_layer_depth = 0.0;
    let delta_uv = view_dir.xy() * ubo.height_scale / (view_dir.z * ubo.num_layers);
    let mut curr_uv = uv;

    for _ in 0..(ubo.num_layers as i32) {
        curr_layer_depth += layer_depth;
        curr_uv -= delta_uv;
        let height = 1.0 - normal_height_map.sample_by_lod(*sampler, curr_uv, 0.0).w;
        if height < curr_layer_depth {
            break;
        }
    }
    curr_uv
}

fn parallax_occlusion_mapping(
    uv: Vec2,
    view_dir: Vec3,
    normal_height_map: &Image!(2D, type=f32, sampled),
    sampler: &Sampler,
    ubo: &FragmentUBO,
) -> Vec2 {
    let layer_depth = 1.0 / ubo.num_layers;
    let mut curr_layer_depth = 0.0;
    let delta_uv = view_dir.xy() * ubo.height_scale / (view_dir.z * ubo.num_layers);
    let mut curr_uv = uv;
    let mut height = 1.0 - normal_height_map.sample_by_lod(*sampler, curr_uv, 0.0).w;

    for _ in 0..(ubo.num_layers as i32) {
        curr_layer_depth += layer_depth;
        curr_uv -= delta_uv;
        height = 1.0 - normal_height_map.sample_by_lod(*sampler, curr_uv, 0.0).w;
        if height < curr_layer_depth {
            break;
        }
    }

    let prev_uv = curr_uv + delta_uv;
    let next_depth = height - curr_layer_depth;
    let prev_depth =
        1.0 - normal_height_map.sample_by_lod(*sampler, prev_uv, 0.0).w - curr_layer_depth
            + layer_depth;
    prev_uv.lerp(curr_uv, next_depth / (next_depth - prev_depth))
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_tangent_light_pos: Vec3,
    in_tangent_view_pos: Vec3,
    in_tangent_frag_pos: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] color_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] color_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] normal_height_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] normal_sampler: &Sampler,
    #[spirv(uniform, descriptor_set = 0, binding = 3)] ubo: &FragmentUBO,
    out_color: &mut Vec4,
) {
    let v = (in_tangent_view_pos - in_tangent_frag_pos).normalize();
    let mut uv = in_uv;

    if ubo.mapping_mode == 0 {
        // Color only
        *out_color = color_map.sample(*color_sampler, in_uv);
    } else {
        match ubo.mapping_mode {
            2 => uv = parallax_mapping(in_uv, v, normal_height_map, normal_sampler, ubo),
            3 => uv = steep_parallax_mapping(in_uv, v, normal_height_map, normal_sampler, ubo),
            4 => uv = parallax_occlusion_mapping(in_uv, v, normal_height_map, normal_sampler, ubo),
            _ => {}
        }

        // Perform sampling before (potentially) discarding
        let normal_height_map_lod = normal_height_map
            .sample_by_lod(*normal_sampler, uv, 0.0)
            .xyz();
        let color = color_map.sample(*color_sampler, uv).xyz();

        // Discard fragments at texture border
        if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
            spirv_std::arch::kill();
        }

        let n = (normal_height_map_lod * 2.0 - 1.0).normalize();
        let l = (in_tangent_light_pos - in_tangent_frag_pos).normalize();
        let h = (l + v).normalize();

        let ambient = 0.2 * color;
        let diffuse = l.dot(n).max(0.0) * color;
        let specular = Vec3::splat(0.15) * n.dot(h).max(0.0).powf(32.0);

        *out_color = Vec4::new(
            ambient.x + diffuse.x + specular.x,
            ambient.y + diffuse.y + specular.y,
            ambient.z + diffuse.z + specular.z,
            1.0,
        );
    }
}
