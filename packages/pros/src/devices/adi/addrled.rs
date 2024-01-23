use alloc::vec::Vec;

use pros_sys::{ext_adi_led_t, PROS_ERR};
use snafu::Snafu;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::{
    devices::smart::vision::Rgb,
    error::{bail_on, map_errno},
};

#[derive(Debug, Eq, PartialEq)]
pub struct AdiAddrLed {
    raw: ext_adi_led_t,
    buffer: Vec<u32>,
    port: AdiPort,
}

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
    pub fn set_all(&mut self, color: Rgb) -> Result<(), AddrLedError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set_all(
                self.raw,
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
                u32::from(color),
            )
        });

        Ok(())
    }

    /// Set the entire led strip using the colors contained in a new buffer.
    pub fn set_buffer<T, I>(&mut self, buffer: T) -> Result<(), AddrLedError>
    where
        T: IntoIterator<Item = I>,
        I: Into<u32>,
    {
        self.buffer = buffer.into_iter().map(|i| i.into()).collect::<Vec<_>>();

        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set(self.raw, self.buffer.as_mut_ptr(), self.buffer.len() as u32)
        });

        Ok(())
    }

    /// Set one pixel on the led strip.
    pub fn set_pixel(&mut self, index: u32, color: Rgb) -> Result<(), AddrLedError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set_pixel(
                self.raw,
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
                u32::from(color),
                index,
            )
        });

        Ok(())
    }

    /// Clear one pixel on the LED strip.
    pub fn clear_pixel(&mut self, index: u32) -> Result<(), AddrLedError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_clear_pixel(
                self.raw,
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
                index,
            )
        });

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
        AdiDeviceType::LegacyPwm
    }
}

#[cfg(feature = "smart-leds-trait")]
impl smart_leds_trait::SmartLedsWrite for AdiAddrLed {
    type Error = AddrLedError;
    type Color = u32;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.buffer = iterator.into_iter().map(|i| i.into()).collect::<Vec<_>>();

        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_set(self.raw, self.buffer.as_mut_ptr(), self.buffer.len() as u32)
        });

        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum AddrLedError {
    #[snafu(display(
        "Failed to access LED buffer. A given value is not correct, or the buffer is null."
    ))]
    InvalidBufferAccess,

    #[snafu(display("{source}"), context(false))]
    Adi { source: AdiError },
}

map_errno! {
    AddrLedError {
        EINVAL => Self::InvalidBufferAccess,
    }
    inherit AdiError;
}
