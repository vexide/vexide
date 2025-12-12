use std::arch::asm;

use vexide::{prelude::*, startup::debug::bkpt::BREAKPOINTS};

#[vexide::main(banner(enabled = false))]
async fn main(_peripherals: Peripherals) {
    let addr: usize = vexide_breakpoint as usize;
    println!("Setting breakpoint at: {addr:x?}");

    unsafe {
        let mut mgr = BREAKPOINTS.lock().unwrap();
        mgr.register(addr, false).unwrap();
    }

    println!("Calling a function...");
    vexide_breakpoint();
    println!("Back from that function!");
}

#[inline(never)]
#[unsafe(no_mangle)]
fn vexide_breakpoint() {
    unsafe {
        asm!("nop", "nop");
    }
}
