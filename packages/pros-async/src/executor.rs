use alloc::{collections::VecDeque, sync::Arc};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
    time::Duration,
};

use async_task::{Runnable, Task};
use pros_core::{os_task_local, task::delay};
use waker_fn::waker_fn;

use super::reactor::Reactor;

os_task_local! {
    pub(crate) static EXECUTOR: Executor = Executor::new();
}

pub(crate) struct Executor {
    queue: RefCell<VecDeque<Runnable>>,
    pub(crate) reactor: RefCell<Reactor>,
}

impl !Send for Executor {}
impl !Sync for Executor {}

impl Executor {
    pub const fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            reactor: RefCell::new(Reactor::new()),
        }
    }

    pub fn spawn<T>(&'static self, future: impl Future<Output = T> + 'static) -> Task<T> {
        // SAFETY: `runnable` will never be moved off this thread or shared with another thread because of the `!Send + !Sync` bounds on `Self`.
        //         Both `future` and `schedule` are `'static` so they cannot be used after being freed.
        //   TODO: Make sure that the waker can never be sent off the thread.
        let (runnable, task) = unsafe {
            async_task::spawn_unchecked(future, |runnable| {
                self.queue.borrow_mut().push_back(runnable)
            })
        };

        runnable.schedule();

        task
    }

    pub(crate) fn tick(&self) -> bool {
        self.reactor.borrow_mut().tick();

        let runnable = {
            let mut queue = self.queue.borrow_mut();
            queue.pop_front()
        };
        match runnable {
            Some(runnable) => {
                runnable.run();
                true
            }
            None => false,
        }
    }

    pub fn block_on<R>(&self, mut task: Task<R>) -> R {
        let woken = Arc::new(AtomicBool::new(true));

        let waker = waker_fn({
            let woken = woken.clone();
            move || woken.store(true, Ordering::Relaxed)
        });
        let mut cx = Context::from_waker(&waker);

        loop {
            if woken.swap(false, Ordering::Relaxed) {
                if let Poll::Ready(output) = Pin::new(&mut task).poll(&mut cx) {
                    return output;
                }
                self.tick();
                // there might be another future to poll, so we continue without sleeping
                continue;
            }

            delay(Duration::from_millis(10));
            self.tick();
        }
    }
}
