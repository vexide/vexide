use core::{cell::UnsafeCell, future::Future, task::Context};

use alloc::sync::Arc;
use concurrent_queue::ConcurrentQueue;
use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
    FutureExt,
};

pub(crate) struct Executor {
    pub task_queue: Arc<ConcurrentQueue<Arc<Task>>>,
    // This is here to make Executor Send but not Sync.
    _marker: core::marker::PhantomData<*const ()>,
}
impl Executor {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(ConcurrentQueue::unbounded()),
            _marker: core::marker::PhantomData,
        }
    }

    pub fn run(&self) {
        while let Ok(task) = self.task_queue.pop() {
            let future_slot = task.future.get();
            if let Some(mut future) = unsafe { (*future_slot).take() } {
                let waker = waker_ref(&task);
                let cx = &mut Context::from_waker(&waker);
                
                if future.as_mut().poll(cx).is_pending() {
                    unsafe { future_slot.write(Some(future)) }
                }
            }
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: UnsafeCell::new(Some(future)),
            task_queue: self.task_queue.clone(),
        });
        // Task queue is never closed, so this should only panic if we needed to anyway.
        self.task_queue.push(task).unwrap();
    }
}
impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct Task {
    future: UnsafeCell<Option<BoxFuture<'static, ()>>>,

    task_queue: Arc<ConcurrentQueue<Arc<Task>>>,
}
// Task can implement Sync because it is only modified by the executor, which isn't Sync.
unsafe impl Sync for Task {}

impl core::fmt::Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task").finish()
    }
}
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        // Again task_queue is never closed.
        cloned.task_queue.push(arc_self.clone()).unwrap();
    }
}
