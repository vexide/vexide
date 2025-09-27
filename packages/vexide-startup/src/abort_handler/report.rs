use std::{
    fmt::{self, Write},
    ptr,
};

use talc::{ErrOnOom, Span, Talc};
use vexide_devices::{
    display::{Display, Font, FontFamily, FontSize, HAlign, Rect, RenderMode, Text, VAlign},
    peripherals::Peripherals,
    rgb::Rgb,
};

use super::fault::{Fault, FaultException};
use crate::{__heap_end, __heap_start, abort_handler::fault::FaultStatus, allocator::ALLOCATOR};

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

const HELP_MESSAGE: &str = "This CPU fault indicates the misuse of `unsafe` code.";

#[instruction_set(arm::a32)]
pub unsafe extern "aapcs" fn fault_exception_handler(fault: *const Fault) -> ! {
    unsafe {
        // how and why does this work
        core::arch::asm!("cpsie i");
    }

    let fault = unsafe { *fault };

    let fault_status = fault.load_status();

    let source = fault_status.source_description();
    let (short_action, ext_action) = fault_status.action_description();

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
        "    while {short_action} {ext_action} address {:#x}",
        fault.address()
    );

    _ = writeln!(&mut writer, "\nregisters at time of fault:");

    for (i, register) in fault.registers.iter().enumerate() {
        if i > 9 {
            _ = writeln!(writer, "r{i}: {register:#x}");
        } else {
            _ = writeln!(writer, " r{i}: {register:#x}");
        }
    }

    _ = writeln!(writer, " sp: {:#x}", fault.stack_pointer);
    _ = writeln!(writer, " lr: {:#x}", fault.link_register);
    _ = writeln!(writer, " pc: {:#x}", fault.program_counter);

    _ = writeln!(&mut writer, "help: {HELP_MESSAGE}");
    _ = writeln!(
        &mut writer,
        "      Use a symbolizer tool to determine the location of the crash."
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

    // Before this we haven't needed to allocate, but display drawing does need the allocator.

    // Since we could have aborted anywhere - including in the middle of an allocation,
    // we must treat the allocator as uninitialized memory and all existing allocations as invalid.
    // Thus, we use `ptr::write` to install a new allocator without dropping the old one.
    unsafe {
        // lock() is a no-op here since we're using `AssumeUnlockable`.
        let mut alloc = ALLOCATOR.lock();
        ptr::write(&raw mut *alloc, Talc::new(ErrOnOom));

        // I solemnly swear I will not access any old allocations!
        // We will not be returning from this function, so this should be fine as long
        // as no heap-allocated globals are accessed. Those would be invalidated anyways.
        _ = alloc.claim(Span::new(&raw mut __heap_start, &raw mut __heap_end));
    }

    // This is similar to above - the previous instances of Peripherals are out-of-scope now.
    let mut peripherals = unsafe { Peripherals::steal() };

    report_display(&mut peripherals.display, &fault, &fault_status);

    // match fault.exception {
    //     FaultException::DataAbort => unsafe {
    //         vex_sdk::vexSystemDataAbortInterrupt();
    //     },
    //     FaultException::PrefetchAbort => unsafe {
    //         vex_sdk::vexSystemPrefetchAbortInterrupt();
    //     },
    //     FaultException::UndefinedInstruction => unsafe {
    //         vex_sdk::vexSystemUndefinedException();
    //     },
    // }

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}

const RED: Rgb<u8> = Rgb::new(0xFF, 0x20, 0x20);
const WHITE: Rgb<u8> = Rgb::new(0xFF, 0xFF, 0xFF);
const BLACK: Rgb<u8> = Rgb::new(0x00, 0x00, 0x00);

struct DrawState<'a> {
    display: &'a mut Display,
    y_pos: i16,
    x_pos: i16,
}

impl DrawState<'_> {
    fn title(&mut self, text: &str) {
        self.draw(text, Font::new(FontSize::LARGE, FontFamily::Proportional));
        self.y_pos += 30;
    }

    fn details(&mut self, text: &str) {
        self.draw(text, Font::new(FontSize::MEDIUM, FontFamily::Proportional));
        self.y_pos += 20;
    }

    fn help(&mut self, text: &str) {
        self.draw(text, Font::new(FontSize::SMALL, FontFamily::Proportional));
        self.y_pos += 15;
    }

    fn draw(&mut self, text: &str, font: Font) {
        self.display.draw_text(
            &Text::new(text, font, [self.x_pos, self.y_pos]),
            WHITE,
            None,
        );
    }
}

fn report_display(display: &mut Display, fault: &Fault, status: &FaultStatus) {
    display.set_render_mode(RenderMode::Immediate);

    let source = status.source_description();
    let (action, _) = status.action_description();

    let entire_screen = Rect::new(
        [0, 0],
        [Display::HORIZONTAL_RESOLUTION, Display::VERTICAL_RESOLUTION],
    );

    display.fill(&entire_screen, RED);

    let mut state = DrawState {
        display,
        y_pos: 20,
        x_pos: 20,
    };

    state.title(&format!("{}!", fault.exception));
    state.details(&format!(" at address {:#x}", fault.program_counter));
    state.details(&format!(" while {action} address {:#x}", fault.address(),));

    state.y_pos += 5;
    state.title("vexide.dev/docs/aborts");
    state.y_pos += 5;

    state.help(HELP_MESSAGE);
    state.y_pos += 5;
    state.help("Additional debugging information has been logged to the serial console.");
    state.y_pos += 8;

    state.help("*     Visit the link above for help!!");

    state.y_pos += 8;

    state.help(&format!("Source: {source}"));

    state.x_pos = 400;
    state.y_pos -= 38;

    // The face is here for emotional support.
    // Users will be understandably terrified that their program has undefined behavior...
    // (Not to worry: vexide is here to help.)
    state.help(r"\(@w@)/");

    draw_docs_qr_code(&mut *display, 365, 15);
}

fn draw_docs_qr_code(display: &mut Display, base_x: i16, base_y: i16) {
    static DOCS_QR_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/abort_qrcode.bin"));
    let mut qr_data = DOCS_QR_CODE;

    let resolution = 4u16;

    let qr_width = u16::from_be_bytes([qr_data[0], qr_data[1]]) as usize;

    qr_data = &qr_data[2..];

    let quiet_zone = 1;
    let real_width_with_quiet_zone = (qr_width + quiet_zone * 2) * resolution as usize;

    let background = Rect::from_dimensions(
        [base_x - resolution as i16, base_y - resolution as i16],
        real_width_with_quiet_zone as u16,
        real_width_with_quiet_zone as u16,
    );

    display.fill(&background, WHITE);

    for y in 0..qr_width {
        for x in 0..qr_width {
            let idx = y * qr_width + x;
            let byte = qr_data[idx / 8];

            let is_dark = (byte >> (idx % 8)) & 1 != 0;

            if is_dark {
                let display_coords = [
                    base_x + x as i16 * resolution as i16,
                    base_y + y as i16 * resolution as i16,
                ];
                display.fill(
                    &Rect::from_dimensions(display_coords, resolution, resolution),
                    BLACK,
                );
            }
        }
    }
}
