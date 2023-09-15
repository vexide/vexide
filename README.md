# Pros-rs

Work in progress opinionated Rust bindings for the [PROS](https://github.com/purduesigbots/pros) library and kernel.

## This project is still early in development

Make sure to check out the todo list [(TODO.md)](TODO.md)

## Compiling

Pros-rs has a few dependancies, namely:

* Rust (obviously)
* The Arm Gnu Toolchain (arm-none-eabi-gcc)
* Clang
* Python
* Pros (pros-cli)

Read the installation guide for your OS to see how to install all of them.

### Windows

Windows is always tricky to compile things on, but luckily you won't have to do too much.

First install Clang. To install Clang either [install it manually](https://releases.llvm.org/),
or if you dont mind it being outdated, run [the installer](https://llvm.org/builds/).

Next install the Arm GNU Toolchain which has an installer [here](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads)

Next install the pros cli, instructions are [here](https://pros.cs.purdue.edu/v5/getting-started/windows.html)

Finally, find the directory of the Arm GNU Toolchain and navigate to arm-none-eabi/include.
Copy the new path. What we need to do now is set the CPATH environment variable to that path.
To set the environment variable temporarily you can run a command in the terminal.
If you use cmd run ``set CPATH=<paste path here>``, If you use powershell run ``$env:CPATH = <paste path here>``.
To chage the environment variable permanently search in the task bar for "Edit Environment Variables" and open it.
Once you are there create a new entry named CPATH and set it to the copied path.

After all that, you are done!
To compile and the project rust run ``cargo r``.

### Linux

The easiest way to get things working on linux is to install nix and direnv, or if you are on nixos, just install direnv.

Installing nix is easy, just follow [their instructions](https://nixos.org/download.html).
Installing direnv is also easy, [follow their guide](https://direnv.net/#basic-installation)

Your may have to enable some options to get nix and direnv to work together, but once that is done they are completely set up.

Unfortunately the pros-cli isn't packaged for nixpkgs, which means that you need to install it manually with ``pip install --user pros-cli``.
On nixos though, you are unfortunately out of luck, at least for now.

Run ``direnv allow`` and then ``cargo r``. And you should be good!

### MacOS

This project depends on the Xcode Command Line Tools.
Chances are that if you develop on MacOS you have them already, but if not you can install them with `xcode-select --install`.

Most of the other dependencies can easily be installed with Homebrew.

Install the Arm GNU Toolchain with
`brew install osx-cross/arm/arm-gcc-bin`.

Install pros-cli with
`brew install purduesigbots/pros/pros-cli`.

And you are done! Compile the project with `cargo build`.
