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

- `CompetitionSystem` and `CompetitionMode` structs for better retrieving information about the robot's competition state. (#38)
- `competition::system` method for retrieving the type of competition control the robot is connected to. (#38)

### Fixed

- Fixed competition state-related getters in the `pros::competition` module. (#38)

### Changed

- Add contributing information, pull request templates, and changelog.
- Overhauled the `competition` module with more straightforward getters for competition state. (#38) (**Breaking Change**)

### Removed

- Removed several broken bindings in `pros_sys` relating to competition state. (#38) (**Breaking Change**)

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
