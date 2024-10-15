# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Before releasing:

- change versions in Cargo.toml
- change Unreleased to the version number
- create new Unreleased section
- update links at the end of the document
- add "New Contributors" section if there were any first-time contributors

### New Contributors

- @new-contributor made their first contribution in #11!
-->


## [Unreleased]

### Added

- You can now detect controller release occurrences with `ButtonState::is_now_released`.
- Added support for 5.5W motors with a new constructor (`Motor::new_exp`) and four new getters (`Motor::max_voltage`, `Motor::motor_type`, `Motor::is_v5`, and `Motor::is_exp`) for `Motor`. (#167)

### Fixed

- The `dbg!();` now works as expected when no arguments are supplied to it. (#175)

### Changed

- Controller state is now returned all at once to reduce error checking. (#152) (**Breaking Change**)
- `Button::was_pressed` has been renamed to `ButtonState::is_now_pressed`.
- `battery::capacity` now returns from 0.0-1.0 rather than 0-100.
- `battery::voltage` is now returned in volts rather than millivolts.
- `battery::current` is now returned in amps rather than milliamps.
- Changed the incorrect return types of `AdiSolenoid::is_open` and `AdiSolenoid::is_closed` from `LogicLevel` to `bool`. (#164) (**Breaking Change**)
- Renamed `Motor::MAX_VOLTAGE` to `Motor::V5_MAX_VOLTAGE` and added `Motor::EXP_MAX_VOLTAGE`. (#167) (**Breaking Change**)
- Moved the ability to convert Smart devices to `SmartPorts` out of the `SmartDevice` trait and into the devices themselves. (#171) (**Breaking Change**)
- Renamed `OpticalSensor::rgb` to `OpticalSensor::color` and `OpticalSensor::raw` to `OpticalSensor::raw_color` (#179) (**Breaking Change**).

### Removed

### New Contributors

@zabackary made their first contribution in #164!

## [0.4.1]

### Added

### Fixed

- Updated to vex-sdk 0.21.0, fixing ABI incompatibilities between the VEXos calling convention and the hard-float ABI introduced in vexide 0.4.0. This should fix broken functions that pass floats to the SDK. (#156)

### Changed

### Removed

### New Contributors

## [0.4.0]

### Added

- Added support for the V5 GPS Sensor (#79)
- Added support for custom banner themes configurable through the `vexide::main` macro (#127)

### Fixed

- Fixed an issue where the distance sensor relative_size returned a u32 when it can be negative. (#116)
- Fixed an issue preventing the `Screen::draw_buffer` function from working properly. (#128)
- Fixed an issue where panic messages would not be displayed even when the `display_panics` feature was enabled if the screens render mode was set to `DoubleBuffered`. (#134)
- `GpsImu` should now validate on the correct port. (#141)

### Changed

- Refactored the distance sensor API. All readings from the sensor are now read at once in a `object` method that can be possibly `None` if no object was detected. (#122) (**Breaking Change**)
- Adjusted distance sensor status code errors to be more clear.
- Overhauled the design of the startup banner.
- Adjusted distance sensor error names. (#113) (**Breaking Change**)
- Renamed `SmartDevice::port_index` and `SmartPort::index` to `SmartDevice::port_number` and `SmartPort::port_number`. (#121) (**Breaking Change**)
- Renamed `AdiDevice::port_index` and `AdiPort::index` to `AdiDevice::port_number` and `AdiDevice::port_number`. (#121) (**Breaking Change**)
- `SmartPort::device_type` now no longer returns a `Result`. (#121) (**Breaking Change**)
- Updated the names of certain misspelled `enum` variants, constants, and fields. (#132) (**Breaking Change**)
- Marks many futures as `#[must_use]` to warn when futures are created without `await`ing them. (#112)
- Changes the banner attribute syntax in the `vexide::main` macro. (#127) (**Breaking Change**)
- Controller joystick axis getters now return `f64` instead of `f32`. (#133) (**Breaking Change**)
- Fixed an issue where the async executor would block indefinetly on the first program run after a Brain reboot (#139)
- Removed the `critical_section` module from `vexide_core`, since vexide doesn't use interrupts and it can potentially break VEXos operations. (#144) (**Breaking Change**)
- Switched to a hard-float libm build with up to 6 times faster floating point operations. (#145)

### Removed

### New Contributors

## [0.3.0]

### Added

- The startup banner and code signature may now be configured using parameters passed to `vexide::main`. (#102)
- Added the ``ProgramOwner``, ``ProgramType``, and ``ProgramFlags`` types for code signature configuration. (#76)
- Created new ``force_rust_libm`` feature to force the use of a slower, 100% Rust, libm implementation. This is useful for building on WASM. (#106)
- Optimized floating point math operations available through the `Float` extension trait. (#77)
- Added text metrics getters to the `Text` widget. (#83)
- Added alignment support for the `Text` widget. (#85)
- `CompetitonBuilder` functions can now return a `ControlFlow` in order to explicitly end execution. (#89)
- `Point2` can now be converted to mint when using the `nalgebra` feature. (#91)

### Fixed

- Fixed a typo in some conditional compilation for the `smart_leds_trait` and `embedded_graphics` features that prevented them from being enabled.
- Peripherals can now be mutated in the main function (#75)
- Panic messages now output over serial even on `display_panics` feature.

### Changed

- Updated ``vex-sdk`` to version 0.17.0. (#76)
- Renamed ``ColdHeader`` to ``CodeSignature``. (#76) (**Breaking Change**)
- Renamed the entrypoint symbol from ``_entry`` to ``_start``. (#76) (**Breaking Change**)
- Renamed ``__stack_start`` and ``__stack_end`` symbols to ``__stack_top`` and ``__stack_bottom`` respectively. (#76) (**Breaking Change**)
- Renamed the ``.cold_magic`` section to ``.code_signature``. (#76) (**Breaking Change**)
- Made fields on screen widgets public. (#81)
- Renamed `Competition` to `CompetitionRuntime`, `CompetitionRobotExt` to `CompetitionExt`, and `CompetitionRobot` to `Competition`. (#87) (**Breaking Change**)
- Removed the `Error` associated type from the `Competition` trait and made all methods infallible. (#87) (**Breaking Change**)

### Removed

- The `no-banner` feature has been removed from `vexide-startup` and must now be toggled through the `vexide:main` attribute. (#102) (**Breaking Change**)
- Removed the useless ``__rodata_start`` and ``__rodata_end`` symbols.
- Support for `vexide-math` has been dropped. (#78) (**Breaking Change**)

### New Contributors

## [0.2.1]

### Added

### Fixed

- Fixed debug builds causing data aborts. (#67)

### Changed

### Removed

### New Contributors

## [0.2.0]

### Added

- Added `TICKS_PER_ROTATION` constant to `AdiEncoder` for use with `Position`.

### Fixed

- Removed unintentional re-exports from `vexide-core` program module. (**Breaking Change**)
- Fixed vision panicking after getting garbage data from vex-sdk.
- Corrected incorrect axis getters in the `Controller` API.

### Changed

### Removed

### New Contributors

[unreleased]: https://github.com/vexide/vexide/compare/v0.3.0...HEAD
[0.2.0]: https://github.com/vexide/vexide/compare/v0.1.0...v0.2.0
[0.2.1]: https://github.com/vexide/vexide/compare/v0.2.0...v0.2.1
[0.3.0]: https://github.com/vexide/vexide/compare/v0.2.1...v0.3.0
[0.4.0]: https://github.com/vexide/vexide/compare/v0.3.0...v0.4.0
[0.4.1]: https://github.com/vexide/vexide/compare/v0.4.0...v0.4.1
