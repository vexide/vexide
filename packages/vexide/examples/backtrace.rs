#![no_main]
#![no_std]

use vexide::{panic::backtrace::Backtrace, prelude::*};

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    foo();
}

fn foo() {
    bar();
}

fn bar() {
    baz();
}

fn baz() {
    let backtrace = Backtrace::try_capture().unwrap();
    println!("{backtrace}");
}
