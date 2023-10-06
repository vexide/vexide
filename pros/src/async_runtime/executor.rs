use core::{any::Any, cell::UnsafeCell, future::Future, ptr::NonNull};

use alloc::sync::Arc;
use concurrent_queue::ConcurrentQueue;
use futures::{future::BoxFuture, task::ArcWake, FutureExt};
use slab::Slab;
use spin::Once;

use super::task::Task;

pub struct Executor {
    queue: Arc<ConcurrentQueue<Arc<TaskInternal>>>,

    returns: Arc<Slab<Once<NonNull<()>>>>,
}
impl !Sync for Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(ConcurrentQueue::unbounded()),
            returns: Arc::new(Slab::new()),
        }
    }

    pub fn spawn<T: Send>(&self, future: impl Future<Output = T> + core::marker::Send) -> Task<T> {
        let future = future.boxed();

        let task = Arc::new(TaskInternal {
            future: UnsafeCell::new(future),

            queue: self.queue.clone(),
            return_key: self.returns.insert(Once::new()),
        });
        self.queue.push(task);

        Task {
            returns: self.returns.clone(),
            return_key: task.return_key,
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
    future: UnsafeCell<BoxFuture<'static, dyn Any>>,

    queue: Arc<ConcurrentQueue<Arc<TaskInternal>>>,
    pub return_key: usize,
}
// TaskInternal can implement Sync because it is only modified by the executor, which isn't Sync.
unsafe impl Sync for TaskInternal {}

impl ArcWake for TaskInternal {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.queue.push(arc_self.clone());
    }
}
