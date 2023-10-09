use core::{
    cell::{OnceCell, RefCell},
    future::Future,
    pin::Pin,
    task::Context,
};

use alloc::{boxed::Box, collections::VecDeque, rc::Rc};

use crate::task_local;

task_local! {
    pub(crate) static EXECUTOR: Rc<Executor> = Rc::new(Executor::new())
}

type ExecutableFuture = Pin<Box<dyn Future<Output = ()>>>;

pub struct Executor {
    queue: RefCell<VecDeque<ExecutableFuture>>,
}
impl Executor {
    pub fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.queue.borrow_mut().push_back(Box::pin(future));
    }

    pub fn block_on<F: Future + 'static>(&self, future: F) -> F::Output {
        let output = Rc::new(RefCell::new(None));

        let _ = self.queue.borrow_mut().push_back(Box::pin({
            let output = output.clone();

            async move {
                let future_output = future.await;
                *output.borrow_mut() = Some(future_output);
            }
        }));

        loop {
            // we unwrap here because the queue should only be empty after our output value is set
            let mut task = self.queue.borrow_mut().pop_front().unwrap();

            let cx = &mut Context::from_waker(futures::task::noop_waker_ref());
            if task.as_mut().poll(cx).is_pending() {
                let _ = self.queue.borrow_mut().push_back(task);
            }

            if let Some(output) = output.borrow_mut().take() {
                break output;
            }
        }
    }
}
