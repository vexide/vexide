#![no_main]
#![no_std]
#![feature(never_type)]

extern crate alloc;

use core::time::Duration;

use vexide::prelude::*;

struct Robot;

impl CompetitionRobot for Robot {
    type Error = !;

    async fn connected(&mut self) -> Result<(), !> {
        println!("Connected");
        Ok(())
    }
    async fn disconnected(&mut self) -> Result<(), !> {
        println!("Disconnected");
        Ok(())
    }
    async fn disabled(&mut self) -> Result<(), !> {
        println!("Disabled");
        Ok(())
    }
    async fn driver(&mut self) -> Result<(), !> {
        println!("Driver");
        Ok(())
    }
    async fn autonomous(&mut self) -> Result<(), !> {
        println!("Autonomous");
        Ok(())
    }
}

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    Robot.compete().await;
}
