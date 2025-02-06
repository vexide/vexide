#![no_main]
#![no_std]

use alloc::string::String;
use core::time::Duration;

use vexide::{
    core::fs::{metadata, read_dir, File},
    prelude::*,
};
extern crate alloc;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    let mut file = File::create("foo").unwrap();
    file.write_all(b"bar").unwrap();
    drop(file);

    let mut file = File::open("foo").unwrap();
    let mut buf = [0; 3];
    file.read(&mut buf).unwrap();
    let buf = String::from_utf8(buf.to_vec()).unwrap();
    println!("{buf}");

    if metadata("testfolder").is_ok_and(|metadata| metadata.is_dir()) {
        read_dir("testfolder").unwrap().for_each(|entry| {
            println!("{:?}", entry);
            block_on(sleep(Duration::from_millis(2)))
        });
    }
}
