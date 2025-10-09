use std::{cell::Cell, time::Duration};

use vexide::prelude::*;

vexide::task::task_local! {
    static COUNTER: Cell<u32> = Cell::new(1);
}

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    dbg!(&COUNTER);

    spawn(async {
        loop {
            println!("Spawned task count: {}", COUNTER.get());
            COUNTER.set(COUNTER.get() + 1);
            sleep(Duration::from_millis(100)).await;
        }
    })
    .detach();

    loop {
        println!("Main task count: {}", COUNTER.get());
        COUNTER.set(COUNTER.get() + 1);
        sleep(Duration::from_millis(100)).await;
    }
}
