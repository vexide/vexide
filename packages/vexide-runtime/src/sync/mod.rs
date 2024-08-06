//! Synchronization types for async tasks.
//!
//! Types implemented here are specifically designed to mimic the standard library.

mod barrier;
mod condvar;
mod lazy;
mod mutex;
mod once;
mod rwlock;

pub mod critical_section;

pub use barrier::{Barrier, BarrierWaitFuture};
pub use condvar::{Condvar, CondvarWaitFuture};
pub use lazy::LazyLock;
pub use mutex::{Mutex, MutexGuard, MutexLockFuture, RawMutex};
pub use once::{Once, OnceLock};
pub use rwlock::{RwLock, RwLockReadFuture, RwLockReadGuard, RwLockWriteFuture, RwLockWriteGuard};
