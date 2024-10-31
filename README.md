# vexide

Open-source Rust runtime for VEX V5 robots. vexide provides a `no_std` Rust runtime, async executor, device API, and more for the VEX V5 Brain!

vexide is the successor to [pros-rs](https://github.com/vexide/pros-rs) which are a set of unmaintained API using bindings over [PROS](https://github.com/purduesigbots/pros).

> [!WARNING]
> vexide is still considered experimental, but can be used today. Check out our [docs](https://vexide.dev/docs) on how to get started.

## Getting Started

vexide is published on [crates.io](https://crates.io/crates/vexide) and can be used like a normal embedded Rust crate.

If you're just getting started, we recommend going through our [docs](https://vexide.dev/docs/), which provide step-by-step instructions for setting up a development environment with [vexide-template](https://github.com/vexide/vexide-template). You can also use our [examples](./examples/) as a reference for your own projects.

## Project Structure

The vexide runtime is a fairly standard rust monorepo split into 7 subcrates:
- [`vexide-core`](https://crates.io/crates/vexide_core) provides lowlevel core functionality for programs, such as allocators, synchronization primitives, serial printing, I/O and timers.
- [`vexide-devices`](https://crates.io/crates/vexide_devices) contains all device-related bindings for things like motors and sensors.
- [`vexide-async`](https://crates.io/crates/vexide_async) implements our cooperative async runtime as well as several important async futures.
- [`vexide-startup`](https://crates.io/crates/vexide_startup) contains bare-metal startup code required to get freestanding user programs running on the Brain.
- [`vexide-panic`](https://crates.io/crates/vexide_panic) contains our [panic handler](https://doc.rust-lang.org/nomicon/panic-handler.html).
- [`vexide-graphics`](https://crates.io/crates/vexide_graphics) implements graphics drivers for some popular embedded Rust graphics libraries like [Slint] and [`embedded-graphics`].
- [`vexide-macro`](https://crates.io/crates/vexide_macro) contains the source code for the `#[vexide::main]` proc-macro.

These subcrates are exported from a single [`vexide`](https://github.com/vexide/vexide/blob/main/packages/vexide/src/lib.rs) crate intended to be used as a complete package.

[Slint]: https://slint.dev/
[`embedded-graphics`]: https://crates.io/crates/embedded-graphics

## Building

vexide relies on some features that are only available in Rust’s nightly release channel, so you’ll need to switch to using nightly. We also depend on the `rust-src` component due to our embedded target requiring a build of `core`.

```sh
rustup override set nightly
rustup component add rust-src
```

This project is compiled like any other Rust project with one caveat - we have our own dedicated wrapper over `cargo` called `cargo-v5`, which passes some additional arguments to `cargo` to correctly build for the platform.

You can install that tool with the following command:

```sh
cargo install cargo-v5
```

From there, the project can be built like any other Rust library through `cargo-v5`:

```sh
cargo v5 build --release
```

Examples can similarly be built this way:

```sh
cargo v5 build --example basic --release
```

> [!NOTE]
> If you don't want to use `cargo-v5` to build your project, you can effectively do the same thing that it's doing by running `cargo build --target ./armv7a-vex-v5.json -Zbuild-std=core,alloc,compiler_builtins`

## Testing Your Changes

When making changes to vexide, it's a good idea to test them. The easiest way to do this is by running one of our examples. `cargo-v5` can be used to upload an example by running a command like this:
```sh
cargo v5 upload --example basic --release
```
Depending on what you have changed, the basic example may not be the best example to test. We have many examples covering different parts of vexide, so choose the one that applies to your changes. If there isn't one, feel free to add it!

## Building for WASM

The vexide runtime is also designed in a way that it can be compiled for the `wasm32-unknown-unknown` target (along with the existing bare metal ARM target). This is done to allow for simulating programs in a [WASM environment](https://github.com/vexide/v5wasm).

To build projects in this repository for WebAssembly, run `cargo v5 build -s`
This will automatically pass all of the correct arguments to cargo.
