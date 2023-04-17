use core::ffi::c_void;
use pros_sys;

pub fn spawn(task: fn(params: *mut c_void), params: *mut c_void, priority: TaskPriority, stack_depth: TaskStackDepth, name: &str) {
    unsafe {
        pros_sys::task_create(Some(task as extern "C" fn(params: *mut c_void)), params, priority as _, stack_depth as _, name.as_ptr() as *const i8);
    }
}

#[repr(u32)]
pub enum TaskPriority {
    High = 16,
    Default = 8,
    Low = 1,
}

#[repr(u32)]
pub enum TaskStackDepth {
    Default = 8192,
    Low = 512,
}