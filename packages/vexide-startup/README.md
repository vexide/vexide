# vexide-startup

Startup primitives for the [vexide runtime](https://vexide.dev/). This project provides a bare-metal entrypoint that freestanding Rust binaries can leverage to run on the VEX V5 brain.

This includes:
- Stack setup
- Code signature/program header types
- BSS section handling
- Global allocator setup for [`vexide_core`](https://crates.io/crates/vexide_core).
- vexos background processing
