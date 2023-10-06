use core::ptr::NonNull;

use alloc::sync::Arc;
use concurrent_queue::ConcurrentQueue;
use futures::{task::ArcWake, Future};
use slab::Slab;
use spin::Once;

use crate::sync::Mutex;

use super::task::Task;

pub struct Executor {
    queue: Arc<ConcurrentQueue<Arc<TaskInternal>>>,

    returns: Arc<Mutex<Slab<Once<NonNull<()>>>>>,
}
impl !Sync for Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(ConcurrentQueue::unbounded()),
            returns: Arc::new(Mutex::new(Slab::new())),
        }
    }

    pub fn spawn<T: Send>(&self, future: impl Future<Output = T> + core::marker::Send) -> Task<T> {
        let return_key = self.returns.lock().insert(Once::new());
        let task = Arc::new(TaskInternal {
            future: &future as *const _ as _,

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
    future: *const (),

    queue: Arc<ConcurrentQueue<Arc<TaskInternal>>>,
    pub return_key: usize,
}
unsafe impl Send for TaskInternal {}
// TaskInternal can implement Sync because it is only modified by the executor, which isn't Sync.
unsafe impl Sync for TaskInternal {}
impl core::fmt::Debug for TaskInternal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskInternal")
            .field("future", &self.future)
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
