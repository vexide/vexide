use core::{
    arch::{asm, global_asm, naked_asm},
    ffi::c_void,
};

// Custom ARM vector table. Pointing the VBAR coproc. register at this
// will configure the CPU to jump to these functions on an exception.
global_asm!(
    r#"
.section .vectors, "ax"
.global vectors
.arm

vectors:
    b {boot}
    b {undefined_instruction}
    b {supervisor_call}
    b {prefetch_abort}
    b {data_abort}
    nop @ Placeholder for address exception vector
    b {irq}
    @ `b fiq` would go here, but the beginning of the FIQ
    @ handler is in the same place so there's no need.

fiq:
    push {{r0-r3,r12,lr}} @ Preserve AAPCS callee-saved registers

	vpush {{d0-d7}}       @ Preserve hard-float registers
	vpush {{d16-d31}}
	vmrs r1, fpscr
	push {{r1}}
	vmrs r1, fpexc
	push {{r1}}

    blx	{fiq_handler}

    pop {{r0-r3,r12,lr}}  @ Restore state

	pop 	{{r1}}
	vmsr    fpexc, r1
	pop 	{{r1}}
	vmsr    fpscr, r1
	vpop    {{d16-d31}}
	vpop    {{d0-d7}}

    subs pc, lr, #4       @ Return to previous code (LR is offset by the
                          @ size of a single instruction)
    "#,
    boot = sym boot,
    undefined_instruction = sym undefined_instruction,
    supervisor_call = sym supervisor_call,
    prefetch_abort = sym prefetch_abort,
    data_abort = sym data_abort,
    irq = sym irq,
    fiq_handler = sym fiq_handler,
);

