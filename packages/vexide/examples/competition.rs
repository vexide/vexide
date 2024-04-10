#![no_main]
#![no_std]
#![feature(never_type)]

extern crate alloc;

use alloc::boxed::Box;

use vexide::prelude::*;

#[vexide_startup::main]
async fn main(_peripherals: Peripherals) {
    Competition::builder(())
        .on_connected(|_| {
            Box::pin(async {
                println!("Connected");
                Ok::<_, !>(())
            })
        })
        .on_disconnected(|_| {
            Box::pin(async {
                println!("Disconnected");
                Ok(())
            })
        })
        .while_disabled(|_| {
            Box::pin(async {
                println!("Disabled");
                Ok(())
            })
        })
        .while_driver_controlled(|_| {
            Box::pin(async {
                println!("Driver");
                Ok(())
            })
        })
        .while_in_autonomous(|_| {
            Box::pin(async {
                println!("Autonomous");
                Ok(())
            })
        })
        .await
        .unwrap();
}
