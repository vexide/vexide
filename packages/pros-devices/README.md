# pros-devices

Functionality for accessing hardware connected to the V5 brain.

## Overview

The V5 brain features 21 RJ9 4p4c connector ports (known as "Smart ports") for communicating with newer V5 peripherals, as well as six 3-wire ports with log-to-digital conversion capability for compatibility with legacy Cortex devices. This module provides access to both smart devices and ADI devices.

## Organization

- `smart` contains abstractions and types for smart port connected ices.
- `adi` contains abstractions for three wire ADI connected devices.
- `battery` provides functions for getting information about the battery.
- `controller` provides types for interacting with the V5 controller.
