use alloc::vec::Vec;

use pros_sys::{ext_adi_led_t, PROS_ERR};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::{devices::smart::vision::Rgb, error::bail_on};

#[derive(Debug, Eq, PartialEq)]
pub struct AdiAddrLed {
    raw: ext_adi_led_t,
    buffer: Vec<u32>,
    port: AdiPort,
}

impl AdiAddrLed {
    pub fn new(port: AdiPort) -> Result<Self, AdiError> {
        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_init(port.internal_expander_index(), port.index())
        });

        Ok(Self {
            port,
            raw,
            buffer: Vec::new(),
        })
    }

    pub fn clear_all(&mut self) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_led_clear_all(
                self.raw,
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
            )
        });

        Ok(())
    }

    pub fn set_all(&mut self, color: Rgb) -> Result<(), AdiError> {
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

    pub fn set_buffer<T, I>(&mut self, buffer: T) -> Result<(), AdiError>
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

    pub fn set_pixel(&mut self, index: u32, color: Rgb) -> Result<(), AdiError> {
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

    pub fn clear_pixel(&mut self, index: u32) -> Result<(), AdiError> {
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
    type Error = AdiError;
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
