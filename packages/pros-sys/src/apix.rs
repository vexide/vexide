use core::ffi::*;

use crate::*;

pub type queue_t = *mut c_void;
pub type sem_t = *mut c_void;
/**
List of possible v5 devices

This list contains all current V5 Devices, and mirrors V5_DeviceType from the
api.
 */
pub type v5_device_e_t = u32;
pub const E_DEVICE_NONE: v5_device_e_t = 0;
pub const E_DEVICE_MOTOR: v5_device_e_t = 2;
pub const E_DEVICE_ROTATION: v5_device_e_t = 4;
pub const E_DEVICE_IMU: v5_device_e_t = 6;
pub const E_DEVICE_DISTANCE: v5_device_e_t = 7;
pub const E_DEVICE_RADIO: v5_device_e_t = 8;
pub const E_DEVICE_VISION: v5_device_e_t = 11;
pub const E_DEVICE_ADI: v5_device_e_t = 12;
pub const E_DEVICE_OPTICAL: v5_device_e_t = 16;
pub const E_DEVICE_GPS: v5_device_e_t = 20;
pub const E_DEVICE_SERIAL: v5_device_e_t = 129;
#[deprecated(note = "use E_DEVICE_SERIAL instead")]
pub const E_DEVICE_GENERIC: v5_device_e_t = E_DEVICE_SERIAL;
pub const E_DEVICE_UNDEFINED: v5_device_e_t = 255;
/**
Action macro to pass into serctl or fdctl that activates the stream
identifier.

When used with serctl, the extra argument must be the little endian
representation of the stream identifier (e.g. "sout" -> 0x74756f73)

Visit <https://pros.cs.purdue.edu/v5/tutorials/topical/filesystem.html#serial>
to learn more.
 */
pub const SERCTL_ACTIVATE: u32 = 10;
/**
Action macro to pass into serctl or fdctl that deactivates the stream
identifier.

When used with serctl, the extra argument must be the little endian
representation of the stream identifier (e.g. "sout" -> 0x74756f73)

Visit <https://pros.cs.purdue.edu/v5/tutorials/topical/filesystem.html#serial>
to learn more.
 */
pub const SERCTL_DEACTIVATE: u32 = 11;
/**
Action macro to pass into fdctl that enables blocking writes for the file

The extra argument is not used with this action, provide any value (e.g.
NULL) instead

Visit <https://pros.cs.purdue.edu/v5/tutorials/topical/filesystem.html#serial>
to learn more.
 */
pub const SERCTL_BLKWRITE: u32 = 12;
/**
Action macro to pass into fdctl that makes writes non-blocking for the file

The extra argument is not used with this action, provide any value (e.g.
NULL) instead

Visit <https://pros.cs.purdue.edu/v5/tutorials/topical/filesystem.html#serial>
to learn more.
 */
pub const SERCTL_NOBLKWRITE: u32 = 13;
/**
Action macro to pass into serctl that enables advanced stream multiplexing
capabilities

The extra argument is not used with this action, provide any value (e.g.
NULL) instead

Visit <https://pros.cs.purdue.edu/v5/tutorials/topical/filesystem.html#serial>
to learn more.
 */
pub const SERCTL_ENABLE_COBS: u32 = 14;
/**
Action macro to pass into serctl that disables advanced stream multiplexing
capabilities

The extra argument is not used with this action, provide any value (e.g.
NULL) instead

Visit <https://pros.cs.purdue.edu/v5/tutorials/topical/filesystem.html#serial>
to learn more.
 */
pub const SERCTL_DISABLE_COBS: u32 = 15;
/**
Action macro to check if there is data available from the Generic Serial
Device

The extra argument is not used with this action, provide any value (e.g.
NULL) instead
 */
pub const DEVCTL_FIONREAD: u32 = 16;
/**
Action macro to check if there is space available in the Generic Serial
Device's output buffer

The extra argument is not used with this action, provide any value (e.g.
NULL) instead
 */
pub const DEVCTL_FIONWRITE: u32 = 18;
/**
Action macro to set the Generic Serial Device's baudrate.

The extra argument is the baudrate.
 */
pub const DECTRL_SET_BAUDRATE: u32 = 17;

