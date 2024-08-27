use core::{
    fmt::Debug,
    panic,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
    task::Poll,
};

use futures_core::Future;
use replace_with::replace_with;

use super::{MutexGuard, MutexLockFuture};

/// A future that resolves once a condition variable is notified.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub enum CondvarWaitFuture<'a, T> {
    /// The future is waiting for a notification.
    WaitingForNotification {
        /// The condition variable to wait on.
        condvar: &'a Condvar,
        /// The mutex guard that was unlocked.
        gaurd: MutexGuard<'a, T>,
    },
    /// The future is waiting for a [`Mutex`](super::Mutex) to lock
    WaitingForMutex {
        /// The mutex lock future.
        gaurd: MutexLockFuture<'a, T>,
    },
}
impl<'a, T> Future for CondvarWaitFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut ret = None;
        replace_with(
            &mut *self,
            || panic!("Failed to replace"),
            |self_| match self_ {
                Self::WaitingForNotification { condvar, gaurd } => {
                    let state = condvar.state.load(Ordering::Acquire);
                    critical_section::with(|_| match state {
                        Condvar::NOTIFIED_ONE => {
                            condvar.state.store(Condvar::WAITING, Ordering::Release);
                            condvar.waiting.fetch_sub(1, Ordering::AcqRel);
                            Self::WaitingForMutex {
                                gaurd: gaurd.relock(),
                            }
                        }
                        Condvar::NOTIFIED_ALL => {
                            let waiting = condvar.waiting.fetch_sub(1, Ordering::AcqRel);
                            if waiting == 1 {
                                condvar.state.store(Condvar::WAITING, Ordering::Release);
                            }
                            Self::WaitingForMutex {
                                gaurd: gaurd.relock(),
                            }
                        }
                        Condvar::WAITING => Self::WaitingForNotification { condvar, gaurd },
                        _ => unreachable!("Invalid state in CondVar::state"),
                    })
                }
                CondvarWaitFuture::WaitingForMutex { mut gaurd } => {
                    match core::pin::pin!(&mut gaurd).poll(cx) {
                        Poll::Ready(lock) => {
                            ret = Some(lock);
                            Self::WaitingForMutex { gaurd }
                        }
                        Poll::Pending => Self::WaitingForMutex { gaurd },
                    }
                }
            },
        );
        match ret {
            None => {
                cx.waker().wake_by_ref();
                core::task::Poll::Pending
            }
            Some(lock) => core::task::Poll::Ready(lock),
        }
    }
}

/// A condition variable.
/// Condition variables allow for tasks to wait until a notification is received.
///
/// # Examples
/// ```rust
/// let pair = Arc::new((Mutex::new(false), Condvar::new()));
/// let pair2 = pair.clone();
///
/// spawn(async move {
///     let (lock, cvar) = &*pair2;
///     let mut started = lock.lock().await;
///     *started = true;
///     cvar.notify_one();
/// }).detach();
///
/// let (lock, cvar) = &*pair;
/// let mut started = lock.lock().await;
/// while !*started {
///     started = cvar.wait(started).await;
/// }
/// ```
pub struct Condvar {
    state: AtomicU8,
    waiting: AtomicUsize,
}
impl Condvar {
    const WAITING: u8 = 0;
    const NOTIFIED_ONE: u8 = 1;
    const NOTIFIED_ALL: u8 = 2;

    /// Creates a new condition variable.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(Self::WAITING),
            waiting: AtomicUsize::new(0),
        }
    }

    /// Waits for a notification on the condition variable.
    pub fn wait<'a, T>(&'a self, gaurd: MutexGuard<'a, T>) -> CondvarWaitFuture<'a, T> {
        // SAFETY: we can unlock the mutex because we gaurentee that it will not be used again until we safely lock it again.
        unsafe {
            gaurd.unlock();
        }
        critical_section::with(|_| self.waiting.fetch_add(1, Ordering::AcqRel));
        CondvarWaitFuture::WaitingForNotification {
            condvar: self,
            gaurd,
        }
    }

    /// Notify one task waiting on the condition variable.
    pub fn notify_one(&self) {
        critical_section::with(|_| self.state.store(Self::NOTIFIED_ONE, Ordering::Release));
    }

    /// Notify all tasks waiting on the condition variable.
    pub fn notify_all(&self) {
        critical_section::with(|_| self.state.store(Self::NOTIFIED_ALL, Ordering::Release));
    }
}
impl Default for Condvar {
    fn default() -> Self {
        Self::new()
    }
}
impl Debug for Condvar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Condvar").finish_non_exhaustive()
    }
}
