use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Waker},
};

use alloc::{boxed::Box, collections::VecDeque, rc::Rc, task::Wake};

use crate::task_local;

use super::reactor::Reactor;

task_local! {
    pub(crate) static EXECUTOR: Rc<Executor> = Rc::new(Executor::new())
}

type Task = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) struct Executor {
    queue: RefCell<VecDeque<Task>>,
    pub(crate) reactor: Reactor,
}
impl Executor {
    pub fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            reactor: Reactor::new(),
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.queue.borrow_mut().push_back(Box::pin(future));
    }

    fn tick(&self) -> bool {
        self.reactor.tick();

        let mut task = match self.queue.borrow_mut().pop_front() {
            Some(task) => task,
            None => return false,
        };

        let task_waker = alloc::sync::Arc::new(TaskWaker {
            task: RefCell::new(None),
        });
        let waker = Waker::from(task_waker.clone());

        let cx = &mut Context::from_waker(&waker);
        if task.as_mut().poll(cx).is_pending() {
            task_waker.task.borrow_mut().replace(task);
        }

        true
    }

    pub fn block_on<F: Future + 'static>(&self, future: F) -> F::Output {
        let output = Rc::new(RefCell::new(None));

        self.queue.borrow_mut().push_back(Box::pin({
            let output = output.clone();

            async move {
                let future_output = future.await;
                *output.borrow_mut() = Some(future_output);
            }
        }));

        loop {
            self.tick();

            if let Some(output) = output.borrow_mut().take() {
                break output;
            }
        }
    }

    pub fn complete(&self) {
        while self.tick() {}
    }
}

pub struct TaskWaker {
    task: RefCell<Option<Task>>,
}
// These are here to apease the waker struct.
// The executor is single threaded and this waker will never be passed around threads or shared between threads.
unsafe impl Send for TaskWaker {}
unsafe impl Sync for TaskWaker {}

impl Wake for TaskWaker {
    fn wake(self: alloc::sync::Arc<Self>) {
        if let Some(task) = self.task.borrow_mut().take() {
            EXECUTOR.with(|e| e.queue.borrow_mut().push_back(task))
        }
    }
}
