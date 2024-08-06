use vexide::prelude::*;

struct Robot;

impl Compete for Robot {
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

#[vexide::main]
async fn main(_p: Peripherals) {
    Robot.compete().await;
}
