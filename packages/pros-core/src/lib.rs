//! Low level core functionality for [`pros-rs`](https://crates.io/crates/pros).
//! The core crate is used in all other crates in the pros-rs ecosystem.
//!
//! Included in this crate:
//! - Global allocator: [`pros_alloc`]
//! - Competition state checking: [`competition`]
//! - Errno handling: [`error`]
//! - Serial terminal printing: [`io`]
//! - No-std [`Instant`](time::Instant)s: [`time`]
//! - Synchronization primitives: [`sync`]
//! - FreeRTOS task management: [`task`]

#![no_std]
#![feature(error_in_core)]

extern crate alloc;

pub mod allocator;
pub mod error;
pub mod io;
pub mod sync;
pub mod task;
pub mod time;
