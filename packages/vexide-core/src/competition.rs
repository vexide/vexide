//! Competition control and state.
//!
//! This module provides functionality for interacting with competition control in VEXos,
//! allowing you to read competition state and respond to changes in competition modes.
//! This is useful for situations where the robot's behavior must adapt based on the
//! state of the competition, such as transitioning between autonomous and driver-controlled
//! modes.
//!
//! # The [`Compete`] Trait
//!
//! The most important item in this module is the [`Compete`] trait, which serves as the
//! foundation for defining competition-specific behavior in vexide programs. This trait
//! allows you to declare different functions on a robot struct to be executed when
//! competition state changes. By implementing [`Compete`], your robot can respond to
//! changes in the various phases of a competition, such as autonomous operation, driver
//! control, or downtime between modes.
//!
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use vexide::prelude::*;
//!
//! struct MyRobot {}
//!
//! impl Compete for MyRobot {
//!     async fn autonomous(&mut self) {
//!         println!("Running in autonomous mode!");
//!     }
//!
//!     async fn driver(&mut self) {
//!         println!("Running in driver mode!");
//!     }
//! }
//!
//! #[vexide::main]
//! async fn main(_peripherals: Peripherals) {
//!     let my_robot = MyRobot {};
//!     my_robot.compete().await;
//! }
//! ```
//!
//! By awaiting the [`compete()`] function on our robot, we are handing over execution to
//! vexide's [`CompetitionRuntime`], which will run a different function on the [`Compete`]
//! trait depending on what is happening in the match.
//!
//! [`compete()`]: CompeteExt::compete
//!
//! # Reading Competition State
//!
//! In addition to providing hooks into different competition modes, this module also provides
//! functions for reading information about the competition environment, such as the current match
//! mode, match control hardware, and whether the robot is enabled or disabled. This is provided
//! by the [`is_connected`], [`system`], [`mode`], and [`status`] functions.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    future::{Future, IntoFuture},
    marker::{PhantomData, PhantomPinned},
    ops::ControlFlow,
    pin::{pin, Pin},
    task::{self, Poll},
};

use bitflags::bitflags;
use futures_core::Stream;
use pin_project::pin_project;
use vex_sdk::vexCompetitionStatus;

bitflags! {
    /// The raw status bits returned by [`vex_sdk::vexCompetitionStatus`].
    ///
    /// These flags contain all the data made available to user code by VEXos
    /// about the state of the match.
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct CompetitionStatus: u32 {
        /// Robot is disabled by field control.
        const DISABLED = 1 << 0;

        /// Robot is in autonomous mode.
        const AUTONOMOUS = 1 << 1;

        /// Robot is connected to competition control (either competition switch or field control).
        const CONNECTED = 1 << 2;

        /// Robot is connected to field control (NOT competition switch)
        const SYSTEM = 1 << 3;
    }
}

/// A match mode in the competition lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionMode {
    /// The Disabled competition mode.
    ///
    /// When in disabled mode, voltage commands to motors are disabled. Motors are forcibly
    /// locked to the "coast" brake mode and cannot be moved.
    ///
    /// Robots may be placed into disabled mode at any point in the competition after
    /// connecting, but are typically disabled before the autonomous period, between
    /// autonomous and opcontrol periods, and following the opcontrol period of a match.
    Disabled,

    /// The Autonomous competition mode.
    ///
    /// When in autonomous mode, all motors and sensors may be accessed, however user
    /// input from controller buttons and joysticks is not available to be read.
    ///
    /// Robots may be placed into autonomous mode at any point in the competition after
    /// connecting, but are typically placed into this mode at the start of a match.
    Autonomous,

    /// The Driver Control (i.e. opcontrol) competition mode.
    ///
    /// When in opcontrol mode, all device access is available including access to
    /// controller joystick values for reading user-input from drive team members.
    ///
    /// Robots may be placed into opcontrol mode at any point in the competition after
    /// connecting, but are typically placed into this mode following the autonomous
    /// period.
    Driver,
}

