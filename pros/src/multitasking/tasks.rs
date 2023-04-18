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
    pub fn spawn<F: FnOnce()>(
        function: F,
        priority: TaskPriority,
        stack_depth: TaskStackDepth,
        name: &str,
    ) -> Self {
        let mut entrypoint = TaskEntrypoint { function };

        unsafe {
            Self {
                task: pros_sys::task_create(
                    Some(TaskEntrypoint::<F>::cast_and_call_external),
                    &mut entrypoint as *mut _ as *mut c_void,
                    priority as _,
                    stack_depth as _,
                    name.as_ptr() as *const i8,
                ),
            }
        }
    }

    /// Deletes the task and consumes it.
    pub fn delete(self) {
        unsafe {
            pros_sys::task_delete(self.task);
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

/// Reprsents how large of a stack the task should get.
/// Tasks that don't have any or many variables and/or don't need floats can use the low stack depth option.
#[repr(u32)]
pub enum TaskStackDepth {
    Default = 8192,
    Low = 512,
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
