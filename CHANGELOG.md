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

- Added support for the V5 GPS Sensor (#79)

### Fixed

- Fixed an issue where the distance sensor relative_size returned a u32 when it can be negative. (#116)

### Changed

- Overhauled the design of the startup banner.
- `DistanceSensor::distance` now returns an `Option` that will be `None` if the sensor is out of range. (#113) (**Breaking Change**)
- Adjusted distance sensor error names. (#113) (**Breaking Change**)
- Renamed `SmartDevice::port_index` and `SmartPort::index` to `SmartDevice::port_number` and `SmartPort::port_number`. (#121) (**Breaking Change**)
- Renamed `AdiDevice::port_index` and `AdiPort::index` to `AdiDevice::port_number` and `AdiDevice::port_number`. (#121) (**Breaking Change**)
- `SmartPort::device_type` now no longer returns a `Result`. (#121) (**Breaking Change**)
- Marks many futures as `#[must_use]` to warn when futures are created without `await`ing them. (#112)

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
