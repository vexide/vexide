#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use core::str;

use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut link = RadioLink::open(peripherals.port_21, "643A", LinkType::Manager).unwrap();

    println!("[MANAGER] Waiting for worker radio...");

    while !link.is_linked() {
        sleep(RadioLink::UPDATE_INTERVAL).await;
    }

    println!("[MANAGER] Found worker - link established.");

    link.write(b"Hi from manager :3").unwrap();

    loop {
        if link.unread_bytes().unwrap() > 0 {
            let mut read = vec![0; link.unread_bytes().unwrap()];
            link.read(&mut read).unwrap();
            println!("[WORKER] {}", str::from_utf8(&read).unwrap());
        }
        sleep(RadioLink::UPDATE_INTERVAL).await;
    }
}
