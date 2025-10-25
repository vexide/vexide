use std::{
    boxed::Box,
    sync::{Arc, LazyLock},
    vec::Vec,
};

use vexide::{
    prelude::*,
    sync::{Barrier, Mutex, RwLock},
};

// A lazily initialized static.
// Lazy locks don't need to be used on statics, but they can be.
static LAZY: LazyLock<Box<u32>> = LazyLock::new(|| Box::new(42));

#[vexide::main]
pub async fn main(_p: Peripherals) {
    // Mutexes allow sharing state between tasks by ensuring that only one task has access to the
    // data at a point in time.
    let state = Arc::new(Mutex::new(0));

    // Spawn two tasks. Each will share access to `state` using a Mutex guard.
    let t1 = spawn({
        let state = state.clone();
        async move {
            *state.lock().await += 1;
        }
    });

    let t2 = spawn({
        let state = state.clone();
        async move {
            *state.lock().await += 2;
        }
    });

    // Wait for both tasks to complete.
    t1.await;
    t2.await;

    println!("state is {}", state.lock().await);

    // Barriers are a tool for making a number of tasks reach the exact same point in execution
    // before continuing.

    // Create a barrier that will wait for 10 tasks to reach it.
    let barrier = Arc::new(Barrier::new(10));
    // Spawn 10 tasks that will all wait on the barrier.
    let mut handles = Vec::new();
    for _ in 0..10 {
        let barrier = barrier.clone();
        handles.push(spawn(async move {
            // All of "before wait" printlns will be executed before any of the "after wait"
            // printlns.
            println!("before wait");
            barrier.wait().await;
            println!("after wait");
        }));
    }

    // Cleanup the tasks by waiting for all of them to complete.
    for handle in handles {
        handle.await;
    }

    // RwLocks ensure that only one writer can access the data at a time, but multiple readers can
    // access it concurrently.

    // Create a new read write lock.
    let lock = RwLock::new(0u32);
    // Create two readers at the same time.
    // This is comparable to creating two immutable references.
    let reader1 = lock.read().await;
    let reader2 = lock.read().await;
    // Assert that the readers are equal.
    assert_eq!(*reader1, 0);
    assert_eq!(*reader2, 0);
    println!("readers are equal");

    // Drop both readers so that we can create a writer.
    // Remember, mutable access is exclusive!
    drop(reader1);
    drop(reader2);

    // Create a writer and write to the lock.
    // This is comparable to creating a mutable reference.
    let mut writer = lock.write().await;
    *writer = 1;
    // Drop the writer so that we can create a reader.
    drop(writer);

    // Create a reader and assert that the value is what we wrote.
    let reader3 = lock.read().await;
    assert_eq!(*reader3, 1);
    println!("writing works");

    // The value inside a LazyLock is initialized once get is called.
    // This is useful for initializing a value that is expensive to create.
    println!("Lazy lock: {}", *LAZY);
}
