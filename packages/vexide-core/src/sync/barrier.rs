use core::sync::atomic::AtomicUsize;

use futures_core::Future;

pub struct BarrierWaitFuture<'a> {
    leader: bool,
    barrier: &'a Barrier,
}
impl<'a> Future for BarrierWaitFuture<'a> {
    type Output = bool;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if critical_section::with(|_| {
            self.barrier
                .current
                .load(core::sync::atomic::Ordering::Acquire)
                == self.barrier.count
        }) {
            core::task::Poll::Ready(self.leader)
        } else {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}

pub struct Barrier {
    count: usize,
    current: AtomicUsize,
}
impl Barrier {
    pub const fn new(count: usize) -> Self {
        Self {
            count,
            current: AtomicUsize::new(0),
        }
    }

    pub fn wait(&self) -> BarrierWaitFuture<'_> {
        let leader = critical_section::with(|_| {
            self.current
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
                == 0
        });
        BarrierWaitFuture {
            leader,
            barrier: self,
        }
    }
}