/// A type of system used to control match state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionSystem {
    /// Competition state is controlled by a VEX Field Controller (either [legacy] or [smart] field control).
    ///
    /// [legacy]: https://www.vexrobotics.com/275-1401.html
    /// [smart]: https://kb.vex.com/hc/en-us/articles/9121731684756-VEX-Field-Control-User-Manual
    FieldControl,

    /// Competition state is controlled by a [VEXnet competition switch].
    ///
    /// [VEXnet competition switch]: https://www.vexrobotics.com/276-2335.html
    CompetitionSwitch,
}

impl CompetitionStatus {
    /// Checks if the robot is connected to a competition control system.
    ///
    /// This is equivalent to the standalone [`is_connected`] function in this module.
    ///
    /// # Example
    ///
    /// ```
    /// let status = competition::status();
    ///
    /// if status.is_connected() {
    ///     println!("Connected to competition control");
    /// }
    /// ```
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        self.contains(CompetitionStatus::CONNECTED)
    }

    /// Returns the current competition mode, or phase from these status flags.
    ///
    /// This is equivalent to the standalone [`mode`] function in this module.
    ///
    /// # Example
    ///
    /// ```
    /// let status = competition::status();
    ///
    /// match status.mode() {
    ///     CompetitionMode::Driver => println!("Driver control"),
    ///     CompetitionMode::Autonomous => println!("Auton"),
    ///     CompetitionMode::Disabled => println!("DIsabled"),
    /// }
    /// ```
    #[must_use]
    pub const fn mode(&self) -> CompetitionMode {
        if self.contains(Self::DISABLED) {
            CompetitionMode::Disabled
        } else if self.contains(Self::AUTONOMOUS) {
            CompetitionMode::Autonomous
        } else {
            CompetitionMode::Driver
        }
    }

    /// Returns the type of system currently controlling the robot's competition state, or [`None`] if the robot
    /// is not tethered to a competition controller.
    ///
    /// This is equivalent to the standalone [`system`] function in this module.
    ///
    /// # Example
    ///
    /// ```
    /// let status = competition::status();
    ///
    /// match status.system() {
    ///     None => println!("Not connected to a match controller"),
    ///     Some(CompetitionSystem::FieldControl) => println!("Connected to field controller"),
    ///     Some(CompetitionSystem::CompetitionSwitch) => println!("Connected to competition switch"),
    /// }
    /// ```
    #[must_use]
    pub const fn system(&self) -> Option<CompetitionSystem> {
        if self.contains(CompetitionStatus::CONNECTED) {
            if self.contains(Self::SYSTEM) {
                Some(CompetitionSystem::FieldControl)
            } else {
                Some(CompetitionSystem::CompetitionSwitch)
            }
        } else {
            None
        }
    }
}

/// Returns all competition status flags reported by VEXos.
///
/// See [`CompetitionStatus`] for usage and examples.
#[must_use]
pub fn status() -> CompetitionStatus {
    CompetitionStatus::from_bits_retain(unsafe { vexCompetitionStatus() })
}

/// Checks if the robot is connected to a competition control system.
///
/// # Example
///
/// ```
/// if competition::is_connected() {
///     println!("Connected to competition control");
/// }
/// ```
#[must_use]
pub fn is_connected() -> bool {
    status().is_connected()
}

/// Returns the type of system currently controlling the robot's competition state, or [`None`] if the robot
/// is not tethered to a competition controller.
///
/// # Example
///
/// ```
/// match competition::system() {
///     None => println!("Not connected to a match controller"),
///     Some(CompetitionSystem::FieldControl) => println!("Connected to field controller"),
///     Some(CompetitionSystem::CompetitionSwitch) => println!("Connected to competition switch"),
/// }
/// ```
#[must_use]
pub fn system() -> Option<CompetitionSystem> {
    status().system()
}

