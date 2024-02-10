//! ADI Addressable LEDs
//!
//! This module contains abstractions for interacting with WS2812B addressable smart LED
//! strips over ADI ports.

use alloc::vec::Vec;

use pros_sys::{ext_adi_led_t, PROS_ERR};
use snafu::Snafu;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::{
    color::IntoRgb,
    error::{bail_on, map_errno},
};

/// WS2812B Addressable LED Strip
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAddrLed {
    raw: ext_adi_led_t,
    buffer: Vec<u32>,
    port: AdiPort,
}

/// The max number of LED diodes on one strip that a single ADI port can control.
pub const MAX_LED_LENGTH: usize = 64;

impl AdiAddrLed {
    /// Initialize an LED strip on an ADI port from a buffer of light colors.
    pub fn new<T, I>(port: AdiPort, buf: T) -> Result<Self, AddrLedError>
    where
        T: IntoIterator<Item = I>,
        I: Into<u32>,
    {
        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_init(port.internal_expander_index(), port.index())
        });

        let mut device = Self {
            port,
            raw,
            buffer: buf.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
        };

        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set(
                device.raw,
                device.buffer.as_mut_ptr(),
                device.buffer.len() as u32,
            )
        });

        Ok(device)
    }

    /// Clear the entire led strip of color.
    pub fn clear_all(&mut self) -> Result<(), AddrLedError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_clear_all(
                self.raw,
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
            )
        });

        Ok(())
    }

    /// Set the entire led strip to one color
    pub fn set_all(&mut self, color: impl IntoRgb) -> Result<(), AddrLedError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set_all(
                self.raw,
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
                color.into_rgb().into(),
            )
        });

        Ok(())
    }

    /// Set the entire led strip using the colors contained in a new buffer.
    pub fn set_buffer<T, I>(&mut self, buf: T) -> Result<(), AddrLedError>
    where
        T: IntoIterator<Item = I>,
        I: IntoRgb,
    {
        self.buffer = buf
            .into_iter()
            .map(|i| i.into_rgb().into())
            .collect::<Vec<_>>();

        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set(self.raw, self.buffer.as_mut_ptr(), self.buffer.len() as u32)
        });

        Ok(())
    }

    /// Set the color of a single LED on the strip.
    pub fn set_pixel(&mut self, index: usize, color: impl IntoRgb) -> Result<(), AddrLedError> {
        if self.buffer.get(index).is_some() {
            self.buffer[index] = color.into_rgb().into();

            bail_on!(PROS_ERR, unsafe {
                pros_sys::ext_adi_led_set(
                    self.raw,
                    self.buffer.as_mut_ptr(),
                    self.buffer.len() as u32,
                )
            });

            Ok(())
        } else {
            Err(AddrLedError::InvalidBufferAccess)
        }
    }

    /// Clear one LED on the strip.
    pub fn clear_pixel(&mut self, index: usize) -> Result<(), AddrLedError> {
        self.set_pixel(index, 0u32)?;

        Ok(())
    }
}

impl AdiDevice for AdiAddrLed {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalOut
    }
}

#[cfg(feature = "smart-leds-trait")]
impl smart_leds_trait::SmartLedsWrite for AdiAddrLed {
    type Error = AddrLedError;
    type Color = Rgb;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.buffer = iterator
            .into_iter()
            .map(|i| i.into().into())
            .collect::<Vec<_>>();

        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set(self.raw, self.buffer.as_mut_ptr(), self.buffer.len() as u32)
        });

        Ok(())
    }
}

/// Errors that can occur when interacting with an Addrled strip.
#[derive(Debug, Snafu)]
pub enum AddrLedError {
    /// Failed to access LED buffer. A given value is not correct, or the buffer is null.
    #[snafu(display(
        "Failed to access LED buffer. A given value is not correct, or the buffer is null."
    ))]
    InvalidBufferAccess,

    #[snafu(display("{source}"), context(false))]
    /// Generic ADI related error.
    Adi {
        /// The source of the error
        source: AdiError,
    },
}

map_errno! {
    AddrLedError {
        EINVAL => Self::InvalidBufferAccess,
    }
    inherit AdiError;
}
