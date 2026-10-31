<h1 align="center">

<img src="https://vexide.dev/images/logo.svg" height="40" align="absmiddle" alt=""> &nbsp;vexide

[![Website][webimg]][weblink] [![docs.rs][blogimg]][bloglink] [![docs.rs][docsimg]][docslink] [![docs.rs][discordimg]][discordlink]

[webimg]: https://img.shields.io/badge/Website-gray
[weblink]: https://vexide.dev
[blogimg]: https://img.shields.io/badge/Blog-gray
[bloglink]: https://vexide.dev/blog/
[docsimg]: https://img.shields.io/badge/API%20Documentation-gray
[docslink]: https://docs.rs/vexide
[discordimg]: https://img.shields.io/discord/1191121307199352923?logo=discord&logoColor=white
[discordlink]: https://discord.gg/d4uazRf2Nh
</h1>

Open-source Rust runtime for VEX V5 robots. vexide provides a safe and efficient set of APIs and tools for programming VEX robots in Rust!

```rust
use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    // Create a green motor on port 1.
    let mut my_motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);

    // Spin the motor at 10 volts!
    _ = my_motor.set_voltage(10.0);
}
```

## What is vexide?

vexide is an independent, community-maintained library for programming VEX robots. It's alternative to [VEXcode](https://www.vexrobotics.com/vexcode) and [PROS](https://pros.cs.purdue.edu/) for students who want to write their VEX V5 robot code in Rust instead of C or C++.

Since vexide handles low-level features like task scheduling, memory allocation, and competition state, you can focus on writing competitive, performant, and reliable code!

### Features

- **Reliability:** Rust catches whole classes of bugs at compile time, and vexide's APIs are designed around explicit error handling. Safe Rust code should never be able to cause a memory error on the Brain; if it does, that's a bug in vexide.
- **Async multitasking:** Instead of an RTOS, vexide uses Rust's `async`/`await`. Spawning async tasks is cheap and there are no data races between them.
- **Fast program uploads:** Differential uploads only transmit the parts of your program that have changed, so wireless uploads are nearly instant even for large projects.
- **Standard Rust tooling:** A vexide project is a normal Cargo project. You can use the standard library, add crates from [crates.io](https://crates.io/), organize code into modules and workspaces, and use standard IDE tooling.
- **Testing without a robot:** vexide programs can be compiled and run on your computer against a mocked SDK, and you can unit test your robot logic with `cargo test`.
- **Useful crash reports:** If a program panics or the CPU faults, vexide shows detailed info about the error, a backtrace, and register state at the time of the crash.
- **Fully open source:** vexide is free and open source under the permissive MIT license with no proprietary code by default. It's maintained by current and former competitors around the world, and we've documented [how it works](https://internals.vexide.dev/).

vexide isn't aimed at people who are new to programming. If you haven't used Rust before, we recommend working through at least the first half of [the Rust Book](https://doc.rust-lang.org/book/) first.

## Getting Started

To start using vexide, we recommend going through our [documentation](https://vexide.dev/docs/), which includes step-by-step instructions for setting up a development environment and guides for some of vexide's commonly-used features.

Other resources:

- The [API reference](https://docs.rs/vexide) documents all functions and types.
- vexide's [examples](./examples/) show complete programs you can use as a starting point.

## Help

If you have questions, come ask in our [Discord server][discordlink]!

## License

vexide is distributed under the terms of the MIT license ([LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>).

## Contributing

For guidelines for contributors, please see [CONTRIBUTING.md](CONTRIBUTING.md).
