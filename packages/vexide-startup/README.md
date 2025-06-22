# vexide-startup

Startup primitives for the [vexide runtime](https://vexide.dev/). This project provides a bare-metal entrypoint that freestanding Rust binaries can leverage to run on the VEX V5 Brain.

This includes:
- Stack setup
- Code signature/program header types
- BSS section handling
- Global allocator setup for [`vexide_core`](https://crates.io/crates/vexide_core).

## License

In certain cases, vexide-startup programs link with the libc.a and libm.a files in the adjacent `link/` directory.

These are sourced from the official release of ARM Toolchain for Embedded v20.1.0, which uses picolibc.
You can download it yourself from: <https://github.com/arm/arm-toolchain/releases/tag/release-20.1.0-ATfE>.
These binaries are from its build for `armv7a_hard_vfpv3_d16`.

These files have been released under a permissive BSD license detailed in the adjacent file `picolibc-licenses.txt`.
