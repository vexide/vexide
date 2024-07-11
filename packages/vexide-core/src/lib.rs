//! Low level core functionality for [`vexide`](https://crates.io/crates/vexide).
//! The core crate is used in all other crates in the vexide ecosystem.
//!
//! Included in this crate:
//! - Global allocator: [`allocator`]
//! - Competition state handling: [`competition`]
//! - Critical-section implementation: [`critical_section`]
//! - Serial terminal printing: [`io`]
//! - No-std [`Instant`](time::Instant)s: [`time`]
//! - Synchronization primitives: [`sync`]
//! - Program control: [`program`]

#![no_std]
#![feature(error_in_core, never_type)]
#![feature(asm_experimental_arch)]

pub mod allocator;
pub mod competition;
pub mod critical_section;
pub mod float;
pub mod io;
pub mod program;
pub mod sync;
pub mod time;
pub mod fs;
