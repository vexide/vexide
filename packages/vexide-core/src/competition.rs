//! Utilities for getting competition control state.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    future::Future,
    marker::PhantomPinned,
    pin::{pin, Pin},
    task::{self, Poll},
};

use bitflags::bitflags;
use futures_core::Stream;
use pin_project::pin_project;
use vex_sdk::vexCompetitionStatus;

bitflags! {
    /// The status bits returned by [`vex_sdk::vexCompetitionStatus`].
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

/// Represents a possible mode that robots can be set in during the competition lifecycle.
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

    /// The drivercontrol competition mode.
    ///
    /// When in opcontrol mode, all device access is available including access to
    /// controller joystick values for reading user-input from drive team members.
    ///
    /// Robots may be placed into opcontrol mode at any point in the competition after
    /// connecting, but are typically placed into this mode following the autonomous
    /// period.
    Driver,
}

/// Represents a type of system used to control competition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionSystem {
    /// Competition state is controlled by a VEX Field Controller.
    FieldControl,

    /// Competition state is controlled by a VEXnet competition switch.
    CompetitionSwitch,
}

impl CompetitionStatus {
    /// Checks if the robot is connected to a competition control system.
    pub fn connected(&self) -> bool {
        self.contains(CompetitionStatus::CONNECTED)
    }

    /// Gets the current competition mode, or phase from these status flags.
    pub fn mode(&self) -> CompetitionMode {
        if self.contains(Self::DISABLED) {
            CompetitionMode::Disabled
        } else if self.contains(Self::AUTONOMOUS) {
            CompetitionMode::Autonomous
        } else {
            CompetitionMode::Driver
        }
    }

    /// Gets the type of system currently controlling the robot's competition state, or [`None`] if the robot
    /// is not tethered to a competition controller.
    pub fn system(&self) -> Option<CompetitionSystem> {
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

/// Gets the current competition status flags.
pub fn status() -> CompetitionStatus {
    CompetitionStatus::from_bits_retain(unsafe { vexCompetitionStatus() })
}

/// Checks if the robot is connected to a competition control system.
pub fn connected() -> bool {
    status().connected()
}

/// Gets the type of system currently controlling the robot's competition state, or [`None`] if the robot
/// is not tethered to a competition controller.
pub fn system() -> Option<CompetitionSystem> {
    status().system()
}

/// Gets the current competition mode, or phase.
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

        if self.last_status != Some(current) {
            self.get_mut().last_status = Some(current);
            Poll::Ready(Some(current))
        } else {
            Poll::Pending
        }
    }
}

impl CompetitionUpdates {
    /// Get the last status update.
    ///
    /// This is slightly more efficient than calling [`status`] as it does not require another poll,
    /// however, it can be out of date if the stream has not been polled recently.
    pub fn last(&self) -> CompetitionStatus {
        self.last_status.unwrap_or_else(status)
    }
}

/// Gets a stream of updates to the competition status.
///
/// Yields the current status when first polled, and thereafter whenever the status changes.
pub fn updates() -> CompetitionUpdates {
    CompetitionUpdates { last_status: None }
}

/// A future which delegates to different futures depending on the current competition mode.
/// I.e., a tiny async runtime specifically for writing competition programs.
#[pin_project]
pub struct Competition<
    Shared: 'static,
    MkConnected,
    MkDisconnected,
    MkDisabled,
    MkAutonomous,
    MkDriver,
> where
    MkConnected: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDisconnected: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDisabled: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkAutonomous: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDriver: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
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
    task: Option<Pin<Box<dyn Future<Output = ()> + 'static>>>,

    /// A cell containing the data shared between all tasks.
    ///
    /// SAFETY: This field MUST NOT be mutated while a task is running, as tasks may still hold
    ///         references to it. This is enforced to owners of this struct by the `Pin`,
    ///         but we have no such guardrails, as we cannot project the pin to it without
    ///         creating an invalid `Pin<&mut Shared>` before possibly (legally) moving it
    ///         during task creation.
    shared: UnsafeCell<Shared>,

    /// Keep `self.current` in place while `self.current` references it.
    _pin: PhantomPinned,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompetitionRuntimePhase {
    NeverConnected,
    Disconnected,
    Connected,
    InMode(CompetitionMode),
}

impl CompetitionRuntimePhase {
    /// Process a status update, modifying the phase accordingly.
    pub fn process_status_update(&mut self, status: CompetitionStatus) {
        *self = match *self {
            Self::NeverConnected | Self::Disconnected
                if status.connected() =>
            {
                Self::Connected
            }
            Self::Connected | Self::InMode(_) if !status.connected() => {
                Self::Disconnected
            }
            Self::InMode(_) => Self::InMode(status.mode()),
            old => old,
        }
    }

