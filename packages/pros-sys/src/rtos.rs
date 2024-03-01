pub const TASK_PRIORITY_MAX: u32 = 16;
pub const TASK_PRIORITY_MIN: u32 = 1;
pub const TASK_PRIORITY_DEFAULT: u32 = 8;
pub const TASK_STACK_DEPTH_DEFAULT: core::ffi::c_uint = 0x2000;
pub const TASK_STACK_DEPTH_MIN: core::ffi::c_uint = 0x200;
pub const TASK_NAME_MAX_LEN: core::ffi::c_uint = 32;
pub const TIMEOUT_MAX: u32 = u32::MAX;

pub const E_TASK_STATE_RUNNING: core::ffi::c_uint = 0;
pub const E_TASK_STATE_READY: core::ffi::c_uint = 1;
pub const E_TASK_STATE_BLOCKED: core::ffi::c_uint = 2;
pub const E_TASK_STATE_SUSPENDED: core::ffi::c_uint = 3;
pub const E_TASK_STATE_DELETED: core::ffi::c_uint = 4;
pub const E_TASK_STATE_INVALID: core::ffi::c_uint = 5;
pub type task_state_e_t = core::ffi::c_uint;

pub const E_NOTIFY_ACTION_NONE: core::ffi::c_uint = 0;
pub const E_NOTIFY_ACTION_BITS: core::ffi::c_uint = 1;
pub const E_NOTIFY_ACTION_INCR: core::ffi::c_uint = 2;
pub const E_NOTIFY_ACTION_OWRITE: core::ffi::c_uint = 3;
pub const E_NOTIFY_ACTION_NO_OWRITE: core::ffi::c_uint = 4;
pub type notify_action_e_t = core::ffi::c_uint;

pub type task_t = *const core::ffi::c_void;
pub type task_fn_t = Option<unsafe extern "C" fn(arg1: *mut ::core::ffi::c_void)>;
pub type mutex_t = *const core::ffi::c_void;

const CURRENT_TASK: task_t = core::ptr::null();

