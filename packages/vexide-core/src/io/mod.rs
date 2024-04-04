mod stdio;

pub use stdio::{stdin, stdout, println, print, dbg, Stdin, StdinLock, Stdout, StdoutLock};
pub use no_std_io::io::*;
