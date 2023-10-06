use core::ptr::NonNull;

use alloc::sync::Arc;
use slab::Slab;
use spin::Once;

use crate::sync::Mutex;

pub struct Task<T: Send> {
    pub returns: Arc<Mutex<Slab<Once<NonNull<()>>>>>,
    pub return_key: usize,

    pub _marker: core::marker::PhantomData<T>,
}

impl<T: Send> Task<T> {
    pub fn poll(&self) -> core::task::Poll<T> {
        match self.returns.lock()[self.return_key]
            .poll()
            .map(|ptr| unsafe { (ptr.as_ptr() as *mut T).read() })
        {
            Some(val) => core::task::Poll::Ready(val),
            None => core::task::Poll::Pending,
        }
    }
}
