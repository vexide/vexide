# Pros-rs

Work in progress opinionated Rust bindings for the [PROS](https://github.com/purduesigbots/pros) library and kernel.

This project is still early in development.
Make sure to check out the todo list [(TODO.md)](TODO.md)

## Usage

## Compiling

The only dependency of pros-rs outside of Rust is The Arm Gnu Toolchain (arm-none-eabi-gcc).

Read the installation guide for your OS to see how to get things set up.

### Windows
Steps:
1. Run The Arm Gnu Toolchain [here](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads)
2. Install the pros cli, instructions are [here](https://pros.cs.purdue.edu/v5/getting-started/windows.html)
3. Install cargo pros with ``cargo install cargo-pros``

To compile and the project rust run ``cargo pros build``.

### Linux

The steps for getting pros-rs compiling are slightly different based on if you use Nix or not.

#### With Nix

The Nix flake contains the Arm Gnu Toolchain, cargo pros, and pros-cli.

There is a ``.envrc`` file included for Nix + Direnv users.

#### Without Nix

install arm-none-eabi-gcc and pros-cli from your package manager of choice. 
Cargo pros can be installed with ``cargo install cargo-pros``

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

If for some reason you want to do it manually, this is the command: 
`cargo build --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort`.

The extra build-std argument is neccesary because this repository's `.cargo/config.toml` enables build-std but only for core, alloc, and compiler_builtins. WebAssembly does come with `std` but there is [currently](https://github.com/rust-lang/cargo/issues/8733) no way to conditionally enable build-std.
