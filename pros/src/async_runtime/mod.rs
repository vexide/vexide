use core::{cell::RefCell, future::Future};

use alloc::rc::Rc;

pub(crate) mod executor;
pub(crate) mod reactor;

pub struct JoinHandle<T> {
    output: Rc<RefCell<Option<T>>>,
}
impl<T> JoinHandle<T> {
    /// Blocks the current task untill a return value can be extracted from your future.
    /// Does not poll all futures to completion. 
    /// If you want to complete all futures, use the [`complete`] function.
    pub fn join(self) -> T {
        loop {
            if let Some(output) = self.output.borrow_mut().take() {
                break output;
            }

            executor::EXECUTOR.with(|e| (*e).tick());
        }
    }

    /// Polls if the future has completed.
    /// If the future has completed, returns `Poll::Ready(output)`.
    pub fn poll(&self) -> core::task::Poll<T> {
        if let Some(output) = self.output.borrow_mut().take() {
            core::task::Poll::Ready(output)
        } else {
            core::task::Poll::Pending
        }
    }
}

pub trait FutureExt: Future + 'static + Sized {
    /// Blocks the current task and polls the future to completion, returning the output of the future.
    fn block_on(self) -> Self::Output {
        block_on(self)
    }
}
impl<F> FutureExt for F where F: Future + Send + 'static {}

/// Runs a future in the background without having to await it
/// To get the the return value you can call [`JoinHandle.join`](JoinHandle::join).
pub fn spawn<T>(future: impl Future<Output = T> + Send + 'static) -> JoinHandle<T> {
    executor::EXECUTOR.with(|e| e.spawn(future))
}

/// Blocks the current task untill a return value can be extracted from the provided future.
/// Does not poll all futures to completion. 
/// If you want to complete all futures, use the [`complete`] function.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    executor::EXECUTOR.with(|e| e.block_on(future))
}

/// Blocks the current task and polls all futures to completion.
pub fn complete() {
    executor::EXECUTOR.with(|e| e.complete());
}

/// Same as [`complete`] but returns None if the executor can not complete all tasks before the given timeout.
pub fn complete_timeout(timeout: core::time::Duration) -> Option<()> {
    let sleep_handle = spawn(crate::task::sleep(timeout));

    loop {
        executor::EXECUTOR.with(|e| e.tick());

        if executor::EXECUTOR.with(|e| e.is_completed()) {
            return Some(());
        } else if sleep_handle.poll().is_ready() {
            return None;
        }
    }
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
