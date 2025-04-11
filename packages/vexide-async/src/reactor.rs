use alloc::collections::BinaryHeap;
use core::task::Waker;

use vexide_core::time::Instant;

pub(crate) struct Sleeper {
    pub deadline: Instant,
    pub waker: Waker,
}

impl PartialEq for Sleeper {
    fn eq(&self, other: &Self) -> bool {
        other.deadline.eq(&other.deadline)
    }
}
impl PartialOrd for Sleeper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Sleeper {}
impl Ord for Sleeper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // NOTE: Sleeper with the earliest deadline (minimum `Instant` ordering) must
        // have the highest priority in the queue.
        self.deadline.cmp(&other.deadline).reverse()
    }
}

pub struct Reactor {
    pub(crate) sleepers: BinaryHeap<Sleeper>,
}

impl Reactor {
    pub const fn new() -> Self {
        Self {
            sleepers: BinaryHeap::new(),
        }
    }

    pub fn tick(&mut self) {
        while let Some(sleeper) = self.sleepers.pop() {
            sleeper.waker.wake();
            if sleeper.deadline < Instant::now() {
                break;
            }
        }
    }
}