/// Enable vexide's CPU exception handling logic by installing
/// its custom vector table. (Temporary internal API)
pub fn install_vector_table() {
    unsafe {
        asm!(
            "ldr r0, =vectors",
            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            // Ensure the write takes effect before any exception occurs
            // in the pipeline.
            "dsb",
            "isb",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn boot() -> ! {
    unsafe {
        vex_sdk::vexSystemBoot();

        loop {
            vex_sdk::vexTasksRun();
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum FaultType {
    UndefinedInstruction = 0,
    SupervisorCall = 1,
    PrefetchAbort = 2,
    DataAbort = 3,
}

#[repr(C)]
struct Fault {
    stack_pointer: u32,
    cause: FaultType,
    /// The address at which the fault occurred.
    ///
    /// This is calculated using the Link Register (`lr`), which is set to this address
    /// plus an offset when an exception occurs.
    ///
    /// Offsets:
    ///
    /// * [plus 8 bytes][da-exception] for data aborts
    /// * [plus 4 bytes][pf-exception] for prefetch aborts
    /// * [plus the size of an instruction][svc-exception] for SVCs and
    ///   undefined instruction aborts (this is different in thumb mode)
    ///
    /// [da-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Data-Abort-exception
    /// [pf-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Prefetch-Abort-exception
    /// [svc-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Supervisor-Call--SVC--exception
    instruction_address: u32,
    /// Registers r0 through r12
    registers: [u32; 13],
}

#[expect(edition_2024_expr_fragment_specifier)]
macro_rules! abort_handler {
    (
        $name:ident:
        lr_offset = $lr_offset:expr,
        cause = $cause:expr$(,)?
    ) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        #[instruction_set(arm::a32)]
        unsafe extern "C" fn $name() -> ! {
            naked_asm!(
                "
                dsb
                stmdb	sp!,{{r0-r3,r12,lr}}
                mov r0, {cause}
                blx	{exception_handler}
                ldmia	sp!,{{r0-r3,r12,lr}}
                subs	pc, lr, #{lr_offset}
                ",
                exception_handler = sym exception_handler,
                lr_offset = const $lr_offset,
                cause = const $cause as u32,
            );
        }
    };
}

abort_handler!(undefined_instruction: lr_offset = 4, cause = FaultType::UndefinedInstruction);
abort_handler!(supervisor_call: lr_offset = 4, cause = FaultType::SupervisorCall);
abort_handler!(prefetch_abort: lr_offset = 4, cause = FaultType::PrefetchAbort);
abort_handler!(data_abort: lr_offset = 8, cause = FaultType::DataAbort);

/// MMIO interface for the Generic Interrupt Controller v1
///
/// GICv1 docs: <https://documentation-service.arm.com/static/5f8ff151f86e16515cdbf95b>
mod interrupt_controller {
    /// The base address of the GIC's [CPU interface] on Zynq-7000 SOCs (e.g. VEX V5).
    ///
    /// Source: <https://docs.amd.com/r/en-US/ug585-zynq-7000-SoC-TRM/CPU-Private-Bus-Registers>
    ///
    /// [CPU interface]: <https://developer.arm.com/documentation/ihi0048/a/GIC-Partitioning/About-GIC-partitioning?lang=en>
    pub const CPU_INTERFACE_BASE_ADDRESS: usize = 0xF8F0_0100;

    /// The base address of the GIC's [distributor] on Zynq-7000 SOCs (e.g. VEX V5).
    ///
    /// Source: <https://docs.amd.com/r/en-US/ug585-zynq-7000-SoC-TRM/CPU-Private-Bus-Registers>
    ///
    /// [distributor]: <https://developer.arm.com/documentation/ihi0048/a/GIC-Partitioning/The-Distributor?lang=en>
    pub const DISTRIBUTOR_BASE_ADDRESS: usize = 0xF8F0_1000;

    /// The address of the Interrupt Acknowledge Register (ICCIAR)
    pub const INTERRUPT_ACKNOWLEDGE_REGISTER_ADDRESS: usize = CPU_INTERFACE_BASE_ADDRESS + 0x0C;

    /// The address of the End of Interrupt Register (ICCEOIR)
    pub const END_OF_INTERRUPT_REGISTER_ADDRESS: usize = CPU_INTERFACE_BASE_ADDRESS + 0x10;

    pub const ICCPMR_PRIORITY_MASK_REGISTER: usize = CPU_INTERFACE_BASE_ADDRESS + 0x04;
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
unsafe extern "C" fn irq() {
    naked_asm!(
        // Copyright (C) 2017 Amazon.com, Inc. or its affiliates.  All Rights Reserved.
        // Licensed under MIT License
        // http://www.FreeRTOS.org
        "
        /* Return to the interrupted instruction. */
        SUB		lr, lr, #4

        /* Push the return address and SPSR. */
        PUSH	{{lr}}
        MRS		lr, SPSR
        PUSH	{{lr}}

        /* Change to supervisor mode to allow reentry. */
        CPS		#{svc_mode}

        /* Push used registers. */
        PUSH	{{r0-r4, r12}}

        /* Read value from the interrupt acknowledge register, which is stored in r0
        for future parameter and interrupt clearing use. */
        LDR 	r2, ={icciar}
        LDR		r0, [r2]

        /* Ensure bit 2 of the stack pointer is clear.  r2 holds the bit 2 value for
        future use.  _RB_ Does this ever actually need to be done provided the start
        of the stack is 8-byte aligned? */
        MOV		r2, sp
        AND		r2, r2, #4
        SUB		sp, sp, r2

        /* Call the interrupt handler.  r4 pushed to maintain alignment. */
        PUSH	{{r0-r4, lr}}
        BLX		{irq_handler}
        POP		{{r0-r4, lr}}
        ADD		sp, sp, r2

        CPSID	i
        DSB
        ISB

        /* Write the value read from ICCIAR to ICCEOIR. */
        LDR 	r4, ={icceoir}
        STR		r0, [r4]

        /* No context switch.  Restore used registers, LR_irq and SPSR before
        returning. */
        POP		{{r0-r4, r12}}
        CPS		#{irq_mode}
        POP		{{LR}}
        MSR		SPSR_cxsf, LR
        POP		{{LR}}
        MOVS	PC, LR
        ",
        icciar = const interrupt_controller::INTERRUPT_ACKNOWLEDGE_REGISTER_ADDRESS,
        icceoir = const interrupt_controller::END_OF_INTERRUPT_REGISTER_ADDRESS,
        irq_mode = const 0x12,
        svc_mode = const 0x13,
        irq_handler = sym irq_handler,
    );
}

#[unsafe(naked)]
#[instruction_set(arm::a32)]
unsafe extern "C" fn irq_handler(icciar: u32) {
    naked_asm!(
        // Copyright (C) 2017 Amazon.com, Inc. or its affiliates.  All Rights Reserved.
        // Licensed under MIT License
        "
        PUSH	{{LR}}
        FMRX	R1,  FPSCR
        VPUSH	{{D0-D15}}
        VPUSH	{{D16-D31}}
        PUSH	{{R1}}

        BLX		{fpu_safe_irq_handler}

        POP		{{R0}}
        VPOP	{{D16-D31}}
        VPOP	{{D0-D15}}
        VMSR	FPSCR, R0

        POP {{PC}}
        ",
        fpu_safe_irq_handler = sym fpu_safe_irq_handler,
    )
}

/// Process an Interrupt Request (IRQ) with the given ID.
///
/// The ID should be obtained from one of the Interrupt Acknowledge Registers (IARs).
unsafe extern "C" fn fpu_safe_irq_handler(interrupt_id: u32) {
    unsafe {
        vex_sdk::vexSystemApplicationIRQHandler(interrupt_id);
    }
}

unsafe extern "C" fn exception_handler(cause: FaultType /* fault: *const Fault */) -> ! {
    unsafe {
        // let Fault {
        //     cause,
        //     instruction_address: instruction_pointer,
        //     stack_pointer,
        //     registers,
        // } = *fault;

        match cause {
            FaultType::DataAbort => {
                vex_sdk::vexDisplayForegroundColor(0xFF_00_00);
            }
            FaultType::PrefetchAbort => {
                vex_sdk::vexDisplayForegroundColor(0x00_FF_00);
            }
            FaultType::UndefinedInstruction => {
                vex_sdk::vexDisplayForegroundColor(0x00_00_FF);
            }
            FaultType::SupervisorCall => {
                vex_sdk::vexDisplayForegroundColor(0xFF_FF_FF);
            }
        }

        vex_sdk::vexDisplayRectFill(0, 0, 100, 100);

        vex_sdk::vexDisplayString(5, c"ERROR!".as_ptr());

        let msg = "ERROR!\n";
        vex_sdk::vexSerialWriteBuffer(1, msg.as_ptr(), msg.len() as u32);

        loop {
            vex_sdk::vexTasksRun();
        }

        // asm!("dsb", "isb");

        // println!("\n\n*** {cause:?} EXCEPTION ***");
        // println!("  at address 0x{instruction_pointer:x?}");

        // match cause {
        //     FaultType::DataAbort => {
        //         let abort = data_abort_info();
        //         let verb = if abort.is_write() {
        //             "writing to"
        //         } else {
        //             "reading from"
        //         };

        //         println!("  while {verb} address 0x{:x?}", abort.address);
        //     }
        //     FaultType::UndefinedInstruction => todo!(),
        //     FaultType::SupervisorCall => todo!(),
        //     FaultType::PrefetchAbort => todo!(),
        // }

        // println!("\nCPU registers at time of fault (r0-r12, in hexadecimal):\n{registers:x?}\n");
        // println!("help: This indicates the misuse of `unsafe` code.");
        // println!("      Use a symbolizer tool to determine the file and line of the crash.");

        // let profile = if cfg!(debug_assertions) {
        //     "debug"
        // } else {
        //     "release"
        // };
        // println!("      (e.g. llvm-symbolizer -e ./target/armv7a-vex-v5/{profile}/program_name 0x{instruction_pointer:x?})");

        // #[cfg(feature = "backtraces")]
        // {
        //     let gprs = backtrace::GPRS {
        //         r: registers,
        //         lr: instruction_pointer,
        //         pc: instruction_pointer,
        //         sp: stack_pointer,
        //     };
        //     let context = backtrace::make_unwind_context(gprs);
        //     let result = backtrace::print_backtrace(&context);
        //     println!("backtrace result: {result:?}");
        // }

        // match (*fault).cause {
        //     FaultType::DataAbort => vex_sdk::vexSystemDataAbortInterrupt(),
        //     FaultType::PrefetchAbort => vex_sdk::vexSystemPrefetchAbortInterrupt(),
        //     FaultType::SupervisorCall => vex_sdk::vexSystemSWInterrupt(),
        //     FaultType::UndefinedInstruction => vex_sdk::vexSystemUndefinedException(),
        // }

        // loop {
        //     asm!("nop");
        // }
    }
}

/// Process a Fast Interrupt Request (FIQ).
unsafe extern "C" fn fiq_handler() {
    unsafe {
        vex_sdk::vexSystemFIQInterrupt();
    }
}

/// A representation of a data abort exception
#[derive(Debug, Clone, Copy)]
struct DataAbort {
    /// A bitfield with information about the fault.
    status: u32,
    /// The address at which the last fault occurred
    address: u32,
}

impl DataAbort {
    /// Returns whether the abort was caused by a write operation.
    pub const fn is_write(self) -> bool {
        const WRITE_NOT_READ_BIT: u32 = 1 << 11;
        self.status & WRITE_NOT_READ_BIT != 0
    }
}

fn data_abort_info() -> DataAbort {
    let address: u32;
    let status: u32;

    unsafe {
        core::arch::asm!(
            "mrc p15, 0, {fsr}, c5, c0, 0",
            "mrc p15, 0, {far}, c6, c0, 0",
            fsr = out(reg) status,
            far = out(reg) address,
        );
    }

    DataAbort { status, address }
}

#[cfg(feature = "backtraces")]
mod backtrace {
    use core::mem::transmute;

    use vex_libunwind::{registers::UNW_REG_IP, UnwindContext, UnwindCursor, UnwindError};
    use vex_libunwind_sys::{unw_context_t, unw_init_local, CONTEXT_SIZE};

    use crate::println;

    #[repr(C)]
    pub struct GPRS {
        pub r: [u32; 13],
        pub sp: u32,
        pub lr: u32,
        pub pc: u32,
    }

    const REGISTERS_DATA_SIZE: usize = size_of::<unw_context_t>() - size_of::<GPRS>();
    #[repr(C)]
    struct Registers_arm {
        gprs: GPRS,
        data: [u8; REGISTERS_DATA_SIZE],
    }

    /// Create an unwind context using custom registers instead of ones captured
    /// from the current processor state.
    ///
    /// This is based on the ARM implementation of __unw_getcontext:
    /// <https://github.com/llvm/llvm-project/blob/6fc3b40b2cfc33550dd489072c01ffab16535840/libunwind/src/UnwindRegistersSave.S#L834>
    pub fn make_unwind_context(gprs: GPRS) -> UnwindContext {
        let context = Registers_arm {
            gprs,
            // This matches the behavior of __unw_getcontext, which leaves
            // this data uninitialized.
            data: [0; REGISTERS_DATA_SIZE],
        };

        // SAFETY: `context` is a valid `unw_context_t` because it has its
        // general-purpose registers field set.
        UnwindContext::from(unsafe { transmute::<Registers_arm, unw_context_t>(context) })
    }

    pub fn print_backtrace(context: &UnwindContext) -> Result<(), UnwindError> {
        let mut cursor = UnwindCursor::new(context)?;

        println!("\nstack backtrace:");
        loop {
            println!("0x{:x?}", cursor.register(UNW_REG_IP)?);

            if !cursor.step()? {
                break;
            }
        }
        println!();

        Ok(())
    }
}
