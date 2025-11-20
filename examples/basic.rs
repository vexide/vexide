use std::time::{Duration, Instant};

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    loop {
        sleep(Duration::from_millis(50)).await;
        println!("{:?}", Instant::now());
    }
}
