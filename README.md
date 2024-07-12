# vexide

Work in progress high level bindings for the V5 Brain VEX SDK.
Unlike other libraries for the V5 Brain, vexide doesn't use an RTOS.
Instead, vexide leverages Rust's powerful async/await (cooperative multitasking) to allow faster and more user friendly code.

vexide is the successor to [pros-rs](https://github.com/vexide/pros-rs) which now serves as a slightly more stable, but unmaintained api using bindings over [PROS](https://github.com/purduesigbots/pros).

vexide is still in development but we are quickly moving towards competition readiness.

## Usage

After setting up your development environment, connect to a VEX V5 via USB and run `cargo run --example basic` to execute the basic example on the V5 brain. Try the other examples like `screen` or `clawbot`, or create your own vexide project by following by the instructions in the [template](https://github.com/vexide/vexide-template).

## Compiling

The vexide library itself has no external dependencies, but cargo-pros depends on pros-cli for uploading and cargo-binutils for necessary binary modification.
Read the installation guide for your OS to see how to get things set up.

### Windows

Steps:

1. Install the pros cli, instructions are [here](https://pros.cs.purdue.edu/v5/getting-started/windows.html)
2. Install cargo pros with ``cargo install cargo-pros``
3. Install cargo-binutils with ``cargo install cargo-binutils``

To compile the project just run ``cargo pros build``.

### Linux

The steps for getting vexide compiling are slightly different based on if you use Nix or not.

#### With Nix

The Nix flake contains the Arm GNU Toolchain, cargo pros, and pros-cli.

There is a ``.envrc`` file included for Nix + Direnv users.

#### Without Nix

Install cargo-binutils and pros-cli from your package manager of choice.
Cargo pros can be installed with ``cargo install cargo-pros``.

### MacOS

This project depends on the Xcode Command Line Tools.
Chances are that if you develop on MacOS you have them already, but if not you can install them with `xcode-select --install`.

Most of the other dependencies can easily be installed with Homebrew.

Install the Arm GNU Toolchain with
`brew install osx-cross/arm/arm-gcc-bin`.

Install pros-cli with
`brew install purduesigbots/pros/pros-cli`.

And you are done! Compile the project with `cargo build`.

## Compiling for WASM

To build projects in this repository for WebAssembly, run ``cargo pros build -s``
This will automatically pass all of the correct arguments to cargo.

If, for some reason, you want to do it manually, this is the command:
`cargo build --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort`.

The extra build-std argument is neccesary because this repository's `.cargo/config.toml` enables build-std but only for core, alloc, and compiler_builtins. WebAssembly does come with `std` but there is [currently](https://github.com/rust-lang/cargo/issues/8733) no way to conditionally enable build-std.