/// Returns the current competition mode, or phase.
///
/// ```
/// let status = competition::status();
///
/// match competition::mode() {
///     CompetitionMode::Driver => println!("Driver control"),
///     CompetitionMode::Autonomous => println!("Auton"),
///     CompetitionMode::Disabled => println!("DIsabled"),
/// }
/// ```
#[must_use]
pub fn mode() -> CompetitionMode {
    status().mode()
}

/// A stream of updates to the competition status.
///
/// See [`updates`] for more information.
pub struct CompetitionUpdates {
    last_status: Option<CompetitionStatus>,
}

impl Stream for CompetitionUpdates {
    type Item = CompetitionStatus;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let current = status();

        // TODO: This should probably be done on a timer in the reactor.
        cx.waker().wake_by_ref();

        if self.last_status == Some(current) {
            Poll::Pending
        } else {
            self.get_mut().last_status = Some(current);
            Poll::Ready(Some(current))
        }
    }
}

impl CompetitionUpdates {
    /// Returns the last status update.
    ///
    /// This is slightly more efficient than calling [`status`] as it does not require another poll,
    /// however, it can be out of date if the stream has not been polled recently.
    pub fn last(&self) -> CompetitionStatus {
        self.last_status.unwrap_or_else(status)
    }
}

/// Returns an async stream of updates to the competition status.
///
/// Yields the current status when first polled, and thereafter whenever the status changes.
#[must_use]
pub const fn updates() -> CompetitionUpdates {
    CompetitionUpdates { last_status: None }
}

/// A future which delegates to different futures depending on the current competition mode.
/// I.e., a tiny async runtime specifically for writing competition programs.
///
/// This runtime provides the internal implementation behind the [`Compete`] trait and the
/// [`CompetitionBuilder`] struct.
#[pin_project]
pub struct CompetitionRuntime<
    Shared: 'static,
    Return,
    MkConnected,
    MkDisconnected,
    MkDisabled,
    MkAutonomous,
    MkDriver,
