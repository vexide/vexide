#![no_main]
#![no_std]

#[vex_startup::main]
async fn main() {
    unsafe {
        // Write something to the screen to test if the program is running
        vex_sdk::vexDisplayRectFill(0, 0, 200, 200);
        loop {}
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
