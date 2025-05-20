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
    local::{Tls, TlsReg},
    task::Task,
};

type Runnable = async_task::Runnable<Tls>;

pub(crate) static EXECUTOR: Executor = unsafe { Executor::new() };

pub(crate) struct Executor {
    queue: RefCell<VecDeque<Runnable>>,
    reactor: RefCell<Reactor>,
    tls_reg: RefCell<TlsReg>,
}
//SAFETY: user programs only run on a single thread cpu core and interrupts are disabled when modifying executor state.
unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

impl Executor {
    pub const unsafe fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            reactor: RefCell::new(Reactor::new()),
            tls_reg: RefCell::new(unsafe { TlsReg::new() }),
        }
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + 'static) -> Task<T> {
        let tls = Tls::new_alloc();

        // SAFETY: `runnable` will never be moved off this thread or shared with another thread because of the `!Send + !Sync` bounds on `Self`.
        //         Both `future` and `schedule` are `'static` so they cannot be used after being freed.
        //   TODO: Make sure that the waker can never be sent off the thread.
        let (runnable, task) = unsafe {
            async_task::Builder::new().metadata(tls).spawn_unchecked(
                move |_| future,
                |runnable| {
                    self.queue.borrow_mut().push_back(runnable);
                },
            )
            // async_task::spawn_unchecked(future, |runnable| {
            //     self.queue.borrow_mut().push_back(runnable);
            // })
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

        if let Some(runnable) = runnable {
            unsafe {
                self.tls_reg.borrow_mut().set(runnable.metadata());
            }

            runnable.run();
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
