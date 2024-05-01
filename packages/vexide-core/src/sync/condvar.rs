use core::{
    panic,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

use futures_core::Future;

use super::MutexGuard;

pub struct CondVarWaitFuture<'a, T> {
    condvar: &'a CondVar,
    gaurd: Option<MutexGuard<'a, T>>,
}
impl<'a, T> Future for CondVarWaitFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let state = self.condvar.state.load(Ordering::Acquire);
        match state {
            CondVar::NOTIFIED_ONE => {
                let Some(gaurd) = self.gaurd.take() else {
                    panic!("CondVarWaitFuture polled after completion");
                };
                self.condvar
                    .state
                    .store(CondVar::WAITING, Ordering::Release);
                core::task::Poll::Ready(gaurd)
            }
            CondVar::NOTIFIED_ALL => {
                let Some(gaurd) = self.gaurd.take() else {
                    panic!("CondVarWaitFuture polled after completion");
                };
                let waiting = self.condvar.waiting.fetch_sub(1, Ordering::AcqRel);
                if waiting == 1 {
                    self.condvar
                        .state
                        .store(CondVar::WAITING, Ordering::Release);
                }
                core::task::Poll::Ready(gaurd)
            }
            CondVar::WAITING => {
                cx.waker().wake_by_ref();
                core::task::Poll::Pending
            }
            _ => unreachable!("Invalid state in CondVar::state"),
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
        self.waiting.fetch_add(1, Ordering::AcqRel);
        CondVarWaitFuture {
            condvar: self,
            gaurd: Some(gaurd),
        }
    }
}
