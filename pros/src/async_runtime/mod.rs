use core::{future::Future, task::Context};

use alloc::{collections::VecDeque, sync::Arc};
use futures::{future::BoxFuture, FutureExt, task::{ArcWake, waker_ref}};
use spin::RwLock;

use crate::sync::Mutex;

pub(crate) struct Spawner {
    task_queue: Arc<RwLock<VecDeque<Arc<Task>>>>,
}
impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_queue: self.task_queue.clone(),
        });
        self.task_queue.write().push_back(task);
    }
}

pub(crate) struct Executor {
    pub task_queue: Arc<RwLock<VecDeque<Arc<Task>>>>,
}
impl Executor {
    pub fn run(&self) {
        while let Some(task) = self.task_queue.write().pop_front() {
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
}

pub(crate) struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    task_queue: Arc<RwLock<VecDeque<Arc<Task>>>>,
}
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        cloned.task_queue.write().push_back(arc_self.clone());
    }
}

pub(crate) fn new_executor_and_spawner() -> (Executor, Spawner) {
    let task_queue = Arc::new(RwLock::new(VecDeque::new()));
    (Executor { task_queue: task_queue.clone() }, Spawner { task_queue })
}