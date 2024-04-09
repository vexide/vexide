#![no_main]
#![no_std]

extern crate alloc;

use core::time::Duration;

use vexide::prelude::*;

struct Robot;

impl CompetitionRobot for Robot {
    async fn connected(&mut self) {
        println!("Connected");
    }
    async fn disconnected(&mut self) {
        println!("Disconnected");
    }
    async fn disabled(&mut self) {
        println!("Disabled");
    }
    async fn driver(&mut self) {
        println!("Driver");
    }
    async fn autonomous(&mut self) {
        println!("Autonomous");
    }
}

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    let robot = Robot;

    robot.compete().await;
}
