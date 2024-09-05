use std::collections::BTreeMap;
use std::task::Waker;
use std::time::Instant;

pub struct Sleepers(BTreeMap<Instant, Waker>);

impl Sleepers {
    pub fn push(&mut self, waker: Waker, instant: Instant) {
        self.0.insert(instant, waker);
    }

    pub fn pop(&mut self) -> Option<Waker> {
        self.0.pop_first().map(|(_, waker)| waker)
    }
}

pub struct Reactor {
    pub(crate) sleepers: Sleepers,
}

impl Reactor {
    pub const fn new() -> Self {
        Self {
            sleepers: Sleepers(BTreeMap::new()),
        }
    }

    pub fn tick(&mut self) {
        if let Some(sleeper) = self.sleepers.pop() {
            sleeper.wake();
        }
    }
}
