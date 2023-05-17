# Pros-rs

Work in progress opinionated Rust bindings for the [PROS](https://github.com/purduesigbots/pros) library and kernel.

## This project is still early in development

Make sure to check out the todo list [(TODO.md)](TODO.md)

## Compiling

### Windows

Windows is always tricky to compile things on, but luckily you won't have to do too much.
You will need to install Clang, the Arm GNU Toolchain, and Rust and Python if you don't already have them, as well as set the CPATH environment variable.

The first step is to install Clang. To install Clang either [install it manually](https://releases.llvm.org/),
or if you dont mind it being outdated, run [the installer](https://llvm.org/builds/).

The next step is to install the Arm GNU Toolchain which has an installer [here](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads)

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

After direnv and nix are installed, run ``direnv allow`` and then ``cargo r``. And you should be good!

### MacOS

building on MacOS works but this guide is currently WIP.
