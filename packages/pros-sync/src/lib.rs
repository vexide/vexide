//! Synchronous robot code trait for [pros-rs](https://crates.io/crates/pros).

#![no_std]

use pros_core::error::Result;

/// A trait for robot code that runs without the async executor spun up.
/// This trait isn't recommended. See `AsyncRobot` in [pros-async](https://crates.io/crates/pros-async) for the preferred trait to run robot code.
pub trait SyncRobot {
    /// Runs during the operator control period.
    /// This function may be called more than once.
    /// For that reason, do not use `Peripherals::take` in this function.
    fn opcontrol(&mut self) -> Result {
        Ok(())
    }
    /// Runs during the autonomous period.
    fn auto(&mut self) -> Result {
        Ok(())
    }
    /// Runs continuously during the disabled period.
    fn disabled(&mut self) -> Result {
        Ok(())
    }
    /// Runs once when the competition system is initialized.
    fn comp_init(&mut self) -> Result {
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gen_sync_exports {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::SyncRobot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::SyncRobot>::auto(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::SyncRobot>::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::SyncRobot>::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }
    };
}

/// Allows your sync robot code to be executed by the pros kernel.
/// If your robot struct implements Default then you can just supply this macro with its type.
/// If not, you can supply an expression that returns your robot type to initialize your robot struct.
/// The code that runs to create your robot struct will run in the initialize function in PROS.
///
/// Example of using the macro with a struct that implements Default:
/// ```rust
/// use pros::prelude::*;
/// #[derive(Default)]
/// struct ExampleRobot;
/// impl SyncRobot for ExampleRobot {
///    asnyc fn opcontrol(&mut self) -> pros::Result {
///       println!("Hello, world!");
///      Ok(())
///   }
/// }
/// sync_robot!(ExampleRobot);
/// ```
///
/// Example of using the macro with a struct that does not implement Default:
/// ```rust
/// use pros::prelude::*;
/// struct ExampleRobot {
///    x: i32,
/// }
/// impl SyncRobot for ExampleRobot {
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
/// sync_robot!(ExampleRobot, ExampleRobot::new());
#[macro_export]
macro_rules! sync_robot {
    ($rbt:ty) => {
        $crate::__gen_sync_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            let robot = Default::default();
            unsafe {
                ROBOT = Some(robot);
            }
        }
    };
    ($rbt:ty, $init:expr) => {
        $crate::__gen_sync_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            let robot = $init;
            unsafe {
                ROBOT = Some(robot);
            }
        }
    };
}
