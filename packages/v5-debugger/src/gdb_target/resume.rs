use gdbstub::{common::Signal, target::ext::base::singlethread::SingleThreadResume};

use crate::gdb_target::V5Target;

impl SingleThreadResume for V5Target {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume = true;
        Ok(())
    }
}
