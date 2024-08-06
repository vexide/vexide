use vexide::prelude::*;

use std::time::Duration;

#[vexide::main]
async fn main(_p: Peripherals) {
    // Async tasks can be spawned using the spawn function.
    // The spawn function returns a handle to the spawned task.
    let handle = spawn(async {
        println!("Hello from an async task!");
    });

    // The handle can be awaited to join the task.
    handle.await;

    // Handles can be "detached" which makes them run in the background forever.
    // There is no way to interact with a task handle once it has been detached.
    let handle = spawn(async {
        println!("Hello from a detached async task!");
    });
    handle.detach();

    // The sleep function can be used to pause the current task for a given duration.
    // This is the easiest way to pass execution to other tasks.
    sleep(Duration::from_secs(1)).await;

    // Tight loops must have sleeps for other tasks to be run.
    // This includes vital tasks such as the task that flushes serial and device comunication.
    // Remember, this is cooperative multitasking!
    spawn(async {
        loop {
            println!("Hello from a spawned task!");
            // Without this sleep, the main task will be run at most once and serial will stop being output.
            sleep(Duration::from_millis(1)).await;
        }
    })
    .detach();

    // If you are forced to make a tight loop in a sync context, you can use the block_on function.
    // This function will block until the future passed to it is completed.
    // This is less convenient and slower than using async/await, but it is still necessary in some occasions.
    loop {
        println!("Hello from the main task!");
        // Roughly equivalent to `sleep(Duration::from_millis(1)).await;`
        block_on(sleep(Duration::from_millis(1)));
    }
}
