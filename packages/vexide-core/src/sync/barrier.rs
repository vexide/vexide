use core::sync::atomic::AtomicUsize;

use futures_core::Future;

/// A future that resolves once all tasks have arrived at a [`Barrier`].
/// This is created by [`Barrier::wait`].
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

/// Allows for multiple tasks to reach the same point in execution before continuing.
///
/// # Examples
///
/// ```rust
/// const N: usize = 10;
/// let handles = Vec::new();
/// let barrier = Barrier::new(N);
/// for i in 0..N {
///     spawn(async {
///         // Every "Before barrier" will be printed before any "After Barrier".
///         println!("Before Barrier");
///         barrier.wait().await;
///         println!("After Barrier");
///     })
/// }
///
/// for handle in handles {
///     handle.await;
/// }
/// ```
pub struct Barrier {
    count: usize,
    current: AtomicUsize,
}
impl Barrier {
    /// Create a new barrier that will block `count` threads before releasing.
    pub const fn new(count: usize) -> Self {
        Self {
            count,
            current: AtomicUsize::new(0),
        }
    }

    /// Wait for the barrier to be reached by every thread.
    /// Currently [`Barrier`]s are not reusable.
    ///
    /// A single task will get a [`BarrierWaitFuture`] that resolves to true.
    /// This is the equivalent of the standard library method [`BarrierWaitResult::is_leader`](https://doc.rust-lang.org/std/sync/struct.BarrierWaitResult.html#method.is_leader)
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