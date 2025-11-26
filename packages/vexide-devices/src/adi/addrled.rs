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

    /// Initializes an LED strip with a given length on an ADI port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new LED strip with 8 addressable pixels.
    ///     let mut leds = AdiAddrLed::<8>::new(peripherals.adi_a);
    /// }
    /// ```
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        const {
            assert!(
                N <= Self::MAX_LENGTH,
                "AdiAddrLed strip size exceeded MAX_LENGTH (64)"
            );
        }

        port.configure(AdiDeviceType::DigitalOut);

        Self { port }
    }

    fn update(&mut self, buf: &[u32], offset: usize) {
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

    /// Set the entire LED strip to one color.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{color::Color, prelude::*};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new LED strip with 8 addressable pixels.
    ///     let mut leds = AdiAddrLed::<8>::new(peripherals.adi_a);
    ///
    ///     // Set all pixels to white.
    ///     _ = leds.set_all(Color::WHITE);
    /// }
    /// ```
    pub fn set_all(&mut self, color: impl Into<Color>) -> Result<(), PortError> {
        _ = self.set_buffer([color.into(); N])?;
        Ok(())
    }

    /// Sets the color of an individual diode on the strip.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of range for this strip (`index < N`).
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{color::Color, prelude::*};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new LED strip with 8 addressable pixels.
    ///     let mut leds = AdiAddrLed::<8>::new(peripherals.adi_a);
    ///
    ///     // Set the first pixel in the strip to white.
    ///     _ = leds.set_pixel(0, Color::WHITE);
    /// }
    /// ```
    pub fn set_pixel(&mut self, index: usize, color: impl Into<Color>) -> Result<(), PortError> {
        assert!(index < N, "pixel index was out of range for LED strip size");

        self.port.validate_expander()?;
        self.update(&[color.into().into_raw()], index);

        Ok(())
    }

    /// Attempt to write an iterator of colors to the LED strip. Returns how many colors were
    /// actually written.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{color::Color, prelude::*};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new LED strip with 8 addressable pixels.
    ///     let mut leds = AdiAddrLed::<8>::new(peripherals.adi_a);
    ///
    ///     // List of colors that each LED pixel will be set to.
    ///     let colors = [
    ///         Color::RED,
    ///         Color::ORANGE,
    ///         Color::YELLOW,
    ///         Color::GREEN,
    ///         Color::BLUE,
    ///         Color::PURPLE,
    ///         Color::RED,
    ///         Color::ORANGE,
    ///     ];
    ///
    ///     // Set the first pixel in the strip to white.
    ///     _ = leds.set_buffer(0, &colors);
    /// }
    /// ```
    pub fn set_buffer<T, I>(&mut self, iter: T) -> Result<usize, PortError>
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
    type Error = PortError;
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
