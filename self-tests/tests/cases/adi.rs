//! ADI (three-wire) port and device tests.
//!
//! These tests are designed to be run on a Brain with nothing plugged into the ADI ports.

use libtest_mimic::Failed;
use vexide::{
    adi::{
        ADI_UPDATE_INTERVAL, AdiDeviceType, AdiPort, analog::ADC_MAX_VALUE, digital::LogicLevel,
    },
    color::Color,
    math::Angle,
    prelude::*,
    smart::{PortError, SmartDeviceType},
};

macro_rules! all_adis {
    ($p:ident) => {
        [
            $p.adi_a, $p.adi_b, $p.adi_c, $p.adi_d, $p.adi_e, $p.adi_f, $p.adi_g, $p.adi_h,
        ]
    };
}

macro_rules! all_ports {
    ($p:ident) => {
        [
            $p.port_1, $p.port_2, $p.port_3, $p.port_4, $p.port_5, $p.port_6, $p.port_7, $p.port_8,
            $p.port_9, $p.port_10, $p.port_11, $p.port_12, $p.port_13, $p.port_14, $p.port_15,
            $p.port_16, $p.port_17, $p.port_18, $p.port_19, $p.port_20, $p.port_21,
        ]
    };
}

fn configure_as_ana_in(port: AdiPort) -> AdiPort {
    AdiAnalogIn::new(port).into()
}

/// Simple accesses to the internal ADI ports should report the correct metadata and must never fail
/// regardless of what's plugged in.
pub async fn can_access_internal_adi(peripherals: Peripherals) -> Result<(), Failed> {
    for (idx, adi) in all_adis!(peripherals).into_iter().enumerate() {
        let port_num = (idx + 1) as u8;
        assert_eq!(adi.number(), port_num);
        assert_eq!(adi.expander_number(), None);

        let ana_in = AdiAnalogIn::new(adi);
        assert_eq!(ana_in.port_numbers(), [port_num]);
        assert_eq!(ana_in.expander_port_number(), None);
        assert_eq!(ana_in.device_type(), AdiDeviceType::AnalogIn);
        ana_in.value()?;
        ana_in.voltage()?;

        let digi_in = AdiDigitalIn::new(ana_in.into());
        assert_eq!(digi_in.port_numbers(), [port_num]);
        assert_eq!(digi_in.expander_port_number(), None);
        assert_eq!(digi_in.device_type(), AdiDeviceType::DigitalIn);
        digi_in.level()?;

        let mut digi_out = AdiDigitalOut::new(digi_in.into());
        assert_eq!(digi_out.port_numbers(), [port_num]);
        assert_eq!(digi_out.expander_port_number(), None);
        assert_eq!(digi_out.device_type(), AdiDeviceType::DigitalOut);

        digi_out.set_high()?;
        digi_out.set_low()?;
        digi_out.set_level(LogicLevel::High)?;
        digi_out.set_level(LogicLevel::Low)?;
        digi_out.toggle()?;
        digi_out.level()?;
        digi_out.is_high()?;
        digi_out.is_low()?;
    }

    Ok(())
}

/// ADI ports should report their current level as whatever was last commanded.
pub async fn outputs_report_requested_level(peripherals: Peripherals) -> Result<(), Failed> {
    let port = AdiDigitalOut::with_initial_level(peripherals.adi_a, LogicLevel::Low);
    assert_eq!(port.level()?, LogicLevel::Low);

    let port = AdiPort::from(port);
    let mut port = AdiDigitalOut::with_initial_level(port, LogicLevel::High);
    assert_eq!(port.level()?, LogicLevel::High);

    port.set_high()?;
    assert_eq!(port.level()?, LogicLevel::High);

    port.set_low()?;
    assert_eq!(port.level()?, LogicLevel::Low);

    port.toggle()?;
    assert_eq!(port.level()?, LogicLevel::High);

    port.toggle()?;
    assert_eq!(port.level()?, LogicLevel::Low);

    for level in [LogicLevel::High, LogicLevel::Low] {
        port.set_level(level)?;
        assert_eq!(port.level()?, level);
    }

    Ok(())
}

/// ADI device constructors should configure their underlying ports as the expected device type.
pub async fn port_configuration_round_trips(peripherals: Peripherals) -> Result<(), Failed> {
    /// Convert the given device back into a AdiPort and verify that it is of the expected type.
    fn assert_configured<D: AdiDevice<1>>(device: D) -> Result<AdiPort, Failed> {
        let expected = device.device_type();
        let port = AdiPort::from(device);

        let configured = port.configured_type()?;
        assert_eq!(
            configured,
            expected,
            "ADI port {} was configured as {configured:?}, expected {expected:?}",
            port.number(),
        );

        Ok(port)
    }

    let mut port = peripherals.adi_a;
    // This loop ensures that re-configuration works for everything as well.
    for _ in 0..2 {
        port = assert_configured(AdiAnalogIn::new(port))?;
        port = assert_configured(AdiDigitalIn::new(port))?;
        port = assert_configured(AdiDigitalOut::new(port))?;
        port = assert_configured(AdiAddrLed::<8>::new(port))?;
        port = assert_configured(AdiPwmOut::new(port))?;
        port = assert_configured(AdiServo::new(port))?;
        port = assert_configured(AdiMotor::new(port, false))?;
        port = assert_configured(AdiMotor::new(port, true))?;
        port = assert_configured(AdiLineTracker::new(port))?;
        port = assert_configured(AdiLightSensor::new(port))?;
        port = assert_configured(AdiAccelerometer::new(port, Sensitivity::High))?;
        port = assert_configured(AdiPotentiometer::new(port, PotentiometerType::Legacy))?;
        port = assert_configured(AdiPotentiometer::new(port, PotentiometerType::V2))?;
        port = assert_configured(AdiGyroscope::new(port))?;
    }

    Ok(())
}

