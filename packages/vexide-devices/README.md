VEX hardware abstractions and peripheral access.

This crate provides APIs for interfacing with hardware and peripherals sold by VEX robotics.

The VEX V5 Brain features 21 RJ9 serial ports (known as "Smart Ports") for communicating with newer V5 devices, as well as six three-wire ports with analog-to-digital conversion capability for compatibility with legacy Cortex devices. The Brain also has a screen, battery, and usually a controller for reading user input.

Hardware access begins at the [`Peripherals`] API, where a singleton to the brain's available I/O and peripherals can be obtained:

[`Peripherals`]: https://docs.rs/vexide-devices/latest/vexide_devices/peripherals/struct.Peripherals.html
