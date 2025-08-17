#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{glam::UVec3, spirv};

fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        return n;
    }
    let mut curr = 1;
    let mut prev = 1;
    for i in 2..n {
        let temp = curr;
        curr += prev;
        prev = temp;
    }
    curr
}

#[spirv(compute(threads(1, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] values: &mut [u32],
    #[spirv(spec_constant(id = 0))] buffer_elements: u32,
) {
    let index = global_id.x;
    if index >= buffer_elements {
        return;
    }

    values[index as usize] = fibonacci(values[index as usize]);
}
