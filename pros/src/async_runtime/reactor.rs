use core::{cell::RefCell, cmp::Reverse, task::Waker};

use alloc::vec::Vec;

pub struct Sleepers {
    sleepers: Vec<(Waker, u32)>,
}

impl Sleepers {
    pub fn push(&mut self, waker: Waker, target: u32) {
        self.sleepers.push((waker, target));

        self.sleepers.sort_by_key(|(_, target)| Reverse(*target));
    }

    pub fn pop(&mut self) -> Option<Waker> {
        self.sleepers.pop().map(|(waker, _)| waker)
    }
}

pub struct Reactor {
    pub(crate) sleepers: RefCell<Sleepers>,
}

impl Reactor {
    pub fn new() -> Self {
        Self {
            sleepers: RefCell::new(Sleepers {
                sleepers: Vec::new(),
            }),
        }
    }

    pub fn tick(&self) {
        if let Some(sleeper) = self.sleepers.borrow_mut().pop() {
            sleeper.wake()
        }
    }
}
