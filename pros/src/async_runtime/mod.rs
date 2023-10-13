use core::future::Future;

pub(crate) mod executor;
pub(crate) mod reactor;

pub trait FutureExt: Future + 'static {
    fn block_on(self) -> Self::Output
    where
        Self: Sized,
    {
        block_on(self)
    }
}
impl<F> FutureExt for F where F: Future + Send + 'static {}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    executor::EXECUTOR.with(|e| e.spawn(future));
}

pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    executor::EXECUTOR.with(|e| e.block_on(future))
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
