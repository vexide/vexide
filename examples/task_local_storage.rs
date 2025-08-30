use std::{cell::Cell, time::Duration};

use vexide::prelude::*;

vexide::task::task_local! {
    static COUNTER: Cell<u32> = Cell::new(10);
}

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    dbg!(&COUNTER);

    spawn(async {
        loop {
            println!("Task 1: {}", COUNTER.get());
            COUNTER.set(COUNTER.get() + 1);
            sleep(Duration::from_millis(50)).await;
        }
    })
    .detach();

    loop {
        println!("Main task: {}", COUNTER.get());
        COUNTER.set(COUNTER.get() + 1);
        sleep(Duration::from_millis(100)).await;
    }
}
