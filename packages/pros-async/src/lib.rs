//! Tiny async runtime and robot traits for `pros-rs`.
//! The async executor supports spawning tasks and blocking on futures.
//! It has a reactor to improve the performance of some futures.
//! It is recommended to use the `AsyncRobot` trait to run robot code.
//! FreeRTOS tasks can still be used, but it is recommended to use only async tasks for performance.

#![no_std]
#![feature(negative_impls)]

extern crate alloc;

use core::{future::Future, task::Poll};

use async_task::Task;
use executor::EXECUTOR;
use pros_core::error::Result;

mod executor;
mod reactor;

/// Runs a future in the background without having to await it
/// To get the the return value you can await a task.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    executor::EXECUTOR.with(|e| e.spawn(future))
}

/// Blocks the current task untill a return value can be extracted from the provided future.
/// Does not poll all futures to completion.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    executor::EXECUTOR.with(|e| e.block_on(spawn(future)))
}

/// A future that will complete after the given duration.
/// Sleep futures that are closer to completion are prioritized to improve accuracy.
#[derive(Debug)]
pub struct SleepFuture {
    target_millis: u32,
}
impl Future for SleepFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.target_millis < unsafe { pros_sys::millis() } {
            Poll::Ready(())
        } else {
            EXECUTOR.with(|e| {
                e.reactor
                    .borrow_mut()
                    .sleepers
                    .push(cx.waker().clone(), self.target_millis)
            });
            Poll::Pending
        }
    }
}

/// Returns a future that will complete after the given duration.
pub fn sleep(duration: core::time::Duration) -> SleepFuture {
    SleepFuture {
        target_millis: unsafe { pros_sys::millis() + duration.as_millis() as u32 },
    }
}

/// A trait for robot code that spins up the pros-rs async executor.
/// This is the preferred trait to run robot code.
pub trait AsyncRobot {
    /// Runs during the operator control period.
    /// This function may be called more than once.
    /// For that reason, do not use `Peripherals::take`in this function.
    fn opcontrol(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
    /// Runs during the autonomous period.
    fn auto(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
    /// Runs continuously during the disabled period.
    fn disabled(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
    /// Runs once when the competition system is initialized.
    fn comp_init(&mut self) -> impl Future<Output = Result> {
        async { Ok(()) }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gen_async_exports {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn opcontrol() {
            $crate::block_on(<$rbt as $crate::AsyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            }))
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn autonomous() {
            $crate::block_on(<$rbt as $crate::AsyncRobot>::auto(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before auto")
            }))
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn disabled() {
            $crate::block_on(<$rbt as $crate::AsyncRobot>::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            }))
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn competition_initialize() {
            $crate::block_on(<$rbt as $crate::AsyncRobot>::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            }))
            .unwrap();
        }
    };
}

/// Allows your async robot code to be executed by the pros kernel.
/// If your robot struct implements Default then you can just supply this macro with its type.
/// If not, you can supply an expression that returns your robot type to initialize your robot struct.
/// The code that runs to create your robot struct will run in the initialize function in PROS.
///
/// Example of using the macro with a struct that implements Default:
/// ```rust
/// use pros::prelude::*;
/// #[derive(Default)]
/// struct ExampleRobot;
/// #[async_trait]
/// impl AsyncRobot for ExampleRobot {
///    asnyc fn opcontrol(&mut self) -> pros::Result {
///       println!("Hello, world!");
///      Ok(())
///   }
/// }
/// async_robot!(ExampleRobot);
/// ```
///
/// Example of using the macro with a struct that does not implement Default:
/// ```rust
/// use pros::prelude::*;
/// struct ExampleRobot {
///    x: i32,
/// }
/// #[async_trait]
/// impl AsyncRobot for ExampleRobot {
///     async fn opcontrol(&mut self) -> pros::Result {
///         println!("Hello, world! {}", self.x);
///         Ok(())
///     }
/// }
/// impl ExampleRobot {
///     pub fn new() -> Self {
///        Self { x: 5 }
///    }
/// }
/// async_robot!(ExampleRobot, ExampleRobot::new());
#[macro_export]
macro_rules! async_robot {
    ($rbt:ty) => {
        $crate::__gen_async_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            let robot = Default::default();
            unsafe {
                ROBOT = Some(robot);
            }
        }
    };
    ($rbt:ty, $init:expr) => {
        $crate::__gen_async_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            let robot = $init;
            unsafe {
                ROBOT = Some(robot);
            }
        }
    };
}