extern "C" {
    /** Gets the number of milliseconds since PROS initialized.

    \return The number of milliseconds since PROS initialized*/
    pub fn millis() -> u32;
    /** Gets the number of microseconds since PROS initialized,

    \return The number of microseconds since PROS initialized*/
    pub fn micros() -> u64;
    /** Creates a new task and add it to the list of tasks that are ready to run.

    This function uses the following values of errno when an error state is
    reached:
    ENOMEM - The stack cannot be used as the TCB was not created.

    \param function
    Pointer to the task entry function
    \param parameters
    Pointer to memory that will be used as a parameter for the task being
    created. This memory should not typically come from stack, but rather
    from dynamically (i.e., malloc'd) or statically allocated memory.
    \param prio
    The priority at which the task should run.
    TASK_PRIO_DEFAULT plus/minus 1 or 2 is typically used.
    \param stack_depth
    The number of words (i.e. 4 * stack_depth) available on the task's
    stack. TASK_STACK_DEPTH_DEFAULT is typically sufficienct.
    \param name
    A descriptive name for the task.  This is mainly used to facilitate
    debugging. The name may be up to 32 characters long.

    \return A handle by which the newly created task can be referenced. If an
    error occurred, NULL will be returned and errno can be checked for hints as
    to why task_create failed.*/
    pub fn task_create(
        function: task_fn_t,
        parameters: *const core::ffi::c_void,
        prio: u32,
        stack_depth: u16,
        name: *const core::ffi::c_char,
    ) -> task_t;
    /** Removes a task from the RTOS real time kernel's management. The task being
    deleted will be removed from all ready, blocked, suspended and event lists.

    Memory dynamically allocated by the task is not automatically freed, and
    should be freed before the task is deleted.

    \param task
    The handle of the task to be deleted.  Passing NULL will cause the
    calling task to be deleted.*/
    pub fn task_delete(task: task_t);
    /** Delays a task for a given number of milliseconds.

    This is not the best method to have a task execute code at predefined
    intervals, as the delay time is measured from when the delay is requested.
    To delay cyclically, use task_delay_until().

    \param milliseconds
    The number of milliseconds to wait (1000 milliseconds per second)*/
    pub fn task_delay(milliseconds: u32);
    /** Delays a task for a given number of milliseconds.

    This is not the best method to have a task execute code at predefined
    intervals, as the delay time is measured from when the delay is requested.
    To delay cyclically, use task_delay_until().

    \param milliseconds
    The number of milliseconds to wait (1000 milliseconds per second)*/
    pub fn delay(milliseconds: u32);
    /** Delays a task until a specified time.  This function can be used by periodic
    tasks to ensure a constant execution frequency.

    The task will be woken up at the time *prev_time + delta, and *prev_time will
    be updated to reflect the time at which the task will unblock.

    \param prev_time
    A pointer to the location storing the setpoint time. This should
    typically be initialized to the return value of millis().
    \param delta
    The number of milliseconds to wait (1000 milliseconds per second)*/
    pub fn task_delay_until(prev_time: *const u32, delta: u32);
    /** Gets the priority of the specified task.

    \param task
    The task to check

    \return The priority of the task*/
    pub fn task_get_priority(task: task_t) -> u32;
    /** Sets the priority of the specified task.

    If the specified task's state is available to be scheduled (e.g. not blocked)
    and new priority is higher than the currently running task, a context switch
    may occur.

    \param task
    The task to set
    \param prio
    The new priority of the task*/
    pub fn task_set_priority(task: task_t, prio: u32);
    /** Gets the state of the specified task.

    \param task
    The task to check

    \return The state of the task*/
    pub fn task_get_state(task: task_t) -> task_state_e_t;
    /** Suspends the specified task, making it ineligible to be scheduled.

    \param task
    The task to suspend*/
    pub fn task_suspend(task: task_t);
    /** Resumes the specified task, making it eligible to be scheduled.

    \param task
    The task to resume*/
    pub fn task_resume(task: task_t);
    /** Gets the number of tasks the kernel is currently managing, including all
    ready, blocked, or suspended tasks. A task that has been deleted, but not yet
    reaped by the idle task will also be included in the count. Tasks recently
    created may take one context switch to be counted.

    \return The number of tasks that are currently being managed by the kernel.*/
    pub fn task_get_count() -> u32;
    /** Gets the name of the specified task.

    \param task
    The task to check

    \return A pointer to the name of the task*/
    pub fn task_get_name(task: task_t) -> *const core::ffi::c_char;
    /** Gets a task handle from the specified name

    The operation takes a relatively long time and should be used sparingly.

    \param name
    The name to query

    \return A task handle with a matching name, or NULL if none were found.*/
    pub fn task_get_by_name(name: *const core::ffi::c_char) -> task_t;
    /** Get the currently running task handle. This could be useful if a task
    wants to tell another task about itself.

    \return The currently running task handle.*/
    pub fn task_get_current() -> task_t;
    /** Sends a simple notification to task and increments the notification counter.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/notifications.html> for
    details.

    \param task
    The task to notify

    \return Always returns true.*/
    pub fn task_notify(task: task_t) -> u32;
    /** Utilizes task notifications to wait until specified task is complete and deleted,
    then continues to execute the program. Analogous to std::thread::join in C++.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/notifications.html> for
    details.

    \param task
    The task to wait on.

    \return void*/
    pub fn task_join(task: task_t);
    /** Sends a notification to a task, optionally performing some action. Will also
    retrieve the value of the notification in the target task before modifying
    the notification value.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/notifications.html> for
    details.

    \param task
    The task to notify
    \param value
    The value used in performing the action
    \param action
    An action to optionally perform on the receiving task's notification
    value
    \param prev_value
    A pointer to store the previous value of the target task's
    notification, may be NULL

    \return Dependent on the notification action.
    For NOTIFY_ACTION_NO_WRITE: return 0 if the value could be written without
    needing to overwrite, 1 otherwise.
    For all other NOTIFY_ACTION values: always return 0*/
    pub fn task_notify_ext(
        task: task_t,
        value: u32,
        action: notify_action_e_t,
        prev_value: *const u32,
    ) -> u32;
    /** Waits for a notification to be nonzero.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/notifications.html> for
    details.

    \param clear_on_exit
    If true (1), then the notification value is cleared.
    If false (0), then the notification value is decremented.
    \param timeout
    Specifies the amount of time to be spent waiting for a notification
    to occur.

    \return The value of the task's notification value before it is decremented
    or cleared*/
    pub fn task_notify_take(clear_on_exit: bool, timeout: u32) -> u32;
    /** Clears the notification for a task.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/notifications.html> for
    details.

    \param task
    The task to clear

    \return False if there was not a notification waiting, true if there was*/
    pub fn task_notify_clear(task: task_t) -> bool;
    /** Creates a mutex.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/multitasking.html#mutexes>
    for details.

    \return A handle to a newly created mutex. If an error occurred, NULL will be
    returned and errno can be checked for hints as to why mutex_create failed.*/
    pub fn mutex_create() -> mutex_t;
    /** Takes and locks a mutex, waiting for up to a certain number of milliseconds
    before timing out.

    See <https://pros.cs.purdue.edu/v5/tutorials/topical/multitasking.html#mutexes>
    for details.

    \param mutex
    Mutex to attempt to lock.
    \param timeout
    Time to wait before the mutex becomes available. A timeout of 0 can
    be used to poll the mutex. TIMEOUT_MAX can be used to block
    indefinitely.

    \return True if the mutex was successfully taken, false otherwise. If false
    is returned, then errno is set with a hint about why the the mutex
    couldn't be taken.*/
    pub fn mutex_take(mutex: mutex_t, timeout: u32) -> bool;
    /** Deletes a mutex

    \param mutex
    Mutex to unlock.*/
    pub fn mutex_give(mutex: mutex_t) -> bool;
    /** Deletes a mutex

    \param mutex
    Mutex to unlock.*/
    pub fn mutex_delete(mutex: mutex_t);

    /** Sets a value in a task's thread local storage array.

    This function is intended for advanced users only.

    Parameters:
        xTaskToSet  The handle of the task to which the thread local data is being written. A task can write to its own thread local data by using NULL as the parameter value.
        xIndex  The index into the thread local storage array to which data is being written.

        The number of available array indexes is set by the configNUM_THREAD_LOCAL_STORAGE_POINTERS compile time configuration constant in FreeRTOSConfig.h.
        pvValue  The value to write into the index specified by the xIndex parameter.

    Example usage:

    See the examples provided on the thread local storage array documentation page. */
    pub fn vTaskSetThreadLocalStoragePointer(
        xTaskToSet: task_t,
        xIndex: i32,
        pvValue: *const core::ffi::c_void,
    );

    /** Retrieves a value from a task's thread local storage array.

    This function is intended for advanced users only.

    Parameters:
        xTaskToQuery  The handle of the task from which the thread local data is being read. A task can read its own thread local data by using NULL as the parameter value.
        xIndex  The index into the thread local storage array from which data is being read.

        The number of available array indexes is set by the configNUM_THREAD_LOCAL_STORAGE_POINTERS compile time configuration constant in FreeRTOSConfig.h.

    Returns:
        The values stored in index position xIndex of the thread local storage array of task xTaskToQuery.

    Example usage:

        See the examples provided on the thread local storage array documentation page. */
    pub fn pvTaskGetThreadLocalStoragePointer(
        xTaskToQuery: task_t,
        xIndex: i32,
    ) -> *const core::ffi::c_void;

    /// Suspends the scheduler.  Suspending the scheduler prevents a context switch from occurring but leaves interrupts enabled.  If an interrupt requests a context switch while the scheduler is suspended, then the request is held pending and is performed only when the scheduler is resumed (un-suspended).
    ///
    ///
    /// Calls to xTaskResumeAll() transition the scheduler out of the Suspended state following a previous call to vTaskSuspendAll().
    ///
    ///
    /// Calls to vTaskSuspendAll() can be nested.  The same number of calls must be made to xTaskResumeAll() as have previously been made to vTaskSuspendAll() before the scheduler will leave the Suspended state and re-enter the Active state.
    ///
    ///
    /// xTaskResumeAll() must only be called from an executing task and therefore must not be called while the scheduler is in the Initialization state (prior to the scheduler being started).
    ///
    ///
    /// Other FreeRTOS API functions must not be called while the scheduler is suspended.
    ///
    /// API functions that have the potential to cause a context switch (for example, vTaskDelayUntil(), xQueueSend(), etc.) must not be called while the
    /// scheduler is suspended.
    pub fn rtos_suspend_all();

    /// Resumes the scheduler after it was suspended using a call to vTaskSuspendAll().
    ///
    ///
    /// xTaskResumeAll() only resumes the scheduler.  It does not unsuspend tasks
    /// that were previously suspended by a call to vTaskSuspend().
    pub fn rtos_resume_all() -> i32;
}
