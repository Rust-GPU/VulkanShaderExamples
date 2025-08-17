#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::glam::Vec4Swizzles;
use spirv_std::ray_tracing::{AccelerationStructure, RayFlags};
use spirv_std::Image;
use spirv_std::{
    glam::{vec2, vec3, vec4, IVec3, Mat4, Vec2, Vec3},
    spirv,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraProperties {
    pub view_inverse: Mat4,
    pub proj_inverse: Mat4,
}

#[spirv(ray_generation)]
pub fn main_rgen(
    #[spirv(launch_id)] launch_id: IVec3,
    #[spirv(launch_size)] launch_size: IVec3,
    #[spirv(descriptor_set = 0, binding = 0)] top_level_as: &AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] image: &Image!(2D, type=f32, sampled=false),
    #[spirv(uniform, descriptor_set = 0, binding = 2)] cam: &CameraProperties,
    #[spirv(ray_payload)] hit_value: &mut Vec3,
) {
    let pixel_center = vec2(launch_id.x as f32, launch_id.y as f32) + vec2(0.5, 0.5);
    let in_uv = pixel_center / vec2(launch_size.x as f32, launch_size.y as f32);
    let d = in_uv * 2.0 - 1.0;

    let origin = cam.view_inverse * vec4(0.0, 0.0, 0.0, 1.0);
    let target = cam.proj_inverse * vec4(d.x, d.y, 1.0, 1.0);
    let normalized_target = target.xyz().normalize();
    let direction = cam.view_inverse
        * vec4(
            normalized_target.x,
            normalized_target.y,
            normalized_target.z,
            0.0,
        );

    let tmin = 0.001;
    let tmax = 10000.0;

    *hit_value = vec3(0.0, 0.0, 0.0);

    unsafe {
        top_level_as.trace_ray(
            RayFlags::OPAQUE,
            0xff,
            0,
            0,
            0,
            origin.xyz(),
            tmin,
            direction.xyz(),
            tmax,
            hit_value,
        );
    }

    unsafe {
        image.write(
            spirv_std::glam::IVec2::new(launch_id.x, launch_id.y),
            vec4(hit_value.x, hit_value.y, hit_value.z, 0.0),
        );
    }
}

#[spirv(closest_hit)]
pub fn main_rchit(
    #[spirv(hit_attribute)] attribs: &Vec2,
    #[spirv(incoming_ray_payload)] hit_value: &mut Vec3,
) {
    let barycentric_coords = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);
    *hit_value = barycentric_coords;
}

#[spirv(miss)]
pub fn main_rmiss(#[spirv(incoming_ray_payload)] hit_value: &mut Vec3) {
    *hit_value = vec3(0.0, 0.0, 0.2);
}
