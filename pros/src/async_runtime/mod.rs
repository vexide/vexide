use core::{cell::RefCell, future::Future};

use alloc::rc::Rc;

pub(crate) mod executor;
pub(crate) mod reactor;

pub struct JoinHandle<T> {
    output: Rc<RefCell<Option<T>>>,
    _marker: core::marker::PhantomData<T>,
}
impl<T> JoinHandle<T> {
    pub fn join(self) -> T {
        loop {
            if let Some(output) = self.output.borrow_mut().take() {
                break output;
            }

            executor::EXECUTOR.with(|e| (*e).tick());
        }
    }
}

pub trait FutureExt: Future + 'static + Sized {
    fn block_on(self) -> Self::Output {
        block_on(self)
    }
}
impl<F> FutureExt for F where F: Future + Send + 'static {}

pub fn spawn<T>(future: impl Future<Output = T> + Send + 'static) -> JoinHandle<T> {
    executor::EXECUTOR.with(|e| e.spawn(future))
}

pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    executor::EXECUTOR.with(|e| e.block_on(future))
}

pub fn complete() {
    executor::EXECUTOR.with(|e| e.complete());
}

#[macro_export]
macro_rules! ready {
    ($e:expr) => {
        match $e {
            core::task::Poll::Ready(val) => val,
            core::task::Poll::Pending => return core::task::Poll::Pending,
        }
    };
}
