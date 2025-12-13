use std::{arch::asm, io::stdin, thread, time::Duration};

use vexide::{
    debug::{StdioTransport, VexideDebugger},
    prelude::*,
    startup::{self, debugger::DEBUGGER},
};

#[vexide::main(banner(enabled = false))]
async fn main(_peripherals: Peripherals) {
    let stdio = StdioTransport::new();
    startup::debugger::install(VexideDebugger::new(stdio));

    let addr: usize = add_nums as usize;
    // println!("Setting breakpoint at: {addr:x?}");

    unsafe {
        let mut debugger = DEBUGGER.get().unwrap().lock().unwrap();
        debugger.register_breakpoint(addr, false).unwrap();
    }

    println!("Calling a function...");
    add_nums(32, 108);
    println!("Calling #2");
    add_nums(100, 10);
    println!("Calling #3");
    add_nums(70, 40);
    println!("Back from that function!");
}

#[inline(never)]
#[unsafe(no_mangle)]
fn add_nums(left: u32, right: u32) -> u32 {
    left + right
}

// async fn readline() {
//     let mut buf = String::new();
//     loop {
//         let bytes_read = stdin().read_line(&mut buf).unwrap();
//         if bytes_read != 0 {
//             break;
//         }
//         let peek = u8::try_from(unsafe { vex_sdk::vexSerialPeekChar(1) })
//             .map(|c| c as char)
//             .ok();

//         println!("Waiting.... got: {buf:?}, peek: {peek:?}");
//         sleep(Duration::from_secs(1)).await;
//     }

//     println!("got: {buf}");
// }
