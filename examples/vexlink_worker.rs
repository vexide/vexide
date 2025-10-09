use std::{vec, str};
use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut link = RadioLink::open(peripherals.port_21, "643A", LinkType::Worker);

    println!("[WORKER] Waiting for manager radio...");

    while !link.is_linked() {
        sleep(RadioLink::UPDATE_INTERVAL).await;
    }

    println!("[WORKER] Found manager - link established.");

    link.write(b"Hi from worker :3").unwrap();

    loop {
        if link.unread_bytes().unwrap() > 0 {
            let mut read = vec![0; link.unread_bytes().unwrap()];
            link.read(&mut read).unwrap();
            println!("[MANAGER] {}", str::from_utf8(&read).unwrap());
        }
        sleep(RadioLink::UPDATE_INTERVAL).await;
    }
}
