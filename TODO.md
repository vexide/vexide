# Todo

This is the todo list for the eventuall 1.0.0 release of pros-rs

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
  * [ ] (Custom) Gear Ratios
* [ ] Async Runtime (ditch tasks)
* [X] Make Robot Functions Take Self
* [X] PID controllers
* [ ] Feedforward loops
* [ ] ADI
* [ ] Ext. ADI
* [ ] Sensors
  * [X] Distance
  * [X] GPS
  * [ ] Inertial (IMU)
  * [ ] Optical
  * [X] Rotational
  * [X] Vision
* [ ] Controllers
  * [X] Controller data 
  * [ ] Controller printing
* [X] Link
* [ ] Async
  * [X] Returning top level futures
  * [ ] Future impls for everything
  * [ ] Reactor
  * [ ] User API (block_on, join!, etc.)
* [ ] MPSC

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