> where
    MkConnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisconnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisabled:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkAutonomous:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDriver:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
{
    // Functions to generate tasks for each mode.
    mk_connected: MkConnected,
    mk_disconnected: MkDisconnected,
    mk_disabled: MkDisabled,
    mk_autonomous: MkAutonomous,
    mk_driver: MkDriver,

    /// A stream of updates to the competition status.
    #[pin]
    updates: CompetitionUpdates,

    /// The current status bits of the competition.
    status: CompetitionStatus,

    /// The current phase of the competition runtime.
    phase: CompetitionRuntimePhase,

    /// The task currently running, or [`None`] if no task is running.
    ///
    /// SAFETY:
    /// - The `'static` lifetime is a lie to the compiler, it actually borrows `self.shared`.
    ///   Therefore, tasks MUST NOT move their `&'static mut` references, or else they will
    ///   still be around when we call a `mk_*` function with a new mutable reference to it.
    ///   We rely on lifetime parametricity of the `mk_*` functions for this (see the HRTBs above).
    /// - This field MUST come before `shared`, as struct fields are dropped in declaration order.
    #[allow(clippy::type_complexity)]
    task: Option<Pin<Box<dyn Future<Output = ControlFlow<Return>> + 'static>>>,

    /// A cell containing the data shared between all tasks.
    ///
    /// SAFETY: This field MUST NOT be mutated while a task is running, as tasks may still hold
    ///         references to it. This is enforced to owners of this struct by the `Pin`,
    ///         but we have no such guardrails, as we cannot project the pin to it without
    ///         creating an invalid `Pin<&mut Shared>` before possibly (legally) moving it
    ///         during task creation.
    shared: UnsafeCell<Shared>,

    /// Keep `self.shared` in place while `self.task` references it.
    _pin: PhantomPinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompetitionRuntimePhase {
    Initial,
    Disconnected,
    Connected,
    Mode(CompetitionMode),
}

impl<Shared, Return, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver> Future
    for CompetitionRuntime<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    >
where
    MkConnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisconnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisabled:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkAutonomous:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDriver:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
{
    type Output = Return;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let old_phase = *this.phase;

        // Poll any updates to competition status.
        match this.updates.as_mut().poll_next(cx) {
            Poll::Ready(Some(new_status)) => {
                let old_status = *this.status;

                // Connected and Disconnected should not be interrupted by other changes.
                if *this.phase != CompetitionRuntimePhase::Connected
                    && *this.phase != CompetitionRuntimePhase::Disconnected
                {
                    // Decide which phase we're in based on the status update.
                    *this.phase = if !old_status.is_connected() && new_status.is_connected() {
                        CompetitionRuntimePhase::Connected
                    } else if old_status.is_connected() && !new_status.is_connected() {
                        CompetitionRuntimePhase::Disconnected
                    } else {
                        CompetitionRuntimePhase::Mode(new_status.mode())
                    };
                }

                *this.status = new_status;
            }
            Poll::Ready(None) => unreachable!(),
            _ => {}
        }

        if let Some(Poll::Ready(res)) = this.task.as_mut().map(|task| task.as_mut().poll(cx)) {
            // If a task says to break out of the competition lifecycle, then we pass the return value up.
            if let ControlFlow::Break(val) = res {
                return Poll::Ready(val);
            }

            // Reset the current task to nothing, since we're done with the previous task.
            *this.task = None;

            match *this.phase {
                // Transition into the current mode if we previously ran the connected/disconnected task and it completed.
                CompetitionRuntimePhase::Connected | CompetitionRuntimePhase::Disconnected => {
                    *this.phase = CompetitionRuntimePhase::Mode(this.updates.last().mode());
                }
                _ => {}
            }
        }

        // We're now in a different competition phase, so we need to start a new task.
        if old_phase != *this.phase {
            // SAFETY: Before we make a new `&mut Shared`, we ensure that the existing task is dropped.
            //         Note that although this would not normally ensure that the reference is dropped,
            //         because the task could move it elsewhere, this is not the case here, because
            //         the task generator functions (and therefore their returned futures) are valid for
            //         any _arbitrarily small_ lifetime `'t`. Therefore, they are unable to move it elsewhere
            //         without proving that the reference will be destroyed before the task returns.
            drop(this.task.take());
            let shared = unsafe { &mut *this.shared.get() };

            // Create a new task based on the new competition phase.
            *this.task = match *this.phase {
                CompetitionRuntimePhase::Initial => None,
                CompetitionRuntimePhase::Disconnected => Some((this.mk_disconnected)(shared)),
                CompetitionRuntimePhase::Connected => Some((this.mk_connected)(shared)),
                CompetitionRuntimePhase::Mode(CompetitionMode::Disabled) => {
                    Some((this.mk_disabled)(shared))
                }
                CompetitionRuntimePhase::Mode(CompetitionMode::Autonomous) => {
                    Some((this.mk_autonomous)(shared))
                }
                CompetitionRuntimePhase::Mode(CompetitionMode::Driver) => {
                    Some((this.mk_driver)(shared))
                }
            };
        }

        Poll::Pending
    }
}

impl<Shared, Return>
    CompetitionRuntime<
        Shared,
        Return,
        DefaultMk<Shared, Return>,
        DefaultMk<Shared, Return>,
        DefaultMk<Shared, Return>,
        DefaultMk<Shared, Return>,
        DefaultMk<Shared, Return>,
    >
{
    /// Create a typed builder for a competition runtime with the given `shared` data.
    /// The default tasks simply do nothing, so you do not need to supply them if you don't want to.
    pub const fn builder(shared: Shared) -> CompetitionBuilder<Shared, Return> {
        fn default_mk<Shared, Return>(
            _: &mut Shared,
        ) -> Pin<Box<dyn Future<Output = ControlFlow<Return>>>> {
            Box::pin(async { ControlFlow::Continue(()) })
        }

        CompetitionBuilder {
            shared,
            mk_connected: default_mk,
            mk_disconnected: default_mk,
            mk_disabled: default_mk,
            mk_autonomous: default_mk,
            mk_driver: default_mk,
            _return: PhantomData,
        }
    }
}

