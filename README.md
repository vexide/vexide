# vexide

Work in progress high level bindings for the V5 Brain VEX SDK.
Unlike other libraries for the V5 Brain, vexide doesn't use an RTOS.
Instead, vexide leverages Rust's powerful async/await (cooperative multitasking) to allow faster and more user friendly code.

vexide is the successor to [pros-rs](https://github.com/vexide/pros-rs) which now serves as a slightly more stable, but unmaintained API using bindings over [PROS](https://github.com/purduesigbots/pros).

vexide is still in development but we are quickly moving towards competition readiness.

## Setup

The only tool you will need to install in order to build, upload, and view to output of vexide programs is `cargo-v5`.
Read the installation guide for your OS to see how to get things set up.

### Windows

Install `cargo-v5` with ``cargo install cargo-v5``


### Linux

In order to upload programs without superuser permissions you may have to add your user to the `dialout` group.
The steps for getting vexide compiling are slightly different based on if you use Nix or not.

#### With Nix

The Nix flake contains cargo-v5 and a working Rust toolchain.

There is a ``.envrc`` file included for Nix + Direnv users.

#### Without Nix

Cargo v5 can be installed with ``cargo install cargo-v5``.

### MacOS

This project depends on the Xcode Command Line Tools.
Chances are that if you develop on MacOS you have them already, but if not you can install them with `xcode-select --install`.

Most of the other dependencies can easily be installed with Homebrew.

Install the Arm GNU Toolchain with
`brew install osx-cross/arm/arm-gcc-bin`.

Install pros-cli with
`brew install purduesigbots/pros/pros-cli`.

And you are done! Compile the project with `cargo build`.

## Usage

To upload your project run `cargo v5 upload --release`.
To build your project without uploading it you can run `cargo v5 build --release`.
To view the output of your program run `cargo v5 terminal`.

## Compiling for WASM

To build projects in this repository for WebAssembly, run ``cargo pros build -s``
This will automatically pass all of the correct arguments to cargo.

If, for some reason, you want to do it manually, this is the command:
`cargo build --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort`.

The extra build-std argument is neccesary because this repository's `.cargo/config.toml` enables build-std but only for core, alloc, and compiler_builtins. WebAssembly does come with `std` but there is [currently](https://github.com/rust-lang/cargo/issues/8733) no way to conditionally enable build-std.
