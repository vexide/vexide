#![no_std]
#![no_main]

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;
#[async_trait]
impl pros::Robot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        println!("basic exasmple");

        Ok(())
    }
}
robot!(Robot);