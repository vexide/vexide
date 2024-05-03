#![no_std]
#![no_main]

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use vexide::{core::sync::Barrier, prelude::*};
use vexide_core::sync::{Condvar, LazyLock, Mutex, RwLock};
extern crate alloc;

static LAZY: LazyLock<Box<u32>> = LazyLock::new(|| Box::new(42));

#[vexide::main]
pub async fn main(_p: Peripherals) {
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = Vec::new();
    for _ in 0..10 {
        let barrier = barrier.clone();
        handles.push(spawn(async move {
            println!("before wait");
            barrier.wait().await;
            println!("after wait");
        }));
    }

    for handle in handles {
        handle.await;
    }

    let lock = RwLock::new(0u32);
    let reader1 = lock.read().await;
    let reader2 = lock.read().await;
    assert_eq!(*reader1, 0);
    assert_eq!(*reader2, 0);
    println!("readers are equal");
    drop(reader1);
    drop(reader2);
    let mut writer = lock.write().await;
    *writer = 1;
    drop(writer);
    let reader3 = lock.read().await;
    assert_eq!(*reader3, 1);
    println!("writing works");

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    spawn(async move {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().await;
        *started = true;
        // Notify the condvar that the value has changed.
        cvar.notify_one();
    })
    .detach();

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().await;
    while !*started {
        started = cvar.wait(started).await;
    }
    println!("condvar works");

    println!("Lazy lock: {}", LAZY.get().await);
}
