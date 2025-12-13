use std::{arch::asm, io::stdin, thread, time::Duration};

use vexide::{
    debug::{StdioTransport, VexideDebugger},
    prelude::*,
    startup::{self, debugger::{DEBUGGER, breakpoint}},
};

#[vexide::main(banner(enabled = false))]
async fn main(_peripherals: Peripherals) {
    // readline().await;
    // return;

    let stdio = StdioTransport::new();
    startup::debugger::install(VexideDebugger::new(stdio));

    // std::panic::set_hook(Box::new(|_panic| {
    //     breakpoint();

    //     loop {
    //         thread::yield_now();
    //     }
    // }));

    let addr: usize = add_nums as usize;
    // println!("Setting breakpoint at: {addr:x?}");

    unsafe {
        let mut debugger = DEBUGGER.get().unwrap().lock().unwrap();
        debugger.register_breakpoint(addr, false).unwrap();
    }

    // println!("Calling a function...");
    add_nums(32, 108);
    // println!("Back from that function!");
}

#[inline(never)]
#[unsafe(no_mangle)]
fn add_nums(left: u32, right: u32) -> u32 {
    left + right
}


async fn readline() {
    let mut buf = String::new();
    loop {
        let bytes_read = stdin().read_line(&mut buf).unwrap();
        if bytes_read != 0 {
            break;
        }
        let peek = u8::try_from(unsafe { vex_sdk::vexSerialPeekChar(1) })
            .map(|c| c as char)
            .ok();

        println!("Waiting.... got: {buf:?}, peek: {peek:?}");
        sleep(Duration::from_secs(1)).await;
    }

    println!("got: {buf}");
}
