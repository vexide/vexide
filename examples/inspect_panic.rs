use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    #[cfg(target_os = "vexos")]
    print_dump();
}

#[cfg(target_os = "vexos")]
fn print_dump() {
    use vexide::startup::crash_dump::read_persistent_crash_dump;

    let Some(dump) = (unsafe { read_persistent_crash_dump() }) else {
        println!("No valid crash dump stored in persistent memory.");
        return;
    };

    let message = dump.payload.message.bytes();
    let msg_str = String::from_utf8_lossy(message);
    println!(" Crash Report");
    println!("==============\n");

    println!("{msg_str:?}");

    println!("\nBacktrace:");
    for frame in dump.payload.backtrace {
        println!("  {frame:x?}");
    }
}
