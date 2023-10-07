use core::{future::Future, task::Poll};

pub(crate) mod executor;
pub(crate) mod task;

pub trait FutureExt: Future + Send + 'static {
    fn block_on(self) -> Self::Output
    where
        Self::Output: Send,
        Self: Sized,
    {
        block_on(self)
    }
}
impl<F> FutureExt for F where F: Future + Send + 'static {}

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    let executor = executor::Executor::new();
    let task = executor.spawn(future);

    loop {
        match task.poll() {
            Poll::Ready(val) => break val,
            Poll::Pending => {
                executor.tick();
            }
        }
    }
}
