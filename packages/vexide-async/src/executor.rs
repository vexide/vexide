use alloc::{collections::VecDeque, sync::Arc};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use waker_fn::waker_fn;

use super::reactor::Reactor;
use crate::{
    task::{Task, TaskMetadata},
};
#[cfg(target_os = "none")]
use crate::local::{is_tls_null, set_tls_ptr, TaskLocalStorage};

type Runnable = async_task::Runnable<TaskMetadata>;

pub(crate) static EXECUTOR: Executor = Executor::new();

pub(crate) struct Executor {
    queue: RefCell<VecDeque<Runnable>>,
    reactor: RefCell<Reactor>,
}

//SAFETY: user programs only run on a single thread cpu core and interrupts are disabled when modifying executor state.
unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

impl Executor {
    pub const fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            reactor: RefCell::new(Reactor::new()),
        }
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + 'static) -> Task<T> {
        let metadata = TaskMetadata {
            #[cfg(target_os = "none")]
            tls: TaskLocalStorage::new(),
        };

        // SAFETY: `runnable` will never be moved off this thread or shared with another thread because of the `!Send + !Sync` bounds on `Self`.
        //         Both `future` and `schedule` are `'static` so they cannot be used after being freed.
        //   TODO: Make sure that the waker can never be sent off the thread.
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
            #[cfg(target_os = "none")]
            let old_ptr = unsafe { runnable.metadata().tls.set_current_tls() };
            runnable.run();

            #[cfg(target_os = "none")]
            unsafe { set_tls_ptr(old_ptr) };

            true
        } else {
            false
        }
    }

    pub fn block_on<R>(&self, mut task: Task<R>) -> R {
        // indicative of entry point task
        #[cfg(target_os = "none")]
        if is_tls_null() {
            unsafe {
                _ = TaskLocalStorage::new().set_current_tls();
            }
        }

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
