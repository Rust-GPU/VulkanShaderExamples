#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub tess_alpha: f32,
    pub tess_level: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PnPatch {
    pub b210: f32,
    pub b120: f32,
    pub b021: f32,
    pub b012: f32,
    pub b102: f32,
    pub b201: f32,
    pub b111: f32,
    pub n110: f32,
    pub n011: f32,
    pub n101: f32,
}

fn wij(i: usize, j: usize, positions: &[Vec4; 3], normals: &[Vec3; 3]) -> f32 {
    (positions[j].truncate() - positions[i].truncate()).dot(normals[i])
}

fn vij(i: usize, j: usize, positions: &[Vec4; 3], normals: &[Vec3; 3]) -> f32 {
    let pj_minus_pi = positions[j].truncate() - positions[i].truncate();
    let ni_plus_nj = normals[i] + normals[j];
    2.0 * pj_minus_pi.dot(ni_plus_nj) / pj_minus_pi.dot(pj_minus_pi)
}

#[spirv(tessellation_control(output_vertices = 3))]
pub fn main_tcs(
    #[spirv(invocation_id)] invocation_id: u32,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_uv: [Vec2; 3],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut [Vec4; 3],
    out_normal: &mut [Vec3; 3],
    out_uv: &mut [Vec2; 3],
    out_patch: &mut [PnPatch; 3],
    #[spirv(tess_level_inner)] tess_level_inner: &mut [f32; 2],
    #[spirv(tess_level_outer)] tess_level_outer: &mut [f32; 4],
) {
    let inv_id = invocation_id as usize;

    // Pass through vertex data
    out_position[inv_id] = in_position[inv_id];
    out_normal[inv_id] = in_normal[inv_id];
    out_uv[inv_id] = in_uv[inv_id];

    // Extract component for current invocation
    // In GLSL, gl_InvocationID is used as a swizzle index (0=x, 1=y, 2=z)
    // to extract a single float from each vertex's position/normal
    let p0 = match inv_id {
        0 => in_position[0].x,
        1 => in_position[0].y,
        2 => in_position[0].z,
        _ => in_position[0].w,
    };
    let p1 = match inv_id {
        0 => in_position[1].x,
        1 => in_position[1].y,
        2 => in_position[1].z,
        _ => in_position[1].w,
    };
    let p2 = match inv_id {
        0 => in_position[2].x,
        1 => in_position[2].y,
        2 => in_position[2].z,
        _ => in_position[2].w,
    };
    let n0 = match inv_id {
        0 => in_normal[0].x,
        1 => in_normal[0].y,
        2 => in_normal[0].z,
        _ => 0.0,
    };
    let n1 = match inv_id {
        0 => in_normal[1].x,
        1 => in_normal[1].y,
        2 => in_normal[1].z,
        _ => 0.0,
    };
    let n2 = match inv_id {
        0 => in_normal[2].x,
        1 => in_normal[2].y,
        2 => in_normal[2].z,
        _ => 0.0,
    };

    // Compute control points
    out_patch[inv_id].b210 = (2.0 * p0 + p1 - wij(0, 1, &in_position, &in_normal) * n0) / 3.0;
    out_patch[inv_id].b120 = (2.0 * p1 + p0 - wij(1, 0, &in_position, &in_normal) * n1) / 3.0;
    out_patch[inv_id].b021 = (2.0 * p1 + p2 - wij(1, 2, &in_position, &in_normal) * n1) / 3.0;
    out_patch[inv_id].b012 = (2.0 * p2 + p1 - wij(2, 1, &in_position, &in_normal) * n2) / 3.0;
    out_patch[inv_id].b102 = (2.0 * p2 + p0 - wij(2, 0, &in_position, &in_normal) * n2) / 3.0;
    out_patch[inv_id].b201 = (2.0 * p0 + p2 - wij(0, 2, &in_position, &in_normal) * n0) / 3.0;

    let e = (out_patch[inv_id].b210
        + out_patch[inv_id].b120
        + out_patch[inv_id].b021
        + out_patch[inv_id].b012
        + out_patch[inv_id].b102
        + out_patch[inv_id].b201)
        / 6.0;
    let v = (p0 + p1 + p2) / 3.0;
    out_patch[inv_id].b111 = e + (e - v) * 0.5;

    // Compute normal control points
    out_patch[inv_id].n110 = n0 + n1 - vij(0, 1, &in_position, &in_normal) * (p1 - p0);
    out_patch[inv_id].n011 = n1 + n2 - vij(1, 2, &in_position, &in_normal) * (p2 - p1);
    out_patch[inv_id].n101 = n2 + n0 - vij(2, 0, &in_position, &in_normal) * (p0 - p2);

    // Set tessellation levels
    tess_level_outer[inv_id] = ubo.tess_level;
    tess_level_inner[0] = ubo.tess_level;
}

