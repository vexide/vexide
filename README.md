# vexide

Open-source Rust runtime for VEX robots. vexide provides a safe and efficient set of APIs and tools for programming VEX robots in Rust!

## Getting Started

vexide is published on [crates.io](https://crates.io/crates/vexide) and can be used like a normal Rust crate.

If you're just getting started, we recommend going through our [docs](https://vexide.dev/docs/), which provide step-by-step instructions for setting up a development environment with [vexide-template](https://github.com/vexide/vexide-template). You can also use our [examples](./examples/) as a reference for your own projects.

## Project Structure

The vexide runtime is a fairly standard rust monorepo split into 5 subcrates:

- [`vexide-core`](https://crates.io/crates/vexide_core) provides common low-level system APIs, such as competition control, synchronization primitives, and backtrace collection.
- [`vexide-devices`](https://crates.io/crates/vexide_devices) provides APIs for all VEX peripherals and hardware, allowing you to control motors and sensors from Rust code.
- [`vexide-async`](https://crates.io/crates/vexide_async) implements vexide's async executor and runtime, as well as a few common futures.
- [`vexide-startup`](https://crates.io/crates/vexide_startup) contains bare-metal runtime initialization code for booting a freestanding vexide program on the V5 brain.
- [`vexide-macro`](https://crates.io/crates/vexide_macro) implements the `#[vexide::main]` proc-macro.

These subcrates are exported from a single [`vexide`](https://github.com/vexide/vexide/blob/main/packages/vexide/src/lib.rs) crate intended to be used as a complete package.

## Building

vexide relies on some features that are only available in Rust’s nightly release channel, so you’ll need to switch to using nightly. We also depend on the `rust-src` component due to our embedded target requiring a build of `core`.

```sh
rustup override set nightly
rustup component add rust-src
```

This project is compiled like any other Rust project with one caveat - we have our own dedicated wrapper over `cargo` called `cargo-v5`, which wraps the normal `cargo build` command and allows for uploading to a VEX brain over USB.

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
> Using `cargo-v5` is optional if you just want to test if your changes compile. In order to upload programs or examples, you will need to use `cargo-v5`.

## Testing Your Changes

When making changes to vexide, it's a good idea to test them. The easiest way to do this is by running one of our examples. `cargo-v5` can be used to upload an example by running a command like this:

```sh
cargo v5 upload --example basic --release
```

Depending on what you have changed, the basic example may not be the best example to test. We have many examples covering different parts of vexide, so choose the one that applies to your changes. If there isn't one, feel free to add it!
