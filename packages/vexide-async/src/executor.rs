use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use waker_fn::waker_fn;

use super::reactor::Reactor;
use crate::{
    local::TaskLocalStorage,
    task::{Task, TaskMetadata},
};

type Runnable = async_task::Runnable<TaskMetadata>;

thread_local! {
    pub(crate) static EXECUTOR: Executor = const { Executor::new() };
}

pub(crate) struct Executor {
    queue: RefCell<VecDeque<Runnable>>,
    reactor: RefCell<Reactor>,
    pub(crate) tls: RefCell<Option<Rc<TaskLocalStorage>>>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            reactor: RefCell::new(Reactor::new()),
            tls: RefCell::new(None),
        }
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + 'static) -> Task<T> {
        let metadata = TaskMetadata {
            tls: Rc::new(TaskLocalStorage::new()),
        };

        // SAFETY: `runnable` will never be moved off this thread or shared with another thread
        // because of the `!Send + !Sync` bounds on `Self`.         Both `future` and
        // `schedule` are `'static` so they cannot be used after being freed.   TODO: Make
        // sure that the waker can never be sent off the thread.
        let (runnable, task) = unsafe {
            async_task::Builder::new()
                .metadata(metadata)
                .spawn_unchecked(
                    move |_| future,
                    |runnable| {
                        self.queue.borrow_mut().push_back(runnable);
                    },
                )
        };

        runnable.schedule();

        task
    }

    /// Run the provided closure with the reactor.
    /// Used to ensure the thread safety of the executor.
    pub(crate) fn with_reactor(&self, f: impl FnOnce(&mut Reactor)) {
        f(&mut self.reactor.borrow_mut());
    }

    pub(crate) fn tick(&self) -> bool {
        self.reactor.borrow_mut().tick();

        let runnable = {
            let mut queue = self.queue.borrow_mut();
            queue.pop_front()
        };

        #[allow(if_let_rescope)]
        if let Some(runnable) = runnable {
            TaskLocalStorage::scope(runnable.metadata().tls.clone(), || {
                runnable.run();
            });

            true
        } else {
            false
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
            }

            unsafe {
                vex_sdk::vexTasksRun();
            }

            self.tick();
        }
    }
}

#[cfg(test)]
mod test {
    use vex_sdk_mock as _;

    use super::*;

    #[test]
    fn spawns_task() {
        let executor = Executor::new();

        let result = executor.block_on(executor.spawn(async { 1 }));

        assert_eq!(result, 1);
    }
}
