use core::{future::Future, task::Context};

use alloc::sync::Arc;
use concurrent_queue::ConcurrentQueue;
use futures::{future::BoxFuture, FutureExt, task::{ArcWake, waker_ref}};

use crate::sync::Mutex;

pub(crate) struct Executor {
    pub task_queue: Arc<ConcurrentQueue<Arc<Task>>>,
}
impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.task_queue.pop() {
            let mut future_slot = task.future.lock();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let cx = &mut Context::from_waker(&waker);
                if future.as_mut().poll(cx).is_pending() {
                    *future_slot = Some(future);
                }
            }   
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_queue: self.task_queue.clone(),
        });
        // Task queue is never closed, so this should only panic if we needed to anyway.
        self.task_queue.push(task).unwrap();
    }
}

pub(crate) struct Task {
    pub future: Mutex<Option<BoxFuture<'static, ()>>>,

    task_queue: Arc<ConcurrentQueue<Arc<Task>>>,
}
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
