# Hardware Self-Tests

This directory contains unit tests which run on VEX V5 hardware and help to verify vexide's compatibility with VEXos and the VEX SDK.

## Important Hardware Setup

These tests will attempt to activate connected hardware, so you should not run them with any actuators connected to your robot controller unless you are okay with devices moving in unpredictable ways.

## Running the Suite

Run `cargo test` in this directory with a VEX V5 Brain connected over USB to run the test suite.

```
cd self-tests
cargo test
```

The test suite will be uploaded and run from program slot 8 by default. This can be configured using the usual `cargo-v5` flags:

```
cargo test -- --slot 1
```

## Development

Test cases run on-device and should be placed in submodules of the `tests/cases/` directory and referenced from `tests/suite.rs`.
