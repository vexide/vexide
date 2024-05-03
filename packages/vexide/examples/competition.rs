#![no_main]
#![no_std]
#![feature(never_type)]

extern crate alloc;

use alloc::boxed::Box;

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    Competition::builder(())
        .on_connect(|_| {
            Box::pin(async {
                println!("Connected");
                Ok::<_, !>(())
            })
        })
        .on_disconnect(|_| {
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
        .while_driving(|_| {
            Box::pin(async {
                println!("Driver");
                Ok(())
            })
        })
        .while_autonomous(|_| {
            Box::pin(async {
                println!("Autonomous");
                Ok(())
            })
        })
        .await
        .unwrap();
}
