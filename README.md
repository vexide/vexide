# vexide

Open-source Rust runtime for VEX robots. vexide provides a safe and efficient set of APIs and tools for programming VEX robots in Rust!

## Getting Started

To start using vexide, we recommend going through our [docs](https://vexide.dev/docs/), which contain step-by-step instructions for setting up a development environment using [vexide-template](https://github.com/vexide/vexide-template) and tutorials for many of vexide's common features. You can also use our [examples](./examples/) as a reference for your own projects.

## Project Structure

vexide's codebase is a fairly standard rust workspace split into 5 crates:

- [`vexide-core`](https://crates.io/crates/vexide_core) implements common low-level system APIs, including competition control, OS facilities, and backtrace collection.
- [`vexide-devices`](https://crates.io/crates/vexide_devices) contains APIs for VEX peripherals and hardware, allowing you to control motors and sensors from Rust code.
- [`vexide-async`](https://crates.io/crates/vexide_async) implements vexide's async executor and runtime, as well as a few common futures and synchronization primitives.
- [`vexide-startup`](https://crates.io/crates/vexide_startup) contains bare-metal runtime initialization code for booting a freestanding vexide program on a V5 brain.
- [`vexide-macro`](https://crates.io/crates/vexide_macro) implements vexide's proc-macros, including the `#[vexide::main]` attribute.

All of these crates are re-exported from the [`vexide`](https://github.com/vexide/vexide/blob/main/packages/vexide/src/lib.rs) crate to be used as a single package.

## Building

vexide relies on some features that are only available in Rust’s nightly release channel, so you’ll need to switch to using nightly to build it. We also use the `rust-src` component due to our target not shipping pre-built versions of the standard library in `rustup`.

```sh
rustup override set nightly
rustup component add rust-src
```

If you want to run your programs on the V5 brain, you'll additionally need to install our upload tool, `cargo-v5`. It extends the normal `cargo build` command, converting executables to a compatible format and sending them over a wired or wireless connection.

You can install that tool with the following command:

```sh
cargo install cargo-v5
```

From there, the project can be built like any other Rust library using `cargo v5`:

```sh
cargo v5 build -p vexide
```

Examples can similarly be built this way:

```sh
cargo v5 build --example basic --release
```

> [!NOTE]
> Installing `cargo-v5` isn't needed if you just want to test if your changes compile. `vexide` supports compiling to host targets using a mocked version of the VEX SDK for testing changes without a Brain. If you want to upload programs/examples or build a program for the Brain, you will need `cargo-v5`.

## Testing

When making changes to vexide, it's a good idea to test them. `vexide` supports Rust's [testing features](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) for verifying that code behaves how it should. You can test vexide using `cargo test`, like any other Rust project:

```sh
cargo test --all
```

You can also run one of our [examples](./examples/).

```sh
cargo v5 upload --example basic --release
```

Depending on what you have changed, the basic example may not be the best example to test. We have many examples covering different parts of vexide, so choose the one that applies to your changes. If there isn't one, feel free to add it!
