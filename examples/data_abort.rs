#![no_main]
#![no_std]

use core::{arch::asm, ptr::dangling_mut};

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    vexide::vectors::install_vector_table();

    // Induce a CPU data abort exception
    unsafe {
        let p = dangling_mut::<u32>();
        core::ptr::write_volatile(p, 123);
    }
}
