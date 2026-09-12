use std::time::Duration;

use libtest_mimic::{Arguments, ColorSetting, Conclusion, Trial};
use vexide::{
    color::Color,
    display::{Alignment, Font, FontFamily, FontSize, Text},
    prelude::*,
};

mod cases;

macro_rules! test {
    ($module:ident :: $name:ident) => {
        Trial::test(stringify!($name), move || {
            block_on(cases::$module::$name(unsafe { Peripherals::steal() }))
        })
        .with_kind(stringify!($module))
    };
}

#[vexide::main(banner(enabled = false))]
async fn main(peripherals: Peripherals) {
    // VEXos programs do not have CLI args, so there's nothing to parse here. In the future we may
    // want to pull args off stdin or something else to allow more configurability.
    let args = Arguments {
        color: Some(ColorSetting::Always),
        test_threads: Some(1),
        ..Default::default()
    };

    let tests = vec![
        test!(adi::can_access_internal_adi),
        // FIXME(#464): ADI outputs report the wrong level.
        test!(adi::outputs_report_requested_level).with_ignored_flag(true),
        test!(adi::port_configuration_round_trips),
        test!(adi::analog_readings_are_in_range),
        test!(adi::two_wire_devices_configure_ports),
        test!(adi::output_devices_accept_writes),
        // FIXME(#464): motor.raw_output() seems to always return -127.
        test!(adi::motor_set_output_reads_back).with_ignored_flag(true),
        test!(adi::gyroscope_reports_calibration_state),
        test!(adi::expander_ports_report_expander_state),
    ];
    let num_tests = tests.len();
    let result = libtest_mimic::run(&args, tests);

    // This message is for the sake of developers using cargo-v5 which doesn't exit on its own
    // when a VEX program finishes.
    println!("All tests have finished, press Ctrl-C to exit.");

    display_results(peripherals.display, &result, num_tests as u64);
    sleep(Duration::from_secs(2)).await;
    result.exit();
}

fn display_results(mut display: Display, result: &Conclusion, num_tests: u64) {
    let results_text = format!(
        "{} {}/{}",
        if result.has_failed() { "FAIL" } else { "OK" },
        result.num_passed,
        num_tests - result.num_ignored - result.num_filtered_out,
    );

    let font = Font::new(FontSize::EXTRA_LARGE, FontFamily::Monospace);
    let pos = [
        Display::HORIZONTAL_RESOLUTION / 2,
        Display::VERTICAL_RESOLUTION / 2,
    ];

    let text = Text::from_string_aligned(
        results_text,
        font,
        pos,
        Alignment::Center,
        Alignment::Center,
    );
    display.erase(if result.has_failed() {
        0xAA2222
    } else {
        0x22AA22
    });
    display.draw_text(&text, Color::WHITE, None);
}
