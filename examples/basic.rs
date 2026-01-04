use std::{convert::Infallible, time::Duration};

use v5_debugger::{debugger::V5Debugger, transport::StdioTransport};
use vex_sdk::{vexSerialReadChar, vexTasksRun};
use vexide::prelude::*;

#[inline(never)]
fn fib(n: u64) -> u64 {
    let mut a = 1;
    let mut b = 0;
    let mut count = 0;

    while count < n {
        let tmp = a + b;
        b = a;
        a = tmp;
        count += 1;
    }

    b
}

#[vexide::main(
    banner(
        enabled = false
    )
)]
async fn main(_peripherals: Peripherals) {
    v5_debugger::install(V5Debugger::new(StdioTransport::new()));

    sleep(Duration::from_millis(500)).await;

    unsafe {
        core::arch::asm!("bkpt");
    }

    let n = fib(80);
    println!("{n}");
}
