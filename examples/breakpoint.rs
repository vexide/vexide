use std::arch::asm;

use vexide::prelude::*;

#[vexide::main(banner(enabled = false))]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world");
    breakpoint();
    println!("Back from breakpoint!");
}

#[inline]
fn breakpoint() {
    unsafe {
        asm!("bkpt");
    }
}
