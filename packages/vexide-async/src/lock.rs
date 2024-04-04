use vexide_core::sync::RawMutex;
use lock_api::RawMutex as _;

use crate::executor::Executor;


pub(crate) struct ExecutorGuard<'a>(&'a ExecutorLock);
impl<'a> Drop for ExecutorGuard<'a> {
    fn drop(&mut self) {
        unsafe { self.0.raw.unlock() };
    }
}
impl core::ops::Deref for ExecutorGuard<'_> {
    type Target = Executor;
    fn deref(&self) -> &Executor {
        &self.0.executor
    }
}

pub(crate) struct ExecutorLock {
    raw: RawMutex,
    executor: Executor,
}
unsafe impl Sync for ExecutorLock {}
impl ExecutorLock {
    pub const fn new() -> Self {
        Self {
            raw: RawMutex::new(),
            executor: Executor::new(),
        }
    }

    pub fn lock(&self) -> ExecutorGuard<'_> {
        self.raw.lock();
        ExecutorGuard(self)
    }
}