/// Analog sensor readings must be within the range of the Brain's 12-bit ADC.
pub async fn analog_readings_are_in_range(peripherals: Peripherals) -> Result<(), Failed> {
    let analog = AdiAnalogIn::new(peripherals.adi_a);
    let light = AdiLightSensor::new(peripherals.adi_b);
    let line = AdiLineTracker::new(peripherals.adi_c);
    let accel = AdiAccelerometer::new(peripherals.adi_d, Sensitivity::High);
    assert_eq!(accel.sensitivity(), Sensitivity::High);
    let pot = AdiPotentiometer::new(peripherals.adi_e, PotentiometerType::V2);
    assert_eq!(pot.potentiometer_type(), PotentiometerType::V2);

    // Allow time for new samples to be received.
    sleep(ADI_UPDATE_INTERVAL * 2).await;

    let value = analog.value()?;
    let voltage = analog.voltage()?;
    assert!(value <= ADC_MAX_VALUE);
    assert!((0.0..=5.0).contains(&voltage));

    let raw_brightness = light.raw_brightness()?;
    let brightness = light.brightness()?;
    assert!(raw_brightness <= ADC_MAX_VALUE);
    assert!((0.0..=1.0).contains(&brightness));

    let raw_reflectivity = line.raw_reflectivity()?;
    let reflectivity = line.reflectivity()?;
    assert!(raw_reflectivity <= ADC_MAX_VALUE);
    assert!((0.0..=1.0).contains(&reflectivity));

    let raw_acceleration = accel.raw_acceleration()?;
    let acceleration = accel.acceleration()?;
    assert!(raw_acceleration <= ADC_MAX_VALUE);
    assert!((0.0..=Sensitivity::HIGH_MAX_ACCELERATION).contains(&acceleration));

    let angle = pot.angle()?;
    assert!((Angle::ZERO..=pot.max_angle()).contains(&angle));

    Ok(())
}

/// Two-wire devices accept both of their ports in either order and configure them properly.
pub async fn two_wire_devices_configure_ports(peripherals: Peripherals) -> Result<(), Failed> {
    // Switching ports to be analog inputs ensures configuration is deterministic regardless of how
    // previous tests configured the ports.
    let top = configure_as_ana_in(peripherals.adi_a);
    let bottom = configure_as_ana_in(peripherals.adi_b);

    let mut encoder = AdiOpticalEncoder::new(top, bottom);
    assert_eq!(encoder.port_numbers(), [1, 2]);
    assert_eq!(encoder.expander_port_number(), None);
    assert_eq!(encoder.device_type(), AdiDeviceType::Encoder);

    // Even if the encoder is unplugged we should be able to access its API.
    encoder.position()?;
    encoder.set_position(Angle::from_degrees(180.0))?;
    assert!((encoder.position()? - Angle::from_degrees(180.0)).abs() < Angle::from_radians(0.1));
    encoder.reset_position()?;
    assert_eq!(encoder.position()?, Angle::ZERO);

    let (top, bottom): (AdiPort, AdiPort) = encoder.into();
    assert_eq!((top.number(), bottom.number()), (1, 2));

    // Both ports should report being an encoder (even though internally only one is configured).
    assert_eq!(top.configured_type()?, AdiDeviceType::Encoder);
    assert_eq!(bottom.configured_type()?, AdiDeviceType::Encoder);

    let first = configure_as_ana_in(peripherals.adi_d);
    let second = configure_as_ana_in(peripherals.adi_c);

    // Both (Low, High) and (High, Low) port orders should be accepted and their order should be
    // preserved.
    let reversed = AdiOpticalEncoder::new(first, second);
    assert_eq!(reversed.port_numbers(), [4, 3]);
    reversed.position()?;

    let (top, bottom): (AdiPort, AdiPort) = reversed.into();
    assert_eq!((top.number(), bottom.number()), (4, 3));
    assert_eq!(top.configured_type()?, AdiDeviceType::Encoder);
    assert_eq!(bottom.configured_type()?, AdiDeviceType::Encoder);

    let output = configure_as_ana_in(peripherals.adi_e);
    let input = configure_as_ana_in(peripherals.adi_f);

    // Range finders do not support reversing the order of their ports.
    let range_finder = AdiRangeFinder::new(output, input);
    assert_eq!(range_finder.port_numbers(), [5, 6]);
    assert_eq!(range_finder.expander_port_number(), None);
    assert_eq!(range_finder.device_type(), AdiDeviceType::RangeFinder);

    // Reading will give bogus results if we don't have hardware connected but it shouldn't error.
    range_finder.distance()?;

    let (output, input): (AdiPort, AdiPort) = range_finder.into();
    assert_eq!((output.number(), input.number()), (5, 6));
    assert_eq!(output.configured_type()?, AdiDeviceType::RangeFinder);
    assert_eq!(input.configured_type()?, AdiDeviceType::RangeFinder);

    Ok(())
}