type DefaultMk<Shared, Return> =
    for<'t> fn(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>;

/// A typed builder for [`CompetitionRuntime`] instances.
pub struct CompetitionBuilder<
    Shared,
    Return,
    MkConnected = DefaultMk<Shared, Return>,
    MkDisconnected = DefaultMk<Shared, Return>,
    MkDisabled = DefaultMk<Shared, Return>,
    MkAutonomous = DefaultMk<Shared, Return>,
    MkDriver = DefaultMk<Shared, Return>,
> {
    shared: Shared,

    mk_connected: MkConnected,
    mk_disconnected: MkDisconnected,
    mk_disabled: MkDisabled,
    mk_autonomous: MkAutonomous,
    mk_driver: MkDriver,

    // We're invariant in the return type.
    _return: PhantomData<fn(Return) -> Return>,
}

impl<Shared, Return, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
    CompetitionBuilder<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    >
where
    MkConnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisconnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisabled:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkAutonomous:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDriver:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
{
    /// Finish the builder, returning a [`CompetitionRuntime`] instance.
    pub fn finish(
        self,
    ) -> CompetitionRuntime<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    > {
        CompetitionRuntime {
            mk_connected: self.mk_connected,
            mk_disconnected: self.mk_disconnected,
            mk_disabled: self.mk_disabled,
            mk_autonomous: self.mk_autonomous,
            mk_driver: self.mk_driver,
            status: status(),
            updates: updates(),
            phase: CompetitionRuntimePhase::Initial,
            task: None,
            shared: UnsafeCell::new(self.shared),
            _pin: PhantomPinned,
        }
    }
}

impl<Shared: 'static, Return, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
    IntoFuture
    for CompetitionBuilder<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    >
where
    MkConnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisconnected:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDisabled:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkAutonomous:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
    MkDriver:
        for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 't>>,
{
    type Output = Return;

    type IntoFuture = CompetitionRuntime<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    >;

    fn into_future(self) -> Self::IntoFuture {
        self.finish()
    }
}

impl<Shared, Return, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
    CompetitionBuilder<
        Shared,
        Return,
        DefaultMk<Shared, Return>,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    >
{
    /// Use the given function to create a task that runs when the robot is connected to a competition system.
    /// This task will run until termination before any other tasks (disconnected, disabled, autonomous, driver) are run.
    pub fn on_connect<Mk>(
        self,
        mk_connected: Mk,
    ) -> CompetitionBuilder<Shared, Return, Mk, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
    where
        Mk: for<'s> FnMut(
            &'s mut Shared,
        ) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 's>>,
    {
        CompetitionBuilder {
            shared: self.shared,
            mk_connected,
            mk_disconnected: self.mk_disconnected,
            mk_disabled: self.mk_disabled,
            mk_autonomous: self.mk_autonomous,
            mk_driver: self.mk_driver,
            _return: self._return,
        }
    }
}

impl<Shared, Return, MkConnected, MkDisabled, MkAutonomous, MkDriver>
    CompetitionBuilder<
        Shared,
        Return,
        MkConnected,
        DefaultMk<Shared, Return>,
        MkDisabled,
        MkAutonomous,
        MkDriver,
    >
{
    /// Use the given function to create a task that runs when the robot is disconnected from a competition system.
    /// This task will run until termination before any other tasks (connected, disabled, autonomous, driver) are run.
    pub fn on_disconnect<Mk>(
        self,
        mk_disconnected: Mk,
    ) -> CompetitionBuilder<Shared, Return, MkConnected, Mk, MkDisabled, MkAutonomous, MkDriver>
    where
        Mk: for<'s> FnMut(
            &'s mut Shared,
        ) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 's>>,
    {
        CompetitionBuilder {
            shared: self.shared,
            mk_connected: self.mk_connected,
            mk_disconnected,
            mk_disabled: self.mk_disabled,
            mk_autonomous: self.mk_autonomous,
            mk_driver: self.mk_driver,
            _return: self._return,
        }
    }
}

