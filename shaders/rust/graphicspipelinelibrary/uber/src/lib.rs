#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Vec3, Vec4},
    num_traits::Float,
    spirv,
};

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(flat)] _in_flat_normal: Vec3,
    #[spirv(spec_constant(id = 0, default = 0))] lighting_model: u32,
    out_frag_color: &mut Vec4,
) {
    let mut color = match lighting_model {
        0 => {
            // Phong
            let ambient = in_color * vec3(0.25, 0.25, 0.25);
            let n = in_normal.normalize();
            let l = in_light_vec.normalize();
            let v = in_view_vec.normalize();
            let r = (-l).reflect(n);
            let diffuse = n.dot(l).max(0.0) * in_color;
            let specular = r.dot(v).max(0.0).powf(32.0) * vec3(0.75, 0.75, 0.75);
            ambient + diffuse * 1.75 + specular
        }
        1 => {
            // Toon
            let n = in_normal.normalize();
            let l = in_light_vec.normalize();
            let intensity = n.dot(l);
            if intensity > 0.98 {
                in_color * 1.5
            } else if intensity > 0.9 {
                in_color * 1.0
            } else if intensity > 0.5 {
                in_color * 0.6
            } else if intensity > 0.25 {
                in_color * 0.4
            } else {
                in_color * 0.2
            }
        }
        2 => {
            // No shading
            in_color
        }
        3 => {
            // Greyscale
            let grey = in_color.x * 0.299 + in_color.y * 0.587 + in_color.z * 0.114;
            vec3(grey, grey, grey)
        }
        _ => in_color, // Default case
    };

    // Scene is dark, brighten up a bit
    color *= 1.25;

    *out_frag_color = vec4(color.x, color.y, color.z, 1.0);
}
