#![no_std]

use spirv_std::glam::{mat4, vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};

const SHADOW_MAP_CASCADE_COUNT: usize = 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub position: Vec4,
    pub cascade_index: u32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_pos: &mut Vec3,
    out_pos: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_color = in_color;
    *out_normal = in_normal;
    *out_uv = in_uv;
    let pos = in_pos + push_consts.position.xyz();
    *out_pos = pos;
    *out_view_pos = (ubo.view * vec4(pos.x, pos.y, pos.z, 1.0)).xyz();
    *out_position = ubo.projection * ubo.view * ubo.model * vec4(pos.x, pos.y, pos.z, 1.0);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO2 {
    pub cascade_splits: Vec4,
    pub inverse_view_mat: Mat4,
    pub light_dir: Vec3,
    pub _pad: f32,
    pub color_cascades: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CVPM {
    pub matrices: [Mat4; SHADOW_MAP_CASCADE_COUNT],
}

const AMBIENT: f32 = 0.3;

fn texture_proj(
    shadow_coord: Vec4,
    offset: Vec2,
    cascade_index: u32,
    shadow_map: &Image!(2D, type=f32, sampled, arrayed),
    sampler: &Sampler,
) -> f32 {
    let mut shadow = 1.0;
    let bias = 0.005;

    if shadow_coord.z > -1.0 && shadow_coord.z < 1.0 {
        let dist = shadow_map
            .sample(
                *sampler,
                vec3(
                    shadow_coord.x + offset.x,
                    shadow_coord.y + offset.y,
                    cascade_index as f32,
                ),
            )
            .x;
        if shadow_coord.w > 0.0 && dist < shadow_coord.z - bias {
            shadow = AMBIENT;
        }
    }
    shadow
}

fn filter_pcf(
    sc: Vec4,
    cascade_index: u32,
    shadow_map: &Image!(2D, type=f32, sampled, arrayed),
    sampler: &Sampler,
) -> f32 {
    // For arrayed textures, we need to use a hardcoded size or pass it as a parameter
    // GLSL textureSize returns the size without the array dimension
    let tex_dim = vec2(2048.0, 2048.0); // Common shadow map size, adjust as needed
    let scale = 0.75;
    let dx = scale * 1.0 / tex_dim.x;
    let dy = scale * 1.0 / tex_dim.y;

    let mut shadow_factor = 0.0;
    let mut count = 0;
    let range = 1;

    for x in -range..=range {
        for y in -range..=range {
            shadow_factor += texture_proj(
                sc,
                vec2(dx * x as f32, dy * y as f32),
                cascade_index,
                shadow_map,
                sampler,
            );
            count += 1;
        }
    }
    shadow_factor / count as f32
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_pos: Vec3,
    in_pos: Vec3,
    in_uv: Vec2,
    #[spirv(spec_constant(id = 0))] enable_pcf: u32,
    #[spirv(descriptor_set = 0, binding = 1)] shadow_map: &Image!(2D, type=f32, sampled, arrayed),
    #[spirv(descriptor_set = 0, binding = 1)] shadow_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0)] color_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 1, binding = 0)] color_sampler: &Sampler,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] ubo: &UBO2,
    #[spirv(uniform, descriptor_set = 0, binding = 3)] cascade_view_proj_matrices: &CVPM,
    out_frag_color: &mut Vec4,
) {
    let color = color_map.sample(*color_sampler, in_uv);
    if color.w < 0.5 {
        spirv_std::arch::kill();
    }

    // Get cascade index for the current fragment's view position
    let mut cascade_index = 0u32;
    if in_view_pos.z < ubo.cascade_splits.x {
        cascade_index = 1;
    } else if in_view_pos.z < ubo.cascade_splits.y {
        cascade_index = 2;
    } else if in_view_pos.z < ubo.cascade_splits.z {
        cascade_index = 3;
    }

    // Depth compare for shadowing
    let bias_mat = mat4(
        vec4(0.5, 0.0, 0.0, 0.0),
        vec4(0.0, 0.5, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.5, 0.5, 0.0, 1.0),
    );
    let cascade_matrix = match cascade_index {
        0 => cascade_view_proj_matrices.matrices[0],
        1 => cascade_view_proj_matrices.matrices[1],
        2 => cascade_view_proj_matrices.matrices[2],
        3 => cascade_view_proj_matrices.matrices[3],
        _ => cascade_view_proj_matrices.matrices[0],
    };
    let shadow_coord = (bias_mat * cascade_matrix) * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let shadow = if enable_pcf == 1 {
        filter_pcf(
            shadow_coord / shadow_coord.w,
            cascade_index,
            shadow_map,
            shadow_sampler,
        )
    } else {
        texture_proj(
            shadow_coord / shadow_coord.w,
            vec2(0.0, 0.0),
            cascade_index,
            shadow_map,
            shadow_sampler,
        )
    };

    // Directional light
    let n = in_normal.normalize();
    let l = (-ubo.light_dir).normalize();
    let diffuse = n.dot(l).max(AMBIENT);
    let light_color = vec3(1.0, 1.0, 1.0);
    let mut frag_color = (light_color * (diffuse * color.xyz())).max(vec3(0.0, 0.0, 0.0));
    frag_color *= shadow;

    // Color cascades (if enabled)
    if ubo.color_cascades == 1 {
        frag_color *= match cascade_index {
            0 => vec3(1.0, 0.25, 0.25),
            1 => vec3(0.25, 1.0, 0.25),
            2 => vec3(0.25, 0.25, 1.0),
            3 => vec3(1.0, 1.0, 0.25),
            _ => vec3(1.0, 1.0, 1.0),
        };
    }

    *out_frag_color = vec4(frag_color.x, frag_color.y, frag_color.z, color.w);
}
