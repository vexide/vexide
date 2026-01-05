use std::{convert::Infallible, time::Duration};

use v5_debugger::{debugger::V5Debugger, gdb_target::arch::configure_debug, transport::StdioTransport};
use vex_sdk::{vexSerialReadChar, vexTasksRun};
use vexide::prelude::*;

// #[inline(never)]
// fn fib(n: u64) -> u64 {
//     let mut a = 1;
//     let mut b = 0;
//     let mut count = 0;

//     while count < n {
//         let tmp = a + b;
//         b = a;
//         a = tmp;
//         count += 1;
//     }

//     b
// }

fn dbg_didr() -> u32 {
    unsafe {
        let didr: u32;
        core::arch::asm!(
            "mrc p14, 0, {didr}, c0, c0, 0",
            didr = out(reg) didr,
            options(nostack, preserves_flags)
        );
        didr
    }
}

fn dbg_drar() -> u32 {
    unsafe {
        let drar: u32;
        core::arch::asm!(
            "mrc p14, 0, {drar}, c1, c0, 0",
            drar = out(reg) drar,
            options(nostack, preserves_flags)
        );
        drar
    }
}

#[vexide::main(
    banner(
        enabled = false
    )
)]
async fn main(_peripherals: Peripherals) {
    let mut zp = zynq7000::Peripherals::take().unwrap();
    configure_debug(&mut zp.devcfg);

    // let n = fib(80);
    // println!("{n}");
}
