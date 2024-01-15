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
-->

## [Unreleased]

### Added

- `SmartPort` struct for device access. (#34)
- `SmartDevice` trait for common functionality across smart port devices. (#34)
- Methods to get a device's port number as well as determine if the device is plugged in or not. (#34)
- Added various missing derives for hardware-related data structures. (#34)

### Fixed

- Inertial sensor (`imu.rs`) example now compiles (#34).

### Changed

- Add contributing information, pull request templates, and changelog. (#34)
- `AdiPort` is now structured with ADI expander modules in mind. (**Breaking change**) (#34)
- Reorganized ADI, Smart Port, and builtin devices into a common `devices` module. (**Breaking change**) (#34)
- Devices now take `SmartPort`/`AdiPort` rather than a raw port index. (**Breaking change**) (#34)
- All devices now take `&mut self` for methods modifying hardware state. (**Breaking change**) (#34)

### Removed

- `Copy`/`Clone` derives for some existing device types. (**Breaking change**) (#34)


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

[unreleased]: https://github.com/pros-rs/pros-rs/compare/v0.5.0...HEAD
[0.4.0]: https://github.com/pros-rs/pros-rs/releases/tag/v0.4.0
[0.5.0]: https://github.com/pros-rs/pros-rs/compare/v0.4.0...v0.5.0
