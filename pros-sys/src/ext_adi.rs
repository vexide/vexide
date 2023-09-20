//! Contains prototypes for interfacing with the 3-Wire Expander.

use crate::adi::*;
use core::ffi::*;

/**
Reference type for an initialized encoder.

This merely contains the port number for the encoder, unlike its use as an
object to store encoder data in PROS 2.
*/
pub type ext_adi_encoder_t = i32;

/**
Reference type for an initialized ultrasonic.

This merely contains the port number for the ultrasonic, unlike its use as an
object to store encoder data in PROS 2.
*/
pub type ext_adi_ultrasonic_t = i32;

/**
Reference type for an initialized gyroscope.

This merely contains the port number for the gyroscope, unlike its use as an
object to store gyro data in PROS 2.

(Might Be useless with the wire expander.)
*/
pub type ext_adi_gyro_t = i32;

/**
Reference type for an initialized potentiometer.

This merely contains the port number for the potentiometer, unlike its use as an
object to store gyro data in PROS 2.
*/
pub type ext_adi_potentiometer_t = i32;

/**
Reference type for an initialized addressable led, which stores its smart and adiport.
*/
pub type ext_adi_led_t = i32;

extern "C" {
    pub fn ext_adi_port_get_config(smart_port: u8, adi_port: u8) -> adi_port_config_e_t;
    pub fn ext_adi_port_get_value(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_port_set_config(smart_port: u8, adi_port: u8, port_type: adi_port_config_e_t) -> i32;
    pub fn ext_adi_port_set_value(smart_port: u8, adi_port: u8, value: i32) -> i32;

    pub fn ext_adi_analog_calibrate(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_analog_read(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_analog_read_calibrated(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_analog_read_calibrated_HR(smart_port: u8, adi_port: u8) -> i32;

    pub fn ext_adi_digital_read(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_digital_get_new_press(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_digital_write(smart_port: u8, adi_port: u8, value: bool) -> i32;
    pub fn ext_adi_pin_mode(smart_port: u8, adi_port: u8, mode: u8) -> i32;

    pub fn ext_adi_motor_set(smart_port: u8, adi_port: u8, speed: i8) -> i32;
    pub fn ext_adi_motor_get(smart_port: u8, adi_port: u8) -> i32;
    pub fn ext_adi_motor_stop(smart_port: u8, adi_port: u8) -> i32;

    pub fn ext_adi_encoder_get(enc: ext_adi_encoder_t) -> i32;
    pub fn ext_adi_encoder_init(smart_port: u8, adi_port_top: u8, adi_port_bottom: u8, reverse: bool) -> ext_adi_encoder_t;
    pub fn ext_adi_encoder_reset(enc: ext_adi_encoder_t) -> i32;
    pub fn ext_adi_encoder_shutdown(enc: ext_adi_encoder_t) -> i32;

    pub fn ext_adi_ultrasonic_get(ult: ext_adi_ultrasonic_t) -> i32;
    pub fn ext_adi_ultrasonic_init(smart_port: u8, adi_port_ping: u8, adi_port_echo: u8) -> ext_adi_ultrasonic_t;
    pub fn ext_adi_ultrasonic_shutdown(ult: ext_adi_ultrasonic_t) -> i32;

    pub fn ext_adi_gyro_get(gyro: ext_adi_gyro_t) -> c_double;
    pub fn ext_adi_gyro_init(smart_port: u8, adi_port: u8, multiplier: c_double) -> ext_adi_gyro_t;
    pub fn ext_adi_gyro_reset(gyro: ext_adi_gyro_t) -> i32;
    pub fn ext_adi_gyro_shutdown(gyro: ext_adi_gyro_t) -> i32;

    pub fn ext_adi_potentiometer_init(smart_port: u8, adi_port: u8, potentiometer_type: adi_potentiometer_type_e_t) -> ext_adi_potentiometer_t;
    pub fn ext_adi_potentiometer_get_angle(pot: ext_adi_potentiometer_t) -> c_double;

    pub fn ext_adi_led_init(smart_port: u8, adi_port: u8) -> ext_adi_led_t;
    // note: Why does this take in colors if its going to remove them all??
    pub fn ext_adi_led_clear_all(led: ext_adi_led_t, buffer: *mut u32, buffer_length: u32) -> i32;
    pub fn ext_adi_led_set(led: ext_adi_led_t, buffer: *mut u32, buffer_length: u32) -> i32;

}