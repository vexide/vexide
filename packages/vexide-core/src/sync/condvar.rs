use core::{
    panic,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
    task::Poll,
};

use futures_core::Future;
use replace_with::replace_with;

use super::{MutexGuard, MutexLockFuture};

pub enum CondVarWaitFuture<'a, T> {
    WaitingForNotification {
        condvar: &'a CondVar,
        gaurd: MutexGuard<'a, T>,
    },
    WaitingForMutex {
        gaurd: MutexLockFuture<'a, T>,
    },
}
impl<'a, T> Future for CondVarWaitFuture<'a, T> {
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
                Self::WaitingForNotification { condvar, mut gaurd } => {
                    let state = condvar.state.load(Ordering::Acquire);
                    critical_section::with(|_| match state {
                        CondVar::NOTIFIED_ONE => {
                            condvar.state.store(CondVar::WAITING, Ordering::Release);
                            condvar.waiting.fetch_sub(1, Ordering::AcqRel);
                            Self::WaitingForMutex {
                                gaurd: gaurd.relock(),
                            }
                        }
                        CondVar::NOTIFIED_ALL => {
                            let waiting = condvar.waiting.fetch_sub(1, Ordering::AcqRel);
                            if waiting == 1 {
                                condvar.state.store(CondVar::WAITING, Ordering::Release);
                            }
                            Self::WaitingForMutex {
                                gaurd: gaurd.relock(),
                            }
                        }
                        CondVar::WAITING => Self::WaitingForNotification { condvar, gaurd },
                        _ => unreachable!("Invalid state in CondVar::state"),
                    })
                }
                CondVarWaitFuture::WaitingForMutex { mut gaurd } => {
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

pub struct CondVar {
    state: AtomicU8,
    waiting: AtomicUsize,
}
impl CondVar {
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

    pub fn wait<'a, T>(&'a self, gaurd: MutexGuard<'a, T>) -> CondVarWaitFuture<'a, T> {
        // SAFETY: we can unlock the mutex because we gaurentee that it will not be used again until we safely lock it again.
        unsafe {
            gaurd.unlock();
        }
        critical_section::with(|_| self.waiting.fetch_add(1, Ordering::AcqRel));
        CondVarWaitFuture::WaitingForNotification {
            condvar: self,
            gaurd,
        }
    }

    pub fn notify_one(&self) {
        critical_section::with(|_| self.state.store(Self::NOTIFIED_ONE, Ordering::Release));
    }

    pub fn notify_all(&self) {
        critical_section::with(|_| self.state.store(Self::NOTIFIED_ALL, Ordering::Release));
    }
}
