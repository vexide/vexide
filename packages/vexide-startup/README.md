VEX user program startup and runtime support.

This crate provides runtime infrastructure for booting vexide programs from Rust code. This infrastructure includes a more optimized heap allocator, differential uploading support, and a panic hook that draws panic messages to the screen and captures backtraces.

- User code begins at an assembly routine called `_vexide_boot`, which sets up the stack section before jumping to the `_start` routine defined in libstd, which then calls the `main` function.
- From there, consumers must call the [`startup`] function to finish the startup process by applying differential upload patches, claiming heap space, and setting up this crate's custom panic hook.

This crate does NOT provide a `libc` [crt0 implementation]. No `libc`-style global constructors are called. This means that the [`__libc_init_array`] function must be explicitly called if you wish to link to C libraries.

[`startup`]: https://docs.rs/vexide-startup/latest/vexide_startup/fn.startup.html
[crt0 implementation]: https://en.wikipedia.org/wiki/Crt0
[`__libc_init_array`]: https://maskray.me/blog/2021-11-07-init-ctors-init-array
