//! Low level core functionality for [`vexide`](https://crates.io/crates/vexide).
//! The core crate is used in all other crates in the vexide ecosystem.
//!
//! Included in this crate:
//! - Global allocator: [`pros_alloc`]
//! - Errno handling: [`error`]
//! - Serial terminal printing: [`io`]
//! - No-std [`Instant`](time::Instant)s: [`time`]
//! - Synchronization primitives: [`sync`]
//! - FreeRTOS task management: [`task`]

#![no_std]
#![feature(error_in_core)]
#![cfg_attr(feature = "critical-section", feature(asm_experimental_arch))]

pub mod allocator;
#[cfg(feature = "critical-section")]
pub mod critical_section;
pub mod io;
pub mod sync;
pub mod time;
