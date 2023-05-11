extern crate alloc;
use core::ffi::c_void;

/// Represents a task
pub struct Task {
    task: pros_sys::task_t,
}

impl Task {
    /// Creates a task to be run 'asynchronously' (More information at the [FreeRTOS docs](https://www.freertos.org/taskandcr.html)).
    /// Takes in a closure that can move variables if needed.
    /// If your task has a loop it is advised to use [`sleep(duration)`](sleep) so that the task does not take up necessary system resources.
    /// Tasks should be long-living; starting many tasks can be slow and is usually not necessary.
    pub fn spawn<F: FnOnce() + Send>(
        function: F,
        priority: TaskPriority,
        stack_depth: TaskStackDepth,
        name: Option<&str>,
    ) -> Self {
        let mut entrypoint = TaskEntrypoint { function };
        let name = alloc::ffi::CString::new(name.unwrap_or(""))
            .unwrap()
            .into_raw();
        unsafe {
            let task = pros_sys::task_create(
                Some(TaskEntrypoint::<F>::cast_and_call_external),
                &mut entrypoint as *mut _ as *mut c_void,
                priority as _,
                stack_depth as _,
                name,
            );

            _ = alloc::ffi::CString::from_raw(name);
            Self { task }
        }
    }

    /// Pause execution of this task.
    /// This can have unintended consiquences if you are not carefull,
    /// for example, if this task is holding a mutex when paused, there is no way to retrieve it untill the task is unpaused.
    pub fn pause(&self) {
        unsafe {
            pros_sys::task_suspend(self.task);
        }
    }

    /// Enables the task for execution again.
    pub fn unpause(&self) {
        unsafe {
            pros_sys::task_resume(self.task);
        }
    }

    /// Sets the priority.
    pub fn set_priority(&self, priority: TaskPriority) {
        unsafe {
            pros_sys::task_set_priority(self.task, priority as _);
        }
    }

    /// Get the state of the task.
    pub fn state(&self) -> TaskState {
        unsafe { pros_sys::task_get_state(self.task).into() }
    }

    /// Send a notification to the task.
    pub fn notify(&self) {
        unsafe {
            pros_sys::task_notify(self.task);
        }
    }

    /// Waits for the task to finish, and then deletes it.
    pub fn join(self) {
        unsafe {
            pros_sys::task_join(self.task);
        }
    }

    /// Deletes the task and consumes it.
    pub fn delete(self) {
        unsafe {
            pros_sys::task_delete(self.task);
        }
    }
}

/// An ergonomic builder for tasks. Alternatively you can use [`Task::spawn`](crate::multitasking::Task::spawn).
pub struct TaskBuilder<'a, F: FnOnce() + Send> {
    name: Option<&'a str>,
    priority: Option<TaskPriority>,
    stack_depth: Option<TaskStackDepth>,
    function: F,
}

impl<'a, F: FnOnce() + Send> TaskBuilder<'a, F> {
    /// Creates a task builder.
    pub fn new(function: F) -> Self {
        Self {
            name: None,
            priority: None,
            stack_depth: None,
            function,
        }
    }

    /// Sets the name of the task, this is useful for debugging.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the priority of the task (how much time the scheduler gives to it.).
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets how large the stack for the task is.
    /// This can usually be set to defualt
    pub fn stack_depth(mut self, stack_depth: TaskStackDepth) -> Self {
        self.stack_depth = Some(stack_depth);
        self
    }

    /// Builds and spawns the task
    pub fn build(self) -> Task {
        Task::spawn(
            self.function,
            self.priority.unwrap_or_default(),
            self.stack_depth.unwrap_or_default(),
            self.name,
        )
    }
}

/// Represents the current state of a task.
pub enum TaskState {
    /// The task is running as normal
    Running,
    /// Unknown to me at the moment
    Ready,
    /// The task is blocked
    Blocked,
    /// The task is suspended
    Suspended,
    /// The task has been deleted
    Deleted,
    /// The tasks state is invalid somehow
    Invalid,
}

impl From<u32> for TaskState {
    fn from(value: u32) -> Self {
        match value {
            pros_sys::task_state_e_t_E_TASK_STATE_RUNNING => Self::Running,
            pros_sys::task_state_e_t_E_TASK_STATE_READY => Self::Ready,
            pros_sys::task_state_e_t_E_TASK_STATE_BLOCKED => Self::Blocked,
            pros_sys::task_state_e_t_E_TASK_STATE_SUSPENDED => Self::Suspended,
            pros_sys::task_state_e_t_E_TASK_STATE_DELETED => Self::Deleted,
            pros_sys::task_state_e_t_E_TASK_STATE_INVALID => Self::Invalid,
            _ => Self::Invalid,
        }
    }
}

/// Represents how much time the cpu should spend on this task.
/// (Otherwise known as the priority)
#[repr(u32)]
pub enum TaskPriority {
    High = 16,
    Default = 8,
    Low = 1,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Default
    }
}

/// Reprsents how large of a stack the task should get.
/// Tasks that don't have any or many variables and/or don't need floats can use the low stack depth option.
#[repr(u32)]
pub enum TaskStackDepth {
    Default = 8192,
    Low = 512,
}

impl Default for TaskStackDepth {
    fn default() -> Self {
        Self::Default
    }
}

struct TaskEntrypoint<F> {
    function: F,
}

impl<F> TaskEntrypoint<F>
where
    F: FnOnce(),
{
    unsafe extern "C" fn cast_and_call_external(this: *mut c_void) {
        let this = this.cast::<Self>().read();

        (this.function)()
    }
}

/// Sleeps the current task for the given amount of time.
/// Useful for when using loops in tasks.
pub fn sleep(duration: core::time::Duration) {
    unsafe { pros_sys::delay(duration.as_millis() as u32) }
}

/// Returns the task the function was called from.
pub fn get_current_task() -> Task {
    unsafe {
        Task {
            task: pros_sys::task_get_current(),
        }
    }
}

/// Gets the first notification in the queue.
/// If there is none, blocks untill a notification is recieved.
/// I am unsure what happens if the thread is unblocked while waiting.
/// returns the value of the notification
pub fn get_notification() -> u32 {
    unsafe { pros_sys::task_notify_take(false, pros_sys::TIMEOUT_MAX_CONST) }
}