extern "C" {
    /**
    Unblocks a task in the Blocked state (e.g. waiting for a delay, on a
    semaphore, etc.).

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#abort_delay> for
    details.
     */
    pub fn task_abort_delay(task: task_t) -> bool;
    /**
    Notify a task when a target task is being deleted.

    This function will configure the PROS kernel to call
    task_notify_ext(task_to_notify, value, action, NULL) when target_task is
    deleted.


    \param target_task
                   The task being watched for deletion
    \param task_to_notify
           The task to notify when target_task is deleted
    \param value
                   The value to supply to task_notify_ext
    \param notify_action
                    The action to supply to task_notify_ext
    */
    pub fn task_notify_when_deleting(
        target_task: task_t,
        task_to_notify: task_t,
        value: u32,
        notify_action: notify_action_e_t,
    );
    /**
    Creates a recursive mutex which can be locked recursively by the owner.

    See
    <https://pros.cs.purdue.edu/v5/extended/multitasking.html#recursive_mutexes>
    for details.

    \return A newly created recursive mutex.
     */
    pub fn mutex_recursive_create() -> mutex_t;
    /**
    Takes a recursive mutex.

    See
    <https://pros.cs.purdue.edu/v5/extended/multitasking.html#recursive_mutexes>
    for details.

    \param mutex
           A mutex handle created by mutex_recursive_create
    \param wait_time
           Amount of time to wait before timing out

    \return 1 if the mutex was obtained, 0 otherwise
     */
    pub fn mutex_recursive_take(mutex: mutex_t, timeout: u32) -> bool;
    /**
    Gives a recursive mutex.

    See
    <https://pros.cs.purdue.edu/v5/extended/multitasking.html#recursive_mutexes>
    for details.

    \param mutex
           A mutex handle created by mutex_recursive_create

    \return 1 if the mutex was obtained, 0 otherwise
     */
    pub fn mutex_recursive_give(mutex: mutex_t) -> bool;
    /**
    Returns a handle to the current owner of a mutex.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#extra> for
    details.

    \param mutex
           A mutex handle

    \return A handle to the current task that owns the mutex, or NULL if the
    mutex isn't owned.
     */
    pub fn mutex_get_owner(mutex: mutex_t) -> task_t;
    /**
    Creates a counting sempahore.

    See <https://pros.cs.purdue.edu/v5/tutorials/multitasking.html#semaphores> for
    details.

    \param max_count
           The maximum count value that can be reached.
    \param init_count
           The initial count value assigned to the new semaphore.

    \return A newly created semaphore. If an error occurred, NULL will be
    returned and errno can be checked for hints as to why sem_create failed.
    */
    pub fn sem_create(max_count: u32, init_count: u32) -> sem_t;
    /**
    Deletes a semaphore (or binary semaphore)

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#semaphores> for
    details.

    \param sem
                  Semaphore to delete
    */
    pub fn sem_delete(sem: sem_t);
    /**
    Creates a binary semaphore.

    See
    <https://pros.cs.purdue.edu/v5/extended/multitasking#.htmlbinary_semaphores>
    for details.

    \return A newly created semaphore.
     */
    pub fn sem_binary_create() -> sem_t;
    /**
    Waits for the semaphore's value to be greater than 0. If the value is already
    greater than 0, this function immediately returns.

    See <https://pros.cs.purdue.edu/v5/tutorials/multitasking.html#semaphores> for
    details.

    \param sem
           Semaphore to wait on
    \param timeout
           Time to wait before the semaphore's becomes available. A timeout of 0
           can be used to poll the sempahore. TIMEOUT_MAX can be used to block
           indefinitely.

    \return True if the semaphore was successfully take, false otherwise. If
    false is returned, then errno is set with a hint about why the sempahore
    couldn't be taken.
     */
    pub fn sem_wait(sem: sem_t, timeout: u32) -> bool;
    /**
    Increments a semaphore's value.

    See <https://pros.cs.purdue.edu/v5/tutorials/multitasking.html#semaphores> for
    details.

    \param sem
           Semaphore to post

    \return True if the value was incremented, false otherwise. If false is
    returned, then errno is set with a hint about why the semaphore couldn't be
    taken.
     */
    pub fn sem_post(sem: sem_t) -> bool;
    /**
    Returns the current value of the semaphore.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#extra> for
    details.

    \param sem
           A semaphore handle

    \return The current value of the semaphore (e.g. the number of resources
    available)
     */
    pub fn sem_get_count(sem: sem_t) -> u32;
    /**
    Creates a queue.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param length
           The maximum number of items that the queue can contain.
    \param item_size
           The number of bytes each item in the queue will require.

    \return A handle to a newly created queue, or NULL if the queue cannot be
    created.
     */
    pub fn queue_create(length: u32, item_size: u32) -> queue_t;
    /**
    Posts an item to the front of a queue. The item is queued by copy, not by
    reference.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           The queue handle
    \param item
           A pointer to the item that will be placed on the queue.
    \param timeout
           Time to wait for space to become available. A timeout of 0 can be used
           to attempt to post without blocking. TIMEOUT_MAX can be used to block
           indefinitely.

    \return True if the item was preprended, false otherwise.
     */
    pub fn queue_prepend(queue: queue_t, item: *const c_void, timeout: u32) -> bool;
    /**
    Posts an item to the end of a queue. The item is queued by copy, not by
    reference.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           The queue handle
    \param item
           A pointer to the item that will be placed on the queue.
    \param timeout
           Time to wait for space to become available. A timeout of 0 can be used
           to attempt to post without blocking. TIMEOUT_MAX can be used to block
           indefinitely.

    \return True if the item was preprended, false otherwise.
     */
    pub fn queue_append(queue: queue_t, item: *const c_void, timeout: u32) -> bool;
    /**
    Receive an item from a queue without removing the item from the queue.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           The queue handle
    \param buffer
           Pointer to a buffer to which the received item will be copied
    \param timeout
           The maximum amount of time the task should block waiting for an item to receive should the queue be empty at
           the time of the call. TIMEOUT_MAX can be used to block indefinitely.

    \return True if an item was copied into the buffer, false otherwise.
     */
    pub fn queue_peek(queue: queue_t, buffer: *mut c_void, timeout: u32) -> bool;
    /**
    Receive an item from the queue.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           The queue handle
    \param buffer
           Pointer to a buffer to which the received item will be copied
    \param timeout
           The maximum amount of time the task should block
           waiting for an item to receive should the queue be empty at the time
           of the call. queue_recv() will return immediately if timeout
           is zero and the queue is empty.

    \return True if an item was copied into the buffer, false otherwise.
     */
    pub fn queue_recv(queue: queue_t, buffer: *mut c_void, timeout: u32) -> bool;
    /**
    Return the number of messages stored in a queue.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           The queue handle.

    \return The number of messages available in the queue.
     */
    pub fn queue_get_waiting(queue: queue_t) -> u32;
    /**
    Return the number of spaces left in a queue.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           The queue handle.

    \return The number of spaces available in the queue.
     */
    pub fn queue_get_available(queue: queue_t) -> u32;
    /**
    Delete a queue.

    See <https://pros.cs.purdue.edu/v5/extended/multitasking.html#queues> for
    details.

    \param queue
           Queue handle to delete
    */
    pub fn queue_delete(queue: queue_t);
    /**
    Resets a queue to an empty state

    \param queue
           Queue handle to reset
     */
    pub fn queue_reset(queue: queue_t);
    /**
    Registers a device in the given zero-indexed port

    Registers a device of the given type in the given port into the registry, if
    that type of device is detected to be plugged in to that port.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (0-20), or a
    a different device than specified is plugged in.
    EADDRINUSE - The port is already registered to another device.

    \param port
           The port number to register the device
    \param device
           The type of device to register

    \return 1 upon success, PROS_ERR upon failure
     */
    pub fn registry_bind_port(port: u8, device_type: v5_device_e_t) -> c_int;
    /**
    Deregisters a devices from the given zero-indexed port

    Removes the device registed in the given port, if there is one.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (0-20).

    \param port
           The port number to deregister

    \return 1 upon success, PROS_ERR upon failure
     */
    pub fn registry_unbind_port(port: u8) -> c_int;
    /*
    Returns the type of device registered to the zero-indexed port.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (0-20).

    \param port
           The V5 port number from 0-20

    \return The type of device that is registered into the port (NOT what is
    plugged in)
     */
    pub fn registry_get_bound_type(port: u8) -> v5_device_e_t;
    /**
    Returns the type of the device plugged into the zero-indexed port.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (0-20).

    \param port
           The V5 port number from 0-20

    \return The type of device that is plugged into the port (NOT what is
    registered)
     */
    pub fn registry_get_plugged_type(port: u8) -> v5_device_e_t;
    /**
    Control settings of the serial driver.

    \param action
                An action to perform on the serial driver. See the SERCTL_* macros for
                details on the different actions.
    \param extra_arg
                An argument to pass in based on the action
     */
    pub fn serctl(action: u32, extra_arg: *mut c_void) -> i32;
    /*
    Control settings of the microSD card driver.

    \param action
                An action to perform on the microSD card driver. See the USDCTL_* macros
         for details on the different actions.
    \param extra_arg
                   An argument to pass in based on the action
     */
    // Not yet implemented
    // pub fn usdctl(file: c_int, action: u32, extra_arg: *mut c_void) -> i32;
    /**
    Control settings of the way the file's driver treats the file

    \param file
                A valid file descriptor number
    \param action
                An action to perform on the file's driver. See the *CTL_* macros for
                details on the different actions. Note that the action passed in must
         match the correct driver (e.g. don't perform a SERCTL_* action on a
         microSD card file)
    \param extra_arg
                  An argument to pass in based on the action
     */
    pub fn fdctl(file: c_int, action: u32, extra_arg: *mut c_void) -> i32;

}
