//! Low-level common functionality in [`vexide`](https://crates.io/crates/vexide).
//!
//! This crate has historically served many purposes, but today provides a set
//! of common safe wrappers around various system APIs used in many of `vexide`'s
//! crates. Most of these modules are re-exported from the top-level [`vexide`]
//! crate.
//!
//! This crate includes:
//! - Competition control, including the [`Compete`](crate::competition::Compete)
//!   trait ([`competition`]).
//! - Backtrace collection ([`backtrace`]).
//! - OS version information ([`os`]).
//! - User program state ([`program`]).
//! - Extended system time APIs ([`time`]).
//!
//! This crate also implements vexide's [synchronization primitives](crate::sync),
//! which are executor agnostic and therefore not in `vexide-async`.

#![no_std]
#![feature(never_type)]

extern crate alloc;

pub mod backtrace;
pub mod competition;
pub mod os;
pub mod program;
pub mod time;
