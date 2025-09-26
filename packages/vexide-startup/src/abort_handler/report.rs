use std::fmt::{self, Write};

use super::fault::{Fault, FaultException};

pub struct AbortWriter(());

impl AbortWriter {
    pub const BUFFER_SIZE: usize = 2048;
    pub const fn new() -> Self {
        Self(())
    }

    fn flush(&mut self) {
        unsafe {
            while (vex_sdk::vexSerialWriteFree(1) as usize) != Self::BUFFER_SIZE {
                vex_sdk::vexTasksRun();
            }
        }
    }
}

impl Write for AbortWriter {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let buf = s.as_bytes();

        for chunk in buf.chunks(Self::BUFFER_SIZE) {
            if unsafe { vex_sdk::vexSerialWriteFree(1) as usize } < chunk.len() {
                self.flush();
            }

            let count: usize =
                unsafe { vex_sdk::vexSerialWriteBuffer(1, chunk.as_ptr(), chunk.len() as u32) }
                    as _;

            if count != chunk.len() {
                break;
            }
        }

        Ok(())
    }
}

#[instruction_set(arm::a32)]
pub unsafe extern "aapcs" fn fault_exception_handler(fault: *const Fault) -> ! {
    unsafe {
        // how and why does this work
        core::arch::asm!("cpsie i");
    }

    let fault = unsafe { *fault };

    let fault_status = fault.load_status();

    let source = fault_status.source_description();
    let action = fault_status.action_description();

    let mut writer = AbortWriter::new();
    writer.flush();

    _ = writeln!(
        &mut writer,
        "\n{exception} Exception: {source}",
        exception = fault.exception
    );
    _ = writeln!(&mut writer, "    at address {:#x}", fault.program_counter);
    _ = writeln!(
        &mut writer,
        "    while {action} address {:#x}",
        fault.address()
    );

    _ = writeln!(&mut writer, "\nregisters at time of fault:");

    for (i, register) in fault.registers.iter().enumerate() {
        if i > 9 {
            _ = writeln!(writer, "r{i}: {:#x} ", register);
        } else {
            _ = writeln!(writer, " r{i}: {:#x} ", register);
        }
    }

    _ = writeln!(writer, " sp: {:#x} ", fault.stack_pointer);
    _ = writeln!(writer, " lr: {:#x} ", fault.link_register);
    _ = writeln!(writer, " pc: {:#x} ", fault.program_counter);

    _ = writeln!(
        &mut writer,
        "\nhelp: This indicates the misuse of `unsafe` code. Use a symbolizer tool to determine the location of the crash."
    );

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    _ = writeln!(
        &mut writer,
        "      (e.g. llvm-symbolizer -e ./target/armv7a-vex-v5/{profile}/program_name {:#x})",
        fault.program_counter
    );

    writer.flush();

    // #[cfg(feature = "backtrace")]
    // {
    //     let context = backtrace::make_unwind_context(backtrace::CoreRegisters {
    //         r: fault.registers,
    //         lr: fault.link_register,
    //         pc: fault.program_counter,
    //         sp: fault.stack_pointer,
    //     });
    //     _ = backtrace::print_backtrace(&mut writer, &context);
    // }

    match fault.exception {
        FaultException::DataAbort => unsafe {
            vex_sdk::vexSystemDataAbortInterrupt();
        },
        FaultException::PrefetchAbort => unsafe {
            vex_sdk::vexSystemPrefetchAbortInterrupt();
        },
        FaultException::UndefinedInstruction => unsafe {
            vex_sdk::vexSystemUndefinedException();
        },
    }

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
