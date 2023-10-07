use core::{cell::UnsafeCell, sync::atomic::AtomicPtr};

use alloc::{boxed::Box, sync::Arc};
use concurrent_queue::ConcurrentQueue;
use futures::{future::BoxFuture, task::ArcWake, Future, FutureExt};
use slab::Slab;
use spin::Once;

use crate::sync::Mutex;

use super::task::Task;

pub struct Executor {
    queue: Arc<ConcurrentQueue<Arc<TaskInternal>>>,

    returns: Arc<Mutex<Slab<Once<AtomicPtr<()>>>>>,
}
impl !Sync for Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(ConcurrentQueue::unbounded()),
            returns: Arc::new(Mutex::new(Slab::new())),
        }
    }

    pub fn spawn<T: Send>(&self, future: impl Future<Output = T> + core::marker::Send + 'static) -> Task<T> {
        let return_key = self.returns.lock().insert(Once::new());
        let future: BoxFuture<'static, AtomicPtr<()>> = Box::pin(future.map(|val| {
            let ptr = Box::into_raw(Box::new(val));
            AtomicPtr::new(ptr as _)
        }));
        
        let task = Arc::new(TaskInternal {
            future: UnsafeCell::new(future),

            queue: self.queue.clone(),
            return_key,
        });
        self.queue.push(task).unwrap();

        Task {
            returns: self.returns.clone(),
            return_key,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn tick(&self) {
        todo!()
    }

    pub fn run(&self) {
        todo!()
    }
}

struct TaskInternal {
    // Raw ptr to a BoxFuture, which is a trait object.
    future: UnsafeCell<BoxFuture<'static, AtomicPtr<()>>>,

    queue: Arc<ConcurrentQueue<Arc<TaskInternal>>>,
    pub return_key: usize,
}
// TaskInternal can implement Sync because it is only modified by the executor, which isn't Sync.
unsafe impl Sync for TaskInternal {}
impl core::fmt::Debug for TaskInternal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskInternal")
            .field("queue", &self.queue)
            .field("return_key", &self.return_key)
            .finish()
    }
}

impl ArcWake for TaskInternal {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.queue.push(arc_self.clone()).unwrap();
    }
}
