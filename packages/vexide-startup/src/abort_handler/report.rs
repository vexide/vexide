use std::{
    ffi::CStr,
    fmt::{self, Write},
};

use vexide_devices::{
    display::{Display, Font, FontFamily, FontSize, Rect, RenderMode, TouchState},
    peripherals::Peripherals,
};

use super::fault::Fault;
use crate::{
    abort_handler::fault::{FaultException, FaultStatus},
    colors::{BLACK, RED, WHITE},
};

pub struct AbortWriter(());

impl AbortWriter {
    pub const BUFFER_SIZE: usize = 2048;
    pub const fn new() -> Self {
        Self(())
    }

    #[expect(clippy::unused_self, reason = "only used for semantics")]
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
        // For some reason IRQ interrupts get disabled on abort. It's unclear why,
        // there's not a ton of info online about this.
        // These are required for serial flushing to work - turn them back on.
        core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
    }

    let fault = unsafe { *fault };
    let fault_status = fault.load_status();

    report_serial(&fault, &fault_status);

    // The previous instances of Peripherals are out-of-scope now.
    let mut peripherals = unsafe { Peripherals::steal() };
    report_display(&mut peripherals.display, &fault, &fault_status);
}

/// Prints the fault to the serial console.
fn report_serial(fault: &Fault, status: &FaultStatus) {
    let source = status.source_description();
    let (short_action, ext_action) = status.action_description();

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
}

/// Draws the fault to the display. Needs a working allocator.
fn report_display(display: &mut Display, fault: &Fault, status: &FaultStatus) -> ! {
    display.set_render_mode(RenderMode::Immediate);

    let source = status.source_description();
    let (action, _) = status.action_description();

    let msg_width = 342;
    let background = Rect::from_dimensions(
        [10, 10],
        msg_width,
        Display::VERTICAL_RESOLUTION as u16 - 10 - 35,
    );

    display.fill(&background, RED);
    display.stroke(&background, WHITE);

    let qr_zone = draw_docs_qr_code(&mut *display, msg_width as i16 + 10 + 10, 10);

    let mut state = DrawState::new(display, 20, 20);

    if fault.exception == FaultException::UndefinedInstruction {
        // This has a shorter message to prevent horizontal overflow.
        state.title(format_args!("{}!", fault.exception));
    } else {
        state.title(format_args!("{} Error!", fault.exception));
    }

    state.details(format_args!(" at address {:#x}", fault.program_counter));
    state.details(format_args!(
        " while {action} address {:#x}",
        fault.address(),
    ));

    state.y_pos += 5;
    state.help(format_args!("Source: {source}"));
    state.y_pos += 10;

    state.help(format_args!("{HELP_MESSAGE}"));
    state.y_pos += 5;
    state.help(format_args!(
        "Additional debugging information has been logged"
    ));
    state.help(format_args!("to the serial console."));
    state.help(format_args!("(Tap QR Code to log the debug info again.)"));

    state.y_pos += 3;
    state.details(format_args!("vexide.dev/docs/aborts"));

    let mut prev_touch_state = display.touch_status().state;

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }

        let touch = display.touch_status();

        if prev_touch_state != TouchState::Released
            && touch.state == TouchState::Released
            && (qr_zone.start.x..=qr_zone.end.x).contains(&touch.x)
            && (qr_zone.start.y..=qr_zone.end.y).contains(&touch.y)
        {
            report_serial(fault, status);
        }

        prev_touch_state = touch.state;
    }
}

/// Draws a QR code pointing to the vexide docs to the screen.
///
/// Returns the region used by the QR code.
fn draw_docs_qr_code(display: &mut Display, base_x: i16, base_y: i16) -> Rect {
    static DOCS_QR_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/abort_qrcode.bin"));
    let mut qr_data = DOCS_QR_CODE;

    let resolution = 4u16;

    let qr_width = u16::from_be_bytes([qr_data[0], qr_data[1]]) as usize;

    qr_data = &qr_data[2..];

    let quiet_zone = 1;
    let real_width_with_quiet_zone = (qr_width + quiet_zone * 2) * resolution as usize;
    let padded_base_x = base_x + quiet_zone as i16 * resolution as i16;
    let padded_base_y = base_y + quiet_zone as i16 * resolution as i16;

    let canvas = Rect::from_dimensions(
        [base_x, base_y],
        real_width_with_quiet_zone as u16,
        real_width_with_quiet_zone as u16,
    );

    display.fill(&canvas, WHITE);

    for y in 0..qr_width {
        for x in 0..qr_width {
            let idx = y * qr_width + x;
            let byte = qr_data[idx / 8];

            let is_dark = (byte >> (idx % 8)) & 1 != 0;

            if is_dark {
                let display_coords = [
                    padded_base_x + x as i16 * resolution as i16,
                    padded_base_y + y as i16 * resolution as i16,
                ];
                display.fill(
                    &Rect::from_dimensions(display_coords, resolution, resolution),
                    BLACK,
                );
            }
        }
    }

    canvas
}

/// Line-based text writer.
struct DrawState<'a> {
    _display: &'a mut Display,
    text: heapless::String<1024>,
    y_pos: i16,
    x_pos: i16,
}

impl<'a> DrawState<'a> {
    pub const fn new(display: &'a mut Display, y_pos: i16, x_pos: i16) -> Self {
        Self {
            _display: display,
            text: heapless::String::new(),
            y_pos,
            x_pos,
        }
    }

    /// Large, title-sized text
    pub fn title(&mut self, args: fmt::Arguments<'_>) {
        self.draw(args, Font::new(FontSize::LARGE, FontFamily::Proportional));
        self.y_pos += 30;
    }

    /// Medium-sized text
    pub fn details(&mut self, args: fmt::Arguments<'_>) {
        self.draw(args, Font::new(FontSize::MEDIUM, FontFamily::Proportional));
        self.y_pos += 20;
    }

    /// Compact text for longer messages
    pub fn help(&mut self, args: fmt::Arguments<'_>) {
        self.draw(args, Font::new(FontSize::SMALL, FontFamily::Proportional));
        self.y_pos += 15;
    }

    /// Common functionality for drawing
    fn draw(&mut self, args: fmt::Arguments<'_>, font: Font) {
        _ = write!(self.text, "{args}\0");

        // This isn't done through the Display API to avoid allocations.
        if let Ok(c_str) = CStr::from_bytes_with_nul(self.text.as_bytes()) {
            font.apply_to_sdk();
            unsafe {
                vex_sdk::vexDisplayForegroundColor(0xFF_FF_FF);
                vex_sdk::vexDisplayPrintf(
                    self.x_pos.into(),
                    (self.y_pos + Display::HEADER_HEIGHT).into(),
                    false.into(),
                    c"%s".as_ptr(),
                    c_str.as_ptr(),
                );
            }
        }
        self.text.clear();
    }
}
