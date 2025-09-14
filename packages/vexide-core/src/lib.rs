//! Low-level common functionality for [`vexide`](https://crates.io/crates/vexide).
//!
//! This crate has historically served many purposes, but today provides a set
//! of common low-level APIs used in many of `vexide`'s crates for interfacing
//! with VEXos in various ways. Most of these modules are re-exported from the
//! top-level [`vexide`] crate.
//!
//! This crate includes:
//! - Competition control, including the [`Compete`](crate::competition::Compete)
//!   trait ([`competition`]).
//! - Backtrace collection ([`backtrace`]).
//! - OS version information ([`os`]).
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
pub mod sync;
pub mod time;
