//! ADI Addressable LEDs
//!
//! This module contains abstractions for interacting with WS2812B addressable smart LED
//! strips over ADI ports.

use alloc::{vec, vec::Vec};

use snafu::Snafu;
use vex_sdk::vexDeviceAdiAddrLedSet;

use super::{AdiDevice, AdiDeviceType, AdiPort};
#[cfg(feature = "smart_leds_trait")]
use crate::color::Rgb;
use crate::{color::IntoRgb, PortError};

/// WS2812B Addressable LED Strip
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAddrLed {
    port: AdiPort,
    buf: Vec<u32>,
}

impl AdiAddrLed {
    /// The max number of LED diodes on one strip that a single ADI port can control.
    pub const MAX_LENGTH: usize = 64;

    /// Initialize an LED strip on an ADI port with a given number of diodes.
    pub fn new<T, I>(port: AdiPort, length: usize) -> Result<Self, AddrLedError> {
        if length > Self::MAX_LENGTH {
            return Err(AddrLedError::BufferTooLarge);
        }

        Ok(Self {
            port,
            buf: vec![0; length],
        })
    }

    fn update(&mut self) {
        unsafe {
            vexDeviceAdiAddrLedSet(
                self.port.device_handle(),
                self.port.index(),
                self.buf.as_mut_ptr(),
                0,
                self.buf.len() as u32,
                0,
            );
        }
    }

    /// Set the entire led strip to one color.
    pub fn set_all(&mut self, color: impl IntoRgb) -> Result<(), AddrLedError> {
        _ = self.set_buffer(vec![u32::from(color.into_rgb()); self.buf.len()])?;
        Ok(())
    }

    /// Sets an individual diode color on the strip. Returns `AddrledError::OutOfRange` if the provided
    /// index is out of range of the current buffer length.
    pub fn set_pixel(&mut self, index: usize, color: impl IntoRgb) -> Result<(), AddrLedError> {
        if let Some(pixel) = self.buf.get_mut(index) {
            *pixel = color.into_rgb().into();
            self.update();
            Ok(())
        } else {
            Err(AddrLedError::OutOfRange)
        }
    }

    /// Attempt to write an iterator of colors to the LED strip. Returns how many colors were
    /// actually written.
    pub fn set_buffer<T, I>(&mut self, iter: T) -> Result<usize, AddrLedError>
    where
        T: IntoIterator<Item = I>,
        I: IntoRgb,
    {
        self.port.validate_expander()?;

        let old_length = self.buf.len();

        self.buf = iter
            .into_iter()
            .map(|i| i.into_rgb().into())
            .collect::<Vec<_>>();

        self.buf.resize(old_length, 0); // Preserve previous strip length.

        self.update();

        Ok(self.buf.len())
    }
}

impl AdiDevice for AdiAddrLed {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalOut
    }
}

#[cfg(feature = "smart_leds_trait")]
impl smart_leds_trait::SmartLedsWrite for AdiAddrLed {
    type Error = AddrLedError;
    type Color = Rgb;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.port.validate_expander()?;

        let buf = iterator
            .into_iter()
            .map(|i| i.into().into())
            .collect::<Vec<_>>();

        if buf.len() > Self::MAX_LENGTH {
            return Err(AddrLedError::BufferTooLarge);
        }

        self.buf = buf;
        self.update();

        Ok(())
    }
}

/// Errors that can occur when interacting with an Addrled strip.
#[derive(Debug, Snafu)]
pub enum AddrLedError {
    /// The provided index was not in range of the current buffer's length.
    OutOfRange,

    /// The length of the provided buffer exceeded the maximum strip length that ADI can control (64).
    BufferTooLarge,

    #[snafu(display("{source}"), context(false))]
    /// Generic ADI related error.
    Adi {
        /// The source of the error
        source: PortError,
    },
}