    /// Finish the task corresponding to this phase, modifying the phase accordingly.
    pub fn finish_task(&mut self, status: CompetitionStatus) {
        *self = match *self {
            Self::Connected => Self::InMode(status.mode()),
            old => old,
        };
    }
}

impl<Shared, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver> Future
    for Competition<Shared, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
where
    MkConnected: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDisconnected: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDisabled: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkAutonomous: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDriver: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
{
    type Output = (); // TODO: switch to `!` when stable.

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let old_phase = *this.phase;

        match this.updates.as_mut().poll_next(cx) {
            Poll::Ready(Some(update)) => this.phase.process_status_update(update),
            Poll::Ready(None) => unreachable!(),
            _ => {}
        }

        if let Some(Poll::Ready(_)) = this.task.as_mut().map(|task| task.as_mut().poll(cx)) {
            *this.task = None;
            this.phase.finish_task(this.updates.last());
        }

        if old_phase != *this.phase {
            // SAFETY: Before we make a new `&mut Shared`, we ensure that the existing task is dropped.
            //         Note that although this would not normally ensure that the reference is dropped,
            //         because the task could move it elsewhere, this is not the case here, because
            //         the task generator functions (and therefore their returned futures) are valid for
            //         any _arbitrarily small_ lifetime `'t`. Therefore, they are unable to move it elsewhere
            //         without proving that the reference will be destroyed before the task returns.
            drop(this.task.take());
            let shared = unsafe { &mut *this.shared.get() };

            *this.task = match *this.phase {
                CompetitionRuntimePhase::NeverConnected => None,
                CompetitionRuntimePhase::Disconnected => Some((this.mk_disconnected)(shared)),
                CompetitionRuntimePhase::Connected => Some((this.mk_connected)(shared)),
                CompetitionRuntimePhase::InMode(CompetitionMode::Disabled) => {
                    Some((this.mk_disabled)(shared))
                }
                CompetitionRuntimePhase::InMode(CompetitionMode::Autonomous) => {
                    Some((this.mk_autonomous)(shared))
                }
                CompetitionRuntimePhase::InMode(CompetitionMode::Driver) => {
                    Some((this.mk_driver)(shared))
                }
            };
        }

        Poll::Pending
    }
}

impl<Shared, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
    Competition<Shared, MkConnected, MkDisconnected, MkDisabled, MkAutonomous, MkDriver>
where
    MkConnected: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDisconnected: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDisabled: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkAutonomous: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
    MkDriver: for<'t> FnMut(&'t mut Shared) -> Pin<Box<dyn Future<Output = ()> + 't>>,
{
    /// Create a new competition runtime from the shared state and raw task generator functions.
    /// This API is not recommended for most users, and [`CompetitionRobot::compete`] should be preferred when possible.
    pub fn new_raw(
        shared: Shared,
        mk_connected: MkConnected,
        mk_disconnected: MkDisconnected,
        mk_disabled: MkDisabled,
        mk_autonomous: MkAutonomous,
        mk_driver: MkDriver,
    ) -> Self {
        Self {
            mk_connected,
            mk_disconnected,
            mk_disabled,
            mk_autonomous,
            mk_driver,
            updates: updates(),
            phase: CompetitionRuntimePhase::NeverConnected,
            task: None,
            shared: UnsafeCell::new(shared),
            _pin: PhantomPinned,
        }
    }
}

/// A set of tasks to run when the competition is in a particular mode.
#[allow(async_fn_in_trait)]
pub trait CompetitionRobot: Sized {
    /// Runs when the competition system is connected.
    async fn connected(&mut self) {}
    /// Runs when the competition system is disconnected.
    async fn disconnected(&mut self) {}
    /// Runs when the robot is disabled.
    async fn disabled(&mut self) {}
    /// Runs when the robot is put into autonomous mode.
    async fn autonomous(&mut self) {}
    /// Runs when the robot is put into driver control mode.
    async fn driver(&mut self) {}
}

/// Extension methods for [`CompetitionRobot`].
/// Automatically implemented for any type implementing [`CompetitionRobot`].
pub trait CompetitionRobotExt: CompetitionRobot {
    /// Build a competition runtime that competes with this robot.
    fn compete(
        self,
    ) -> Competition<
        Self,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ()> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ()> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ()> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ()> + 's>>,
        impl for<'s> FnMut(&'s mut Self) -> Pin<Box<dyn Future<Output = ()> + 's>>,
    > {
        Competition::new_raw(
            self,
            |s| Box::pin(s.connected()),
            |s| Box::pin(s.disconnected()),
            |s| Box::pin(s.disabled()),
            |s| Box::pin(s.autonomous()),
            |s| Box::pin(s.driver()),
        )
    }
}

impl<R: CompetitionRobot> CompetitionRobotExt for R {}