#[spirv(tessellation_evaluation(triangles, spacing_equal, vertex_order_cw))]
pub fn main_tes(
    #[spirv(tess_coord)] tess_coord: Vec3,
    #[spirv(position)] in_position: [Vec4; 3],
    in_normal: [Vec3; 3],
    in_uv: [Vec2; 3],
    in_patch: [PnPatch; 3],
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    let u = tess_coord.x;
    let v = tess_coord.y;
    let w = tess_coord.z;

    let uu = u * u;
    let vv = v * v;
    let ww = w * w;
    let uvw_cubed = Vec3::new(uu * u, vv * v, ww * w);

    // Extract control points as vec3s from the three patches
    let b210 = Vec3::new(in_patch[0].b210, in_patch[1].b210, in_patch[2].b210);
    let b120 = Vec3::new(in_patch[0].b120, in_patch[1].b120, in_patch[2].b120);
    let b021 = Vec3::new(in_patch[0].b021, in_patch[1].b021, in_patch[2].b021);
    let b012 = Vec3::new(in_patch[0].b012, in_patch[1].b012, in_patch[2].b012);
    let b102 = Vec3::new(in_patch[0].b102, in_patch[1].b102, in_patch[2].b102);
    let b201 = Vec3::new(in_patch[0].b201, in_patch[1].b201, in_patch[2].b201);
    let b111 = Vec3::new(in_patch[0].b111, in_patch[1].b111, in_patch[2].b111);

    // Extract control normals
    let n110 = Vec3::new(in_patch[0].n110, in_patch[1].n110, in_patch[2].n110).normalize();
    let n011 = Vec3::new(in_patch[0].n011, in_patch[1].n011, in_patch[2].n011).normalize();
    let n101 = Vec3::new(in_patch[0].n101, in_patch[1].n101, in_patch[2].n101).normalize();

    // Compute texcoords - note the order: w*tc0 + u*tc1 + v*tc2
    *out_uv = w * in_uv[0] + u * in_uv[1] + v * in_uv[2];

    // Compute normals
    // Barycentric normal
    let bar_normal = w * in_normal[0] + u * in_normal[1] + v * in_normal[2];
    // PN normal
    let pn_normal = in_normal[0] * ww
        + in_normal[1] * uu
        + in_normal[2] * vv
        + n110 * w * u
        + n011 * u * v
        + n101 * w * v;
    *out_normal = (ubo.tess_alpha * pn_normal + (1.0 - ubo.tess_alpha) * bar_normal).normalize();

    // Compute position
    // Barycentric position
    let bar_pos = w * in_position[0].truncate()
        + u * in_position[1].truncate()
        + v * in_position[2].truncate();

    // Save some computations
    let uvw_squared3 = Vec3::new(uu * 3.0, vv * 3.0, ww * 3.0);

    // Compute PN position
    let pn_pos = in_position[0].truncate() * uvw_cubed.z
        + in_position[1].truncate() * uvw_cubed.x
        + in_position[2].truncate() * uvw_cubed.y
        + b210 * uvw_squared3.z * u
        + b120 * uvw_squared3.x * w
        + b201 * uvw_squared3.z * v
        + b021 * uvw_squared3.x * v
        + b102 * uvw_squared3.y * w
        + b012 * uvw_squared3.y * u
        + b111 * 6.0 * u * v * w;

    // Final position
    let final_pos = (1.0 - ubo.tess_alpha) * bar_pos + ubo.tess_alpha * pn_pos;
    *out_position =
        ubo.projection * ubo.model * Vec4::new(final_pos.x, final_pos.y, final_pos.z, 1.0);
}
