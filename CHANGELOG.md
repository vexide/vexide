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

### Fixed

### Changed

### Removed

### New Contributors

## [0.5.0]

### Added

- You can now detect controller release occurrences with `ButtonState::is_now_released`.
- Added support for 5.5W motors with a new constructor (`Motor::new_exp`) and four new getters (`Motor::max_voltage`, `Motor::motor_type`, `Motor::is_v5`, and `Motor::is_exp`) for `Motor`. (#167)
- Added support for the V5 Workcell Electromagnet smart device. (#176)
- The conditions upon which functions return errors are now documented. (#155).
- Implemented the `Copy` trait for `BannerTheme`.
- Added a getter that retrieves a `Controller`'s identifier. (#189)
- Added support for controllers in `DynamicPeripherals`. (#196)
- Added the ability to return Smart Ports, ADI ports, the display, and controllers to `DynamicPeripherals`. (#196)
- Added a `SmartDevice::UPDATE_INTERVAL` constant for all devices, representing the amount of time between data updates from a given device. (#199) (**Breaking Change**)
- Added a `toggle` method to `AdiDigitalOut` to toggle between level outputs (210).
- Added the `OpticalSensor::GESTURE_UPDATE_INTERVAL` (50mS) constant (#211).
- Added a `toggle` method to `AdiDigitalOut` to toggle between level outputs.
- Added a `SerialPort::set_baud_rate` method for the adjusting baudrate of a generic serial smartport after initialization. (#217)
- Added fields containing relevant failure information to several error types (#221) (**Breaking Change**)
- Added support for the power button in the `Controller` API. (#231) (**Breaking Change**)
- Added implementations of `Mul<i64>` and `Div<i64>` for `Position`, allowing
  for opaque scaling (#230)
- Added panic hook support comparable to the Rust standard library through `vexide::panic::set_hook` and `vexide::panic::take_hook` (#234)

### Fixed

- `AdiAddrLed::set_pixel` will now correctly return an error if the device's ADI expander is disconnected. (#155)
- The `dbg!();` now works as expected when no arguments are supplied to it. (#175)
- `Motor::velocity` now correctly returns the estimated velocity instead of target velocity. (#184) (**Breaking Change**)
- Removed useless generics from `AdiAddrLed::new`. (#197) (**Breaking Change**)
- IMU calibration timeouts should no longer appear when the IMU is in working condition. (#212)
- Fixed an issue preventing ADI updates in fast loops. (#210)
- `Motor::status` can now actually return the `MotorStatus::BUSY` flag. (#211)
- Fixed a memory leak on every `RadioLink` construction. (#220)
- Fixed a problem where `Sync` was incorrectly implemented for `Mutex` when it shouldn't have been (#238) (**Breaking Change**)

### Changed

- Controller state is now returned all at once to reduce error checking. (#152) (**Breaking Change**)
- Controller bumper naming scheme has been changed from `<left/right>_trigger_<1/2>` to `button_<r/l><1/2>`. (#204) (**Breaking Change**)
- `Button::was_pressed` has been renamed to `ButtonState::is_now_pressed`.
- `battery::capacity` now returns from 0.0-1.0 rather than 0-100.
- `battery::voltage` is now returned in volts rather than millivolts.
- `battery::current` is now returned in amps rather than milliamps.
- Changed the incorrect return types of `AdiSolenoid::is_open` and `AdiSolenoid::is_closed` from `LogicLevel` to `bool`. (#164) (**Breaking Change**)
- Renamed `Motor::MAX_VOLTAGE` to `Motor::V5_MAX_VOLTAGE` and added `Motor::EXP_MAX_VOLTAGE`. (#167) (**Breaking Change**)
- Moved the ability to convert Smart devices to `SmartPorts` out of the `SmartDevice` trait and into the devices themselves. (#171) (**Breaking Change**)
- Renamed `SmartDeviceType::Magnet` to `SmartDeviceType::Electromagnet`. (#176) (**Breaking Change**)
- Getters and constructors will now create warnings when their return values are not used. (#155)
- Renamed `OpticalSensor::rgb` to `OpticalSensor::color` and `OpticalSensor::raw` to `OpticalSensor::raw_color` (#179) (**Breaking Change**).
- Made the following functions infallible: `AdiAccelerometer::sensitivity`, `AdiAccelerometer::max_acceleration`, `AdiPotentiometer::potentiometer_type`, `AdiPotentiometer::max_angle`, `Motor::target`, and `RotationSensor::direction`. (#182) (**Breaking Change**)
- `OpticalSensor::led_brightness` now returns a number from `0.0` - `1.0` rather than a number from `1` - `100`. (#155) (**Breaking Change**)
- Renamed `Motor::update_profiled_velocity` to `Motor::set_profiled_velocity`. (#155) (**Breaking Change**)
- `Mutex` is now `?Sized`, matching the behavior of the standard library. (#202) (**Breaking Change**)
- Switched to the [`rgb`](https://crates.io/crates/rgb) for color storage. `vexide::devices::color` is now `vexide::devices::rgb` which re-exports the `Rgb` type. (#201) (**Breaking Change**)
- Renamed `AddrledError::Adi` to `AddrledError::Port`. (#203) (**Breaking Change**)
- Renamed `GpsImu::set_data_rate` to `GpsImu::set_data_interval`. (#199) (**Breaking Change**)
- Renamed `InertialSensor::set_data_rate` to `InertialSensor::set_data_interval`. (#199) (**Breaking Change**)
- Renamed `Motor::DATA_WRITE_INTERVAL` to `Motor::WRITE_INTERVAL`. (#199) (**Breaking Change**)
- Renamed `InertialSensor::accel` to `InertialSensor::acceleration` (#213) (**Breaking Change**)
- Renamed `GpsImu::accel` to `GpsImu::acceleration` (#211) (**Breaking Change**)
- `SerialPort::read_byte` now takes `&mut self`. (#215) (**Breaking Change**)
- `OpticalSensor::last_gesture` now returns an `Option<Gesture>` if no gesture was detected. (#215) (**Breaking Change**)
- The `time` field on `Gesture` is now returned as an instance of `SmartDeviceTimestamp`. (#215) (**Breaking Change**)
- `Gesture` and `GestureDirection` no longer implements `Default`. (#215) (**Breaking Change**)
- Renamed `vexide::devices::geometry` to `vexide::devices::math`. (#218) (**Breaking Change**)
- Replaced the custom `Point2` type with `mint`'s `Point2` type for better interop. (#218) (**Breaking Change**)
- `SmartPort::device_type` now returns an `Option<SmartDeviceType>` which returns `None` if no device is connected or configured to a port. (#219) (**Breaking Change**)
- Renamed the `LinkError::NonTerminatingNul` and `ControllerError::NonTerminatingNul` variants to simply `Nul` and added a source error. (#220) (**Breaking Change**)
- Made `ControllerScreen` methods and `Controller::rumble` asynchronous and added synchronous `try_<action>` variants. (#222) (**Breaking Change**)
- Renamed `ControllerScreen::MAX_LINE_LENGTH` to `ControllerScreen::MAX_COLUMNS`. (#222) (**Breaking Change**)
- Refactored `InertialCalibrateFuture` to an opaque wrapper over the internal state machine. (#225) (**Breaking Change**)

### Removed

- Removed `Motor::DATA_READ_INTERVAL`. Use `Motor::UPDATE_INTERVAL` instead. (#199) (**Breaking Change**)
- Removed `InertialSensor::CALIBRATION_TIMEOUT` and replaced it with the `InertialSensor::CALIBRATION_START_TIMEOUT` and `InertialSensor::CALIBRATION_START_TIMEOUT` constants. (#212) (**Breaking Change**)
- `AdiDigitalOut::level` now reads the actual reported level value from VEXos, and thus now returns a `Result`. (#210) (**Breaking Change**)
- Removed the defunct `usd` module from `vexide::devices`. (#198) (**Breaking Change**)
- Removed `AdiSolenoid`. Use `AdiDigitalOut` instead. (#210) (**Breaking Change**)
- Removed the deprecated `ZERO_POSITION` and `ZERO_VELOCITY` `Motor` status flags. (#211) (**Breaking Change**)
- `GestureDirection::None` has been removed, as `OpticalSensor::next_gesture` now returns an `Option<Gesture>`. (#215) (**Breaking Change**)
- `GestureDirection` no longer has a `From` conversion for `u32`. (#215) (**Breaking Change**)
- Removed the `nalgebra` feature. All math types should natively support nalgebra conversions without any additional features. (#218) (**Breaking Change**)
- Removed `SmartDeviceType::None`. `SmartPort::device_type` now returns an `Option<SmartDeviceType>` which serves the same purpose. (#219) (**Breaking Change**)
- Removed `Position`-to-`Position` `Mul`/`Div` ops, as they were mathematically unsound. Prefer using `Position`-to-scalar operations for this. (#237) (**Breaking Change**)

### New Contributors

@zabackary made their first contribution in #164!


## [0.4.2]

### Added

### Fixed

- Fixed an issue related to the calling convention of vex-sdk functions causing docs.rs api reference build failures. (#165)

### Changed

### Removed

### New Contributors

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
- Renamed `Screen` and its associated structs to `Display`. (#138) (**Breaking Change**)
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

[unreleased]: https://github.com/vexide/vexide/compare/v0.5.0...HEAD
[0.2.0]: https://github.com/vexide/vexide/compare/v0.1.0...v0.2.0
[0.2.1]: https://github.com/vexide/vexide/compare/v0.2.0...v0.2.1
[0.3.0]: https://github.com/vexide/vexide/compare/v0.2.1...v0.3.0
[0.4.0]: https://github.com/vexide/vexide/compare/v0.3.0...v0.4.0
[0.4.1]: https://github.com/vexide/vexide/compare/v0.4.0...v0.4.1
[0.4.2]: https://github.com/vexide/vexide/compare/v0.4.1...v0.4.2
[0.5.0]: https://github.com/vexide/vexide/compare/v0.4.2...v0.5.0
