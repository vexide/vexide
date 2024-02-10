//! Low level core functionality for [`pros-rs`](https://crates.io/crates/pros).
//! The core crate is used in all other crates in the pros-rs ecosystem.
//!
//! Included in this crate:
//! - Global allocator: [`pros_alloc`]
//! - Competition state checking: [`competition`]
//! - Errno handling: [`error`]
//! - Serial terminal printing: [`io`]
//! - No-std [`Instant`](time::Instant)s: [`time`]

#![no_std]
#![feature(error_in_core)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn,
    clippy::missing_const_for_fn
)]

extern crate alloc;

pub mod competition;
pub mod error;
pub mod io;
pub mod pros_alloc;
pub mod sync;
pub mod task;
pub mod time;
