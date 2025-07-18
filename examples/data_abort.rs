#![no_main]
#![no_std]

use core::{arch::asm, ptr::dangling_mut, time::Duration};

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    unsafe {
        vexide::vectors::install_vector_table();

        sleep(Duration::from_secs(1)).await;

        println!("Hello world");

        // Induce a CPU data abort exception
        let p = dangling_mut::<u32>();
        core::ptr::write_volatile(p, 123);
    }
}
