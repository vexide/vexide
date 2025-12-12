use std::arch::asm;

use vexide::{
    debug::{StdioTransport, VexideDebugger},
    prelude::*,
    startup::{self, debugger::DEBUGGER},
};

#[vexide::main(banner(enabled = false))]
async fn main(_peripherals: Peripherals) {
    let stdio = StdioTransport::new();
    startup::debugger::install(VexideDebugger::new(stdio));

    let addr: usize = vexide_breakpoint as usize;
    println!("Setting breakpoint at: {addr:x?}");

    unsafe {
        let mut debugger = DEBUGGER.get().unwrap().lock().unwrap();
        debugger.register_breakpoint(addr, false).unwrap();
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