/// Output devices should accept all supported values and saturate on out-of-range.
pub async fn output_devices_accept_writes(peripherals: Peripherals) -> Result<(), Failed> {
    let mut pwm = AdiPwmOut::new(peripherals.adi_a);
    for output in [0, 128, u8::MAX] {
        pwm.set_output(output)?;
    }

    let mut servo = AdiServo::new(peripherals.adi_b);
    for target in [
        AdiServo::MIN_POSITION,
        Angle::ZERO,
        AdiServo::MAX_POSITION,
        // Out-of-range targets saturate instead of panicking/erroring.
        Angle::from_degrees(720.0),
        Angle::from_degrees(-720.0),
    ] {
        servo.set_target(target)?;
    }
    for target in [i8::MIN, 0, i8::MAX] {
        servo.set_raw_target(target)?;
    }

    let mut leds = AdiAddrLed::<8>::new(peripherals.adi_c);
    leds.set_all(Color::RED)?;
    leds.set_pixel(0, Color::GREEN)?;
    leds.set_pixel(7, Color::BLUE)?;
    assert_eq!(leds.set_buffer(&[Color::WHITE; 8])?, 8);
    // A short buffer only updates the pixels it covers.
    assert_eq!(leds.set_buffer(&[Color::BLACK; 4])?, 4);

    Ok(())
}

/// ADI motors should saturate inputs when they are out-of-range and report back their output
/// values.
pub async fn motor_set_output_reads_back(peripherals: Peripherals) -> Result<(), Failed> {
    let mut motor = AdiMotor::new(peripherals.adi_a, false);
    for output in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        motor.set_output(output)?;

        let expected_output = output.clamp(-1.0, 1.0);
        assert!((motor.output()? - expected_output).abs() < 0.1);
    }
    for output in [i8::MIN, 0, i8::MAX] {
        motor.set_raw_output(output)?;

        let expected_output = output.clamp(-127, 127);
        assert_eq!(motor.raw_output()?, expected_output);
    }

    motor.stop()?;
    assert_eq!(motor.raw_output()?, 0);

    Ok(())
}

/// A gyroscope on an unpopulated port is readable and reports that it isn't calibrating.
pub async fn gyroscope_reports_calibration_state(peripherals: Peripherals) -> Result<(), Failed> {
    let gyro = AdiGyroscope::new(peripherals.adi_a);
    assert_eq!(gyro.port_numbers(), [1]);
    assert_eq!(gyro.expander_port_number(), None);
    assert_eq!(gyro.device_type(), AdiDeviceType::Gyro);

    sleep(ADI_UPDATE_INTERVAL * 2).await;

    // Simply creating the AdiGyroscope shouldn't start calibration, so yaw should be readable
    // immediately.
    assert!(!gyro.is_calibrating()?);
    gyro.yaw()?;

    Ok(())
}

/// Devices created from an [`AdiExpander`] correctly report the expander's port info and fail
/// to read only if the expander is missing.
pub async fn expander_ports_report_expander_state(peripherals: Peripherals) -> Result<(), Failed> {
    for smart_port in all_ports!(peripherals) {
        let smart_port_number = smart_port.number();
        // Note: possible time-of-check to time-of-use race condition here, but hopefully people
        // won't be unplugging stuff in the middle of tests so this shouldn't be an issue.
        let connected = smart_port.device_type();

        let expander = AdiExpander::new(smart_port);
        assert_eq!(expander.port_number(), smart_port_number);
        assert_eq!(expander.device_type(), SmartDeviceType::Adi);

        for (idx, adi) in all_adis!(expander).into_iter().enumerate() {
            let port_num = (idx + 1) as u8;
            assert_eq!(adi.number(), port_num);
            assert_eq!(adi.expander_number(), Some(smart_port_number));

            let analog = AdiAnalogIn::new(adi);
            assert_eq!(analog.port_numbers(), [port_num]);
            assert_eq!(analog.expander_port_number(), Some(smart_port_number));

            // Unlike the builtin ports, expander devices may report an error if the expander
            // itself is missing or is the wrong kind of device.
            match connected {
                None => assert_eq!(
                    analog.value(),
                    Err(PortError::Disconnected {
                        port: smart_port_number
                    }),
                ),
                Some(SmartDeviceType::Adi) => {
                    analog.value()?;
                }
                Some(actual) => assert_eq!(
                    analog.value(),
                    Err(PortError::IncorrectDevice {
                        expected: SmartDeviceType::Adi,
                        actual,
                        port: smart_port_number,
                    }),
                ),
            }
        }
    }

    Ok(())
}
