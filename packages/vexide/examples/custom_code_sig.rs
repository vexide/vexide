#![no_main]
#![no_std]

use vexide::{prelude::*, startup::ColdHeader};

// The custom code signature can be used to configure program behavior.
static CODE_SIG: ColdHeader = ColdHeader::new(2, 0, 0);

#[vexide::main(code_sig = CODE_SIG)]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world!");
}