impl<Shared, Return, MkConnected, MkDisconnected, MkAutonomous, MkDriver>
    CompetitionBuilder<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        DefaultMk<Shared, Return>,
        MkAutonomous,
        MkDriver,
    >
{
    /// Use the given function to create a task that runs while the robot is disabled.
    /// If the task terminates before the end of the disabled period, it will NOT be restarted.
    pub fn while_disabled<Mk>(
        self,
        mk_disabled: Mk,
    ) -> CompetitionBuilder<Shared, Return, MkConnected, MkDisconnected, Mk, MkAutonomous, MkDriver>
    where
        Mk: for<'s> FnMut(
            &'s mut Shared,
        ) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 's>>,
    {
        CompetitionBuilder {
            shared: self.shared,
            mk_connected: self.mk_connected,
            mk_disconnected: self.mk_disconnected,
            mk_disabled,
            mk_autonomous: self.mk_autonomous,
            mk_driver: self.mk_driver,
            _return: self._return,
        }
    }
}

impl<Shared, Return, MkConnected, MkDisconnected, MkDisabled, MkDriver>
    CompetitionBuilder<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        DefaultMk<Shared, Return>,
        MkDriver,
    >
{
    /// Use the given function to create a task that runs while the robot is autonomously controlled.
    /// If the task terminates before the end of the autonomous period, it will NOT be restarted.
    pub fn while_autonomous<Mk>(
        self,
        mk_autonomous: Mk,
    ) -> CompetitionBuilder<Shared, Return, MkConnected, MkDisconnected, MkDisabled, Mk, MkDriver>
    where
        Mk: for<'s> FnMut(
            &'s mut Shared,
        ) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 's>>,
    {
        CompetitionBuilder {
            shared: self.shared,
            mk_connected: self.mk_connected,
            mk_disconnected: self.mk_disconnected,
            mk_disabled: self.mk_disabled,
            mk_autonomous,
            mk_driver: self.mk_driver,
            _return: self._return,
        }
    }
}

impl<Shared, Return, MkConnected, MkDisconnected, MkDisabled, MkAutonomous>
    CompetitionBuilder<
        Shared,
        Return,
        MkConnected,
        MkDisconnected,
        MkDisabled,
        MkAutonomous,
        DefaultMk<Shared, Return>,
    >
{
    /// Use the given function to create a task that runs while the robot is driver controlled.
    /// If the task terminates before the end of the driver control period, it will NOT be restarted.
    pub fn while_driving<Mk>(
        self,
        mk_driver: Mk,
    ) -> CompetitionBuilder<Shared, Return, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, Mk>
    where
        Mk: for<'s> FnMut(
            &'s mut Shared,
        ) -> Pin<Box<dyn Future<Output = ControlFlow<Return>> + 's>>,
    {
        CompetitionBuilder {
            shared: self.shared,
            mk_connected: self.mk_connected,
            mk_disconnected: self.mk_disconnected,
            mk_disabled: self.mk_disabled,
            mk_autonomous: self.mk_autonomous,
            mk_driver,
            _return: self._return,
        }
    }
}

