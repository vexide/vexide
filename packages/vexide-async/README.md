# pros-async

Tiny async runtime and robot traits for `pros-rs`.
The async executor supports spawning tasks and blocking on futures.
It has a reactor to improve the performance of some futures.
FreeRTOS tasks can still be used, but it is recommended to use only async tasks for performance.
