//! Low-level common functionality in [`vexide`](https://crates.io/crates/vexide).
//!
//! This crate has historically served many purposes, but today provides a set of common safe
//! wrappers around various system APIs used in some of `vexide`'s crates. Most of these modules are
//! re-exported from the top-level [`vexide`] crate.
//!
//! [`vexide`]: https://docs.rs/vexide/
//!
//! This crate includes:
//! - Competition control, including the [`Compete`](crate::competition::Compete) trait
//!   ([`competition`]).
//! - Backtrace collection ([`backtrace`]).
//! - OS version information ([`os`]).
//! - User program state ([`program`]).
//! - Extended system time APIs ([`time`]).

#![no_std]
#![feature(never_type)]

extern crate alloc;

pub mod backtrace;
pub mod competition;
pub mod os;
pub mod program;
pub mod time;
