//! Low level core functionality for [`vexide`](https://crates.io/crates/vexide).
//! The core crate is used in all other crates in the vexide ecosystem.
//!
//! Included in this crate:
//! - Global allocator: [`allocator`]
//! - Competition state handling: [`competition`]
//! - Serial terminal printing: [`io`]
//! - No-std [`Instant`](time::Instant)s: [`time`]
//! - Synchronization primitives: [`sync`]
//! - Program control: [`program`]

#![no_std]
#![feature(never_type)]
#![feature(asm_experimental_arch)]

extern crate alloc;

pub mod allocator;
pub mod backtrace;
pub mod competition;
pub mod float;
pub mod io;
pub mod program;
pub mod sync;
pub mod time;
