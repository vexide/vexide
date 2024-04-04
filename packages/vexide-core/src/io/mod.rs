mod stdio;

pub use no_std_io::io::*;
pub use stdio::{dbg, print, println, stdin, stdout, Stdin, StdinLock, Stdout, StdoutLock};
