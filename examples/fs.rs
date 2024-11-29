#![no_main]
#![no_std]

use alloc::string::String;

use vexide::prelude::*;
extern crate alloc;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    let mut file = vexide::core::fs::File::create("foo").unwrap();
    file.write_all(b"bar").unwrap();
    file.flush().unwrap();
    let mut file = vexide::core::fs::File::open("foo").unwrap();
    let mut buf = [0; 3];
    file.read(&mut buf).unwrap();
    let buf = String::from_utf8(buf.to_vec()).unwrap();
    println!("{buf}");
}
