use vexide::prelude::*;

#[vexide::main(banner(enabled = false))]
async fn main(_peripherals: Peripherals) {

    println!("Hello");

    unsafe {
        core::ptr::write_volatile(0 as *mut u32, 0);
    }
}
