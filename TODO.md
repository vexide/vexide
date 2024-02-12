# Todo

This is the todo list for the eventual 1.0.0 release of pros-rs

## Bindings

* [X] Basic LCD Printing.
* [X] Buttons
  * [X] Pressed buttons
  * [X] Button press callback functions
* [X] Multitasking
  * [X] Mutexes
  * [X] Tasks
  * [X] Notifications
* [ ] Motors
  * [x] Internal gearsets
  * [ ] (Custom) Gear Ratios
* [X] Make Robot Functions Take Self
* [X] PID controllers
* [X] Feedforward loops
* [ ] ADI (3 wire ports)
* [ ] Ext. ADI
* [X] Sensors
  * [X] Distance
  * [X] GPS
  * [x] Inertial (IMU)
  * [x] Optical
  * [X] Rotational
  * [X] Vision
* [X] Controllers
  * [X] Controller data
  * [x] Controller printing
* [X] Link
* [X] Async runtime
  * [X] Returning top level futures
  * [X] Reactor
* [ ] More asynchronous APIs
* [ ] MPSC
* [X] Task Locals

## API

* [X] Make more ergonomic start functions. (macros?)
* [X] Consider hiding task priority and stack depth from task API.

## Docs

* [X] Guides for building
  * [X] Windows
  * [X] Linux
  * [X] MacOS
* [ ] Examples in docs and readme
* [ ] More comprehensive documentation in general

## Non essential

* [ ] Drivetrain
* [ ] Xapi bindings
  * [ ] LVGL bindings
  * [X] Serial bindings (pros-sys)
