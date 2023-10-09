use core::future::Future;

pub(crate) mod executor;

pub trait FutureExt: Future {
    fn block_on(self) -> Self::Output
    where
        Self: Sized,
    {
        block_on(self)
    }
}
impl<F> FutureExt for F where F: Future + Send + 'static {}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    executor::EXECUTOR.with(|e| e.get().unwrap().spawn(future));
}

pub fn block_on<F: Future>(future: F) -> F::Output
{
    todo!()
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