# Examples

Each of vexide's examples showcase a different part of the runtime.

Table of contents:

- [Full Codebases](#full-codebases)
- [Devices](#devices)
- [Async](#async)
- [Macro](#macro)
- [Misc](#misc)

## Full Codebases

Fully functional robot programs.

> [!NOTE]
> These can be used as starting points for your own programs if you don't know where to start.

### [`split_arcade_drive`](./split_arcade_drive.rs)

Implements a drivetrain controlled through arcade drive split between both joysticks on the primary controller.

### [`tank_drive`](./tank_drive.rs)

Implements a full tankdrive drivebase.

### [`clawbot`](./clawbot.rs)

A reimplementation of the PROS clawbot example using vexide.

## Devices

Examples of using specific devices or categories of devices.

### [`adi_expander`](./adi_expander.rs)

Showcases creating a ADI device on an ADI expander.

### [`adi`](./adi.rs)

Showcases creating and reading from several ADI devices on the Brain's built in ADI ports.

### [`pneumatics`](./pneumatics.rs)

Controls pneumatics through an ADI solenoid device.

### [`screen`](./screen.rs)

Showcases writing text to the display through `write!`, drawing a shape, both filled and outlined, and filling the display with a solid color.

### [`smart_devices`](./smart_devices.rs)

Creates and communicates with several smart devices.

## Async

Examples showcasing vexide's async capabilities

### [`competition`](./competition.rs)

Demonstrates how the [`Compete`](https://docs.rs/vexide/latest/vexide/core/competition/trait.Compete.html) trait can be used to run code during specific competition states.

### [`async`](./async.rs)

Explains and showcases several features of vexide's async executor including:

- Tasks
- Sleep futures
- Blocking on a future

### [`sync`](./sync.rs)

Demonstrates the usage of the asynchronous synchronization primitives included in [`vexide-core`](https://docs.rs/vexide/0.4.2/vexide/core/sync/index.html).

## Macro

Examples related to the `vexide::main` macro.

### [`bannerless`](./bannerless.rs)

The banner at program startup can be fully disabled by passing `banner(enabled = false)` to the `vexide::main` macro.

### [`themed_banner`](./themed_banner.rs)

As well as optionally disabling the banner, you can change its color theme by passing a theme to the `vexide::main` macro: `banner(theme = <THEME_CONST>)`.
Several default themes can be found in the [`vexide::startup::banner::themes`](https://docs.rs/vexide/latest/vexide/startup/banner/themes/index.html) module.

### [`custom_code_sig`](./custom_code_sig.rs)

Uses the `vexide::main` macro to set a custom code signature for the example. A custom code signature can be used to slightly alter the way a program runs. For example, the default screen colors can be inverted.

## Misc

Miscelaneous examples.

### [`basic`](./basic.rs)

A simple hello world example program. This is commonly used by vexide contributors to test changes to the runtime.
