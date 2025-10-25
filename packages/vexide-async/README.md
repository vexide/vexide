Tiny async runtime for `vexide`.

This crate contains an implementation of vexide's async executor, which is driven by `smol`'s [`async_task`] crate. The executor is optimized for use on VEXos-based systems and supports spawning tasks and blocking on futures. It has a reactor to improve the performance of some futures (such as [`Sleep`]).

[`Sleep`]: https://docs.rs/vexide-async/latest/vexide_async/time/struct.Sleep.html
[`async_task`]: https://crates.io/crates/async-task
