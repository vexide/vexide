# vexide

> [!IMPORTANT]
>
> This branch contains a version of vexide refactored to prepare for adding VEX V5 support
> to the upstream Rust compiler & standard library. It has different usage requirements
> than the trunk edition of vexide; for instance, until our changes are merged, you must
> use our fork of Rust.
>
> <details>
> <summary>Click to view details</summary>
>
> vexide will automatically detect if you are using the builtin Rust target by checking for
> `cfg(not(target_os = "none"))` and adapt by enabling or disabling features as available.
>
> Currently, differential uploading and task-locals are unsupported when using the builtin Rust
> target because they require modifications to a program's memory layout at link-time.
>
> ## Building Rust
>
> Clone the `minimal-armv7a-vex-v5` branch of [our fork of Rust](https://github.com/vexide/rust),
> then run `./x setup`. Specify "compiler" if you intend to make changes to Rust, or "dist"
> if you just want to use vexide.
>
> You will need to update `bootstrap.toml` to something like the following:
>
> ```toml
> # See bootstrap.example.toml for documentation of available options
> #
> profile = "compiler"  # Includes one of the default files in src/bootstrap/defaults
> change-id = 144675
>
> llvm.download-ci-llvm = true
>
> [rust]
> # Effectively copies the LLVM linker (downloaded from CI) to the Rust build outputs
> lld = true
> # optional; makes the build output bigger, but subsequent builds are faster
> incremental = true
> ```
>
> Build it with `./x build library compiler cargo clippy proc-macro-srv-cli` (the last 3 arguments
> help rust-analyzer work properly).
>
> Run `rustup toolchain link vexv5 build/host/stage1` to give this toolchain a name ("vexv5").
>
> ## Building vexide
>
> Tell Cargo to use the new "vexv5" toolchain in this directory:
>
> ```shell
> rustup override set vexv5
> ```
>
> `cargo v5 upload` will tell Cargo to use the old JSON-based target. You will need to
> build and upload in two steps to avoid this.
>
> ```shell
> cargo build --example basic
> cargo v5 upload --file target/armv7a-vex-v5/debug/examples/basic
> ```
>
> </summary>

Open-source Rust runtime for VEX V5 robots. vexide provides a `no_std` Rust runtime, async executor, device API, and more for the VEX V5 Brain!

vexide is the successor to [pros-rs](https://github.com/vexide/pros-rs) which is a set of unmaintained APIs using bindings over [PROS](https://github.com/purduesigbots/pros).

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
- [`vexide-macro`](https://crates.io/crates/vexide_macro) contains the source code for the `#[vexide::main]` proc-macro.

These subcrates are exported from a single [`vexide`](https://github.com/vexide/vexide/blob/main/packages/vexide/src/lib.rs) crate intended to be used as a complete package.

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
