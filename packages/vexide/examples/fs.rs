#![no_main]
#![no_std]

use vexide::prelude::*;
use vexide_core::fs::path::Path;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    let path = Path::new("file.txt\0");
    let mut file = vexide_core::fs::File::create(path).unwrap();
    file.write_all(b"Hello, world!").unwrap();
    let mut buf = [0; 13];
    file.read(&mut buf).unwrap();
    println!("{buf:?}");
}
