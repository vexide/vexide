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

- `CompetitionSystem` and `CompetitionMode` structs for better retrieving information about the robot's competition state. (#38)
- `competition::system` method for retrieving the type of competition control the robot is connected to. (#38)
- New `From` implementation to convert `Quaternion` and `Euler` to their pros-sys equivalents. (#45)
- `pros::io` module for I/O related operations. (#30)
- Various types from the `no_std_io` have are re-exported from this module to provide missing functionality from `std`. (#30)
- Macros for printing to stdout (`println`, `print`, `eprintln`, etc...) (#30)

### Fixed

- Fixed competition state-related getters in the `pros::competition` module. (#38)
- Fixed error handling in IMU sensor bindings. (#37)
- Fixed errors in doctests and examples throughout the crate. (#37)

### Changed

- Overhauled the `competition` module with more straightforward getters for competition state. (#38) (**Breaking Change**)
- LLEMU-related macros have been prefixed with `llemu_` (e.g. `llemu_println`). (**Breaking Change**) (#30)
- Added `Debug`, `Copy`, and `Clone` derives for common structs (#37)

### Removed

- Removed several broken bindings in `pros_sys` relating to competition state. (#38) (**Breaking Change**)

## [0.6.0] - 2024-01-14

### Added

### Fixed

- GPS sensor `set_offset` function now returns a result. The relevant PROS C bindings have been fixed as well. (**Breaking change**)
- FreeRTOS task creation now does not garble data that the provided closure captured.
- Grammar in the feature request template has been fixed.
- Wasm build flags have been updated and fixed.

### Changed

- Panicking behavior has been improved so that spawned tasks will not panic the entire program.
- Panic messages are now improved and printed over the serial connection.
- `AsyncRobot` should now be implemented using the newly stabilized async trait syntax instead of the old `async_trait` attribute macro. (**Breaking change**)

### Removed

- A nonexistent runner for armv7a-vexos-eabi target has been removed from the cargo config.

## [0.5.0] - 2024-01-08

### Added

- Standard library like `Instant`s
- Optical sensor bindings.
- IMU sensor bindings.

### Fixed

- The async executor now does not starve the OS of cycles when unnecessary.

### Changed

- Updated readme with fixed grammar.

### Removed

## [0.4.0] - 2024-01-02

### Added

- Add methods to controller for checking individual buttons and axes.

### Fixed

### Changed

- Write doc comments for previously undocumented modules and functions.

### Removed

[unreleased]: https://github.com/pros-rs/pros-rs/compare/v0.6.0...HEAD
[0.4.0]: https://github.com/pros-rs/pros-rs/releases/tag/v0.4.0
[0.5.0]: https://github.com/pros-rs/pros-rs/compare/v0.4.0...v0.5.0
[0.6.0]: https://github.com/pros-rs/pros-rs/compare/v0.5.0...v0.6.0