/// A set of functions to run when the competition is in a particular mode.
///
/// This trait allows you to declare different functions on a common robot struct to be
/// executed when competition state changes. By implementing `Compete`, your robot can
/// respond to changes in the various phases of a competition, such as autonomous operation,
/// driver control, or downtime between modes.
///
/// # Example
///
/// ```
/// #![no_std]
/// #![no_main]
///
/// use vexide::prelude::*;
///
/// struct MyRobot {}
///
/// impl Compete for MyRobot {
///     async fn autonomous(&mut self) {
///         println!("Running in autonomous mode!");
///     }
///
///     async fn driver(&mut self) {
///         println!("Running in driver mode!");
///     }
/// }
///
/// #[vexide::main]
/// async fn main(_peripherals: Peripherals) {
///     let my_robot = MyRobot {};
///     my_robot.compete().await;
/// }
/// ```
///
/// By awaiting the [`compete()`] function on our robot, we are handing over execution to
/// vexide's [`CompetitionRuntime`], which will run a different function on the [`Compete`]
/// trait depending on what is happening in the match.
///
/// [`compete()`]: CompeteExt::compete
#[allow(async_fn_in_trait, clippy::unused_async)]
pub trait Compete: Sized {
    /// Runs when the robot becomes connected into a competition controller.
    ///
    /// See [`CompetitionBuilder::on_connect`] for more information.
    async fn connected(&mut self) {}

    /// Runs when the robot disconnects from a competition controller.
    ///
    /// <section class="warning">
    ///
    /// This function does NOT run if connection to the match is lost due to
    /// a radio issue. It will only execute if the field control wire becomes
    /// physically disconnected from the controller (i.e.) from unplugging after
    /// a match ends.
    ///
    /// </section>
    ///
    /// See [`CompetitionBuilder::on_disconnect`] for more information.
    async fn disconnected(&mut self) {}

    /// Runs when the robot is disabled.
    ///
    /// When in disabled mode, voltage commands to motors are disabled. Motors are forcibly
    /// locked to the "coast" brake mode and cannot be moved.
    ///
    /// Robots may be placed into disabled mode at any point in the competition after
    /// connecting, but are typically disabled before the autonomous period, between
    /// autonomous and opcontrol periods, and following the opcontrol period of a match.
    async fn disabled(&mut self) {}

    /// Runs when the robot is put into autonomous mode.
    ///
    /// When in autonomous mode, all motors and sensors may be accessed, however user
    /// input from controller buttons and joysticks is not available to be read.
    ///
    /// Robots may be placed into autonomous mode at any point in the competition after
    /// connecting, but are typically placed into this mode at the start of a match.
    async fn autonomous(&mut self) {}

    /// Runs when the robot is put into driver control mode.
    ///
    /// When in opcontrol mode, all device access is available including access to
    /// controller joystick values for reading user-input from drive team members.
    ///
    /// Robots may be placed into opcontrol mode at any point in the competition after
    /// connecting, but are typically placed into this mode following the autonomous
    /// period.
    async fn driver(&mut self) {}
}

/// Extension methods for [`Compete`].
///
/// Automatically implemented for any type implementing [`Compete`].
#[allow(clippy::type_complexity)]
pub trait CompeteExt: Compete {
    /// Build a competition runtime that competes with this robot.
    fn compete(
        self,
    ) -> CompetitionRuntime<
        Self,
        !,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ControlFlow<!>> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ControlFlow<!>> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ControlFlow<!>> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ControlFlow<!>> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ControlFlow<!>> + 's>>,
    > {
        #[allow(clippy::unit_arg)]
        CompetitionRuntime::builder(self)
            .on_connect(|s| Box::pin(async { ControlFlow::Continue(s.connected().await) }))
            .on_disconnect(|s| Box::pin(async { ControlFlow::Continue(s.disconnected().await) }))
            .while_disabled(|s| Box::pin(async { ControlFlow::Continue(s.disabled().await) }))
            .while_autonomous(|s| Box::pin(async { ControlFlow::Continue(s.autonomous().await) }))
            .while_driving(|s| Box::pin(async { ControlFlow::Continue(s.driver().await) }))
            .finish()
    }
}

impl<R: Compete> CompeteExt for R {}
