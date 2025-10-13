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
- copy and paste the following sections to the top

## [Unreleased]

### Added

### Fixed

### Changed

### Removed

### New Contributors

- @new-contributor made their first contribution in #11!
-->

## [Unreleased]

### Added

- Added support for custom encoders with different resolutions when using `AdiEncoder`. (#328) (**Breaking Change**)
- Added the `AdiOpticalEncoder` for simplifying the creation of VEX optical encoders with the new `AdiEncoder` API. (#328) (**Breaking Change**)
- Added `Position::ZERO` constant. (#328)
- Added the ability to create/convert `Position` instances to/from gradians. (#328)
- Added several missing derived trait implementations for many device error types. (#331)
- Added support for task-local data storage using the new `task_local!` macro. This is closely modeled after `thread_local!`s in the standard library. (#333)
- Added the `AiVisionCode::iter`/`into_iter` methods for iterating over the available signature IDs stored in a color code. (#376).

### Fixed

- Fixed an issue with `Metadata::len` using the wrong condition. (#314)
- Fixed backwards assertion logic causing a panic in `AiVision::color` and `AiVision::set_color`. (#316)
- `vexide::startup::startup` no longer handles banner printing and no longer takes arguments. If you wish to print a banner without using `#[vexide::main]`, consider using `vexide::startup::banner::print` instead. (#313) (**Breaking Change**)
- Symbols within the internal implementation of the patcher's `memcpy` will no longer clash with some libc compiler intrinsics. This should only matter if are linking to C libraries. (#314)
- Fixed a signature validation problem in the original `VisionSensor`. (#319)
- Fixed `AdiDigital*::is_low` improperly returning `is_high` (#324)
- `Display::erase` now functions as expected instead of using the program flags' background color. (#350)

### Changed

- If a custom panic hook causes a panic itself, its error message will now be reported using the default panic hook instead of causing the program to abort. (#346)
- Renamed `File::tell` to `File::stream_position`, made Public and Infaliable. (#314)
- The `Position` type now stores encoder positions as a floating-point number rather than a fixed-point number. (#328) (**Breaking Change**)
- The `AdiGyro::yaw` returns an `f64` rather than a `Position` now to match the behavior of `InertialSensor` and friends. (#328) (**Breaking Change**)
- Renamed `RotationSensor::set_computation_interval` to `RotationSensor::set_data_interval`. (#329) (**Breaking Change**)
- Moved the `_boot` routine to a naked function. (#337)
- Programs must now opt-in to vexide's custom memory layout by specifying the linker flag `-Tvexide.ld`. (#355) (**Breaking Change**)
- `TouchEvent` now stores the location of the press in a `point: Point2<i16>` field rather than separate `x` and `y` `i16` fields. (#375) (**Breaking Change**)
- Feature-gated the `MotorTuningConstants` type behind the `dangerous-motor-tuning` feature. (#374) (**Breaking Change**)
- Renamed `{SerialPort, RadioLink}::available_write_bytes` to `{SerialPort, RadioLink}::write_capacity`. (#376) (**Breaking Change**)
- `Motor` methods now return `PortError` rather than `MotorError`, which has been removed. (#376) (**Breaking Change**)

### Removed

- Removed `stride` from `Display::draw_buffer`, fixing a buffer size validation error. If you wish to specify the stride, use `vex-sdk` directly instead. (#323) (**Breaking change**)
- `SmartPort` and `AdiPort` are no longer in `vexide::prelude`. (#376) (**Breaking Change**)
- Removed `AiVisionCode::colors`. Prefer using `AiVisionCode::iter`/`AiVisionCode::into_iter` instead. (#376) (**Breaking Change**)
- Removed `MotorError`. Motors now return `PortError` with the exception of `set_gearset`, which returns `SetGearsetError`. (#376) (**Breaking Change**)

### New Contributors

- GLS <<contact@glstudios.org>> made their first contribution in #314!
- @fibonacci61 made their first contribution in #333!

## [0.7.0]

### Added

- Added the `FsString` `PathBuf` types as mutable and owned equivalents to `FsStr` and `Path`. (#296)
- Added `read_dir`, `ReadDir`, and `DirEntry` to `vexide_core::fs` for directory reading support. (#296)
- Implemented `PartialOrd` for `Version`. (#288)
- Added `RadioLink::INTERNAL_BUFFER_SIZE` constant. (#293)
- `AiVisionSensor` is now re-exported through `vexide::devices`. (#302)
- Added a new `GpsSensor::set_offset` method that allows reconfiguring the GPS sensor's physical offset after creation. (#302)
- Added the `vexide::program::abort` method to match `std::process::abort`. Unlike `vexide::program::exit`, `abort` attempts to terminate the program as immediately as possible without doing any cleanup to the serial buffer. (#309)
- Added `Deref` implementation back to `LazyLock`, which will panic if lazy initialization is performed recursively. (#310)
- Added `DerefMut` implementation and `force_mut` functionto `LazyLock`. (#310)
- Added `Once::try_call_once` which will return an error if called from within itself rather than returning a future. (#310)

### Fixed

- Added a missing `Drop` implementation to `File` that will close and flush the file descriptor. (#295)
- Fixed an issue where printing large amounts of data to `Stdout` without ticking the executor would immediately exit the program. (#296)
- `StdoutRaw::flush` now flushes the outgoing serial buffer (#296)
- Fixed an issue with `AdiEncoder` potentially configuring the wrong port. (#301)
- Fixed flipped assert logic when writing to the controller screen. (#300)

### Changed

- `Controller::battery_capacity` now returns a float from 0.0 to 1.0 instead of an i32. (#286) (**Breaking Change**)
- `RadioLink::open` now panics if `id` is not a valid `CStr` rather than returning a `Result`. (#293) (**Breaking Change**)
- `SerialPort::open` now returns a `Future` that must be awaited before opening the port. (#293) (**Breaking Change**)
- Renamed `vexide::async_runtime` module to `vexide::runtime`. (#305) (**Breaking Change**)
- The `vexide::async_runtime::task` module is now a top-level `vexide::task` module. (#305) (**Breaking Change**)
- Merged `vexide::core::time` and `vexide::async_runtime::time` into a single `vexide::time` module. (#305) (**Breaking Change**)
- Moved `vexide::core` modules to the top-level `vexide` crate. For example, `vexide::core::fs` is now `vexide::fs`. (#305) (**Breaking Change**)
- `InertialSensor::calibrate`, `AdiGyroscope::calibrate`, `DynamicPeripherals` methods, `OpenOptions` methods, and `Text::align` are now callable in `const fn` context. (#308)
- `vexide::allocator` is no longer cfg-gated to `target_vendor = "vex"`. (#307)
- Refactored the GPS Sensor API to provide all functionality through a single struct. (#302) (**Breaking Change**)
- Renamed `LazyLock::get` back to `LazyLock::force`. (#310) (**Breaking Change**)
- The default `talc` allocator can now be removed by disabling the `allocator` feature, which is enabled by default. (#311) (**Breaking Change**)

### Removed

- Removed `SerialError::Port`. `SerialPort` methods can no longer return `PortError`. (#293) (**Breaking Change**)
- Removed the `vexide::macro` module. (#305) (**Breaking Change**)
- Removed the `vexide::core` module in favor of top-level modules of the `vexide` crate. (#305) (**Breaking Change**)
- Removed the `GpsImu` struct. This functionality is now provided entirely through `GpsSensor`. (#302) (**Breaking Change**)

### New Contributors

## [0.6.1]

### Added

### Fixed

- Fixed docs.rs build failures.
- Fixed outdated dependencies of vexide-graphics.

### Changed

### Removed

### New Contributors

## [0.6.0]

### Added

- Added functions to get VEXos version and uptime. (#278)
- Added a self-modifying memory patcher to `vexide_startup` that applies patches over the current program. This will be paired with `cargo-v5` changes to allow for much faster uploading.

### Fixed

- Fixed error handling for encoder port numbers. (#264)
- Fixed error handling for rangefinder port numbers. (#268)
- Fixed an internal issue regarding units with `Motor::set_position`.
- Fixed `File::seek` seeking to the wrong position when using `SeekFrom::End` with a negative offset. (#267)
- Fixed a rare issue with IMU calibration timing out at the start of some programs. (#275, #279)
- Recursive panics (panics that occur *within* `vexide_panic`'s handler) will now immediately abort rather than potentially causing a stack overflow. (#275)

### Changed

- Renamed `Once::is_complete` to `Once::is_completed` for consistency with the standard library. (#257) (**Breaking Change**)
- All `Position` methods are now usable in `const` context. (#254)
- Two-wire ADI devices (`AdiEncoder` and `AdiRangeFinder`) now take their ports as separate arguments instead of a tuple. (#271) (**Breaking Change**)
- `AdiEncoder` and `AdiRangeFinder` will now panic if invalid port pairings are passed rather than return a `Result`. (#271) (**Breaking Change**)
- `AdiDevice` is now const-generic over the number of ports used by the device. (#271) (**Breaking Change**)
- Replaced `AdiDevice::port_number` with `AdiDevice::port_numbers`. (#271) (**Breaking Change**)

### Removed

- Replaced `vexide_core::allocator::init_heap` with `vexide_core::allocator::claim`, which allows claiming uninitialized memory spans as heap space.
- The `Nul`, `InvalidLine`, and `InvalidColumn` `ControllerError` variants have been removed. These errors now cause panics. (#266) (**Breaking Change**)
- `DisplayError` has been removed and `Display::draw_buffer` now panics when given a buffer of invalid size. (#266) (**Breaking Change**)
- The `InvalidId` and `InvalidIdInCode` `AiVisionError` variants have been removed. These errors now cause panics. (#266) (**Breaking Change**)
- `VisionError::InvalidId` has been removed. Invalid signature IDs given to `VisionSensor` will now cause panics. (#266) (**Breaking Change**)
- The `lock` functions on `Stdin` and `Stdout` are now async. (#265) (**Breaking Change**)
- `Stdin` and `Stdout` instances can no longer be instantiated using struct initialization syntax. Prefer using `stdin()`/`stdout()`. (#281) (**Breaking Change**)

### Removed

- Removed the `Deref` implementation and `force` method on `LazyLock` to prevent deadlocks. use the async `LazyLock::get` instead. (#265) (**Breaking Change**)
- Removed the `Read` and `Write` implementations on `Stdin` and `Stdout` respectively to prevent deadlocks. (#265) (**Breaking Change**)
- Removed `EncoderError` and `RangeFinderError`. The respective devices now just return `PortError`. (#271) (**Breaking Change**)

### New Contributors

- @Saylar27 made their first contribution in #279!
- @ion908 made their first contribution in #278!

## [0.5.1]

### Added

### Fixed

- Fixed docs.rs build by updating to `vex-sdk` 0.26.0. No functional changes from 0.5.0.

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
- Added support for legacy ADI servos through the `AdiServo` API. (#241)
- Added support for the V5 AI Vision Sensor (#58)
- Added FOV constants to the Vision Sensor (#58)
- Added missing `Send` and `Sync` `impl`s for RwLock. (#239)
- Added the `Proportional` font family and support for fractional font scaling. (#248) (**Breaking Change**)
- Added `AdiDigitalOut::with_initial_state` to set the initial state of a digital output while creating it (#246)
- Added `Display::draw_text` to write `Text` to a `Display`. (#247)
- Added support for the legacy Yaw Rate Gyroscope through the `AdiGyroscope` struct. (#236)
- Added support for reading/writing to the Brain's SDCard slot using the `vexide::core::fs` module. (#22)

### Fixed

- `AdiAddrLed::set_pixel` will now correctly return an error if the device's ADI expander is disconnected. (#155)
- The `dbg!();` now works as expected when no arguments are supplied to it. (#175)
- `Motor::velocity` now correctly returns the estimated velocity instead of target velocity. (#184) (**Breaking Change**)
- Removed useless generics from `AdiAddrLed::new`. (#197) (**Breaking Change**)
- IMU calibration timeouts should no longer appear when the IMU is in working condition. (#212)
- Fixed an issue preventing ADI updates in fast loops. (#210)
- `Motor::status` can now actually return the `MotorStatus::BUSY` flag. (#211)
- Fixed a memory leak on every `RadioLink` construction. (#220)
- Fixed a panic in `RadioLink::open` that would occur if a program using a VEXlink radio was ran twice. (#243)
- Fixed a bug with IMU reset offsets being applied incorrectly. (#242)

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
- `GpsSensor::new` is now infallible and no longer returns a `Result`. (#240) (**Breaking Change**)
- `RadioLink::new` can now only fail on `NulError` and will not bail if a radio is disconnected. (#240) (**Breaking Change**)
- `RadioLink::unread_bytes` can now return a `LinkError::ReadError`. (#243)
- `RadioLink::is_linked` is now infallible. (#243) (**Breaking Change**)

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
- Removed `LinkError::Nul`. (#240) (**Breaking Change**)
- Removed `LinkError::Port`, because it was broken. VEXlink will no longer perform port validation. (#243) (**Breaking Change**)
- Removed the `TextSize` enum. Use the associated constants on the new `FontSize` struct instead. (#248) (**Breaking Change**)

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
- Removed the `vexide-graphics` crate and associated features (containing embedded-graphics and slint drivers) from the main vexide crate due to licensing concerns. These drivers will be available as crates licensed separately from the main `vexide` project. (#297) (**Breaking Change**)

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

[unreleased]: https://github.com/vexide/vexide/compare/v0.7.0...HEAD
[0.2.0]: https://github.com/vexide/vexide/compare/v0.1.0...v0.2.0
[0.2.1]: https://github.com/vexide/vexide/compare/v0.2.0...v0.2.1
[0.3.0]: https://github.com/vexide/vexide/compare/v0.2.1...v0.3.0
[0.4.0]: https://github.com/vexide/vexide/compare/v0.3.0...v0.4.0
[0.4.1]: https://github.com/vexide/vexide/compare/v0.4.0...v0.4.1
[0.4.2]: https://github.com/vexide/vexide/compare/v0.4.1...v0.4.2
[0.5.0]: https://github.com/vexide/vexide/compare/v0.4.2...v0.5.0
[0.5.1]: https://github.com/vexide/vexide/compare/v0.5.0...v0.5.1
[0.6.0]: https://github.com/vexide/vexide/compare/v0.5.1...v0.6.0
[0.6.1]: https://github.com/vexide/vexide/compare/v0.6.0...v0.6.1
[0.7.0]: https://github.com/vexide/vexide/compare/v0.6.1...v0.7.0
