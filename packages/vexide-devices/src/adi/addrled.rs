//! ADI Addressable LEDs
//!
//! This module provides an interface for controlling [WS2812B] addressable LED strips over ADI
//! ports. These are commonly used for decorative lighting. More can be read about using them in
//! [this blog post](https://sylvie.fyi/posts/v5-addrled/) and
//! [this forum post](https://www.vexforum.com/t/v5-addressable-leds/106960).
//!
//! # Hardware Overview
//!
//! ADI ports are capable of controlling a WS2812B LED strip with up to 64 diodes per set of 8 ADI
//! ports. This limitation is due to the 2A current limit on ADI ports — plugging multiple strips
//! into the same set of ADI ports may cause your lights to flicker due to this limit being reached.
//! If you require more than 64 continuously running diodes, then you can run each strip through its
//! own [ADI Expander](crate::smart::expander::AdiExpander).
//!
//! The V5's ADI ports can present some technical challenges when interfacing with LEDs. Some
//! commercially available strips will not work with the V5 out of the box, but mileage may vary.
//! This is mainly caused by two "quirks" of the V5's ADI ports:
//!
//! - ADI ports operate at 3.3V digital logic, but most WS2812B strips expect 5V logic.
//! - The Brain's ADI ports include built-in short protection via a 1kΩ resistor that may impact
//!   signal timing on some strips, slowing down the edges of digital logic pulses sent to strip. In
//!   rare cases, this can cause issues with some strips.
//!
//! Using something like a [74HCT125 buffer] inline with the output to convert the 3.3-5V logic
//! addresses both these problems.
//!
//! # `smart-leds-trait` Integration
//!
//! vexide implements the [`SmartLedsWrite`] trait from the [`smart-leds-rs`](https://github.com/smart-leds-rs)
//! ecosystem on [`AdiAddrLed`]. This is useful if you need more advanced features for controlling
//! the strip, such as gradients or gamma correction.
//!
//! [WS2812B]: https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf
//! [74HCT125 buffer]: https://www.diodes.com/assets/Datasheets/74HCT125.pdf
//! [`smart-leds-trait`]: https://docs.rs/smart-leds-trait/0.3.0/smart_leds_trait/index.html
//! [`SmartLedsWrite`]: https://docs.rs/smart-leds-trait/0.3.0/smart_leds_trait/trait.SmartLedsWrite.html

use snafu::Snafu;
use vex_sdk::vexDeviceAdiAddrLedSet;

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};
use crate::color::Color;

/// WS2812B Addressable LED Strip
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAddrLed<const N: usize> {
    port: AdiPort,
}

impl<const N: usize> AdiAddrLed<N> {
    /// The max number of LED diodes on one strip that a single ADI port can control.
    pub const MAX_LENGTH: usize = 64;

    /// Initialize an LED strip on an ADI port with a given number of diodes.
    ///
    /// # Errors
    ///
    /// If the `length` parameter exceeds [`Self::MAX_LENGTH`], the function returns
    /// [`AddrLedError::BufferTooLarge`].
    #[must_use]
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    fn update(&mut self, buf: &[u32], offset: usize) {
        let buf = &buf[offset..buf.len().max(N)];

        unsafe {
            vexDeviceAdiAddrLedSet(
                self.port.device_handle(),
                self.port.index(),
                buf.as_ptr().cast_mut(),
                offset as _,
                buf.len() as _,
                Default::default(),
            );
        }
    }

    /// Set the entire led strip to one color.
    ///
    /// # Errors
    ///
    /// If the ADI device could not be accessed, [`AddrLedError::Port`] is returned.
    pub fn set_all(&mut self, color: impl Into<Color>) -> Result<(), AddrLedError> {
        _ = self.set_buffer([color.into(); N])?;
        Ok(())
    }

    /// Sets an individual diode color on the strip.
    ///
    /// # Errors
    ///
    /// - Returns [`AddrLedError::OutOfRange`] if the provided index is out of range of the current
    ///   buffer length.
    /// - If the ADI device could not be accessed, [`AddrLedError::Port`] is returned.
    pub fn set_pixel(&mut self, index: usize, color: impl Into<Color>) -> Result<(), AddrLedError> {
        self.port.validate_expander()?;

        if index > N {
            OutOfRangeSnafu { index, length: N }.fail()?;
        }

        self.update(&[color.into().into_raw()], index);
        Ok(())
    }

    /// Attempt to write an iterator of colors to the LED strip. Returns how many colors were
    /// actually written.
    ///
    /// # Errors
    ///
    /// If the ADI device could not be accessed, [`AddrLedError::Port`] is returned.
    pub fn set_buffer<T, I>(&mut self, iter: T) -> Result<usize, AddrLedError>
    where
        T: IntoIterator<Item = I>,
        I: Into<Color>,
    {
        self.port.validate_expander()?;

        let mut buf = [0; N];
        let mut len = 0;

        for (pixel, color) in buf.iter_mut().zip(iter.into_iter()) {
            *pixel = color.into().into_raw();
            len += 1;
        }

        self.update(&buf, 0);

        Ok(len)
    }
}

impl<const N: usize> AdiDevice<1> for AdiAddrLed<N> {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalOut
    }
}

impl<const N: usize> smart_leds_trait::SmartLedsWrite for AdiAddrLed<N> {
    type Error = AddrLedError;
    type Color = Color;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.port.validate_expander()?;

        let mut buf = [0; N];

        for (pixel, color) in buf.iter_mut().zip(iterator.into_iter()) {
            *pixel = color.into().into_raw();
        }

        self.update(&buf, 0);

        Ok(())
    }
}

/// Errors that can occur when interacting with an [`AdiAddrLed`] strip.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum AddrLedError {
    /// The provided index was not in range of the current buffer's length.
    #[snafu(display("Index `{index}` is out of range for buffer of length `{length}`"))]
    OutOfRange {
        /// The index that was out of range
        index: usize,
        /// The length of the buffer
        length: usize,
    },

    /// The length of the provided buffer exceeded the maximum strip length (of 64) that ADI can
    /// control.
    #[snafu(display("Buffer length `{length}` exceeds maximum strip length of `64`"))]
    BufferTooLarge {
        /// The length of the buffer that was too large
        length: usize,
    },

    /// Generic ADI related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error
        source: PortError,
    },
}
