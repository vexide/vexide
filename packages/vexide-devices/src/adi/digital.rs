//! ADI Digital I/O
//!
//! ADI ports on the V5 brain are capable of sending and receiving digital signals
//! with external devices. Digital signals represent binary information using voltage levels
//! (called [logic levels](`LogicLevel`)) - they can only be in one of two states at any time.
//! Unlike analog signals which can take on any voltage within a range, digital signals are
//! either fully "on" (high) or fully "off" (low), making them ideal for simple sensors and
//! actuators such as buttons, switches and solenoids.
//!
//! # Hardware Description
//!
//! The ADI (Analog/Digital Interface) ports can be configured as either digital inputs or
//! outputs. When configured as inputs, they detect voltage levels to determine a logical high
//! (3.3V or above) or low (below 3.3V) state. When configured as outputs, they drive the pin
//! to either 3.3V (high) or 0V (low). These ports can be used with both direct brain connections
//! and through an ADI expander module.

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Logic level of a digital pin.
///
/// On digital devices, logic levels represent the two possible voltage signals that define
/// the state of a port. This value is either [`High`](LogicLevel::High) or [`Low`](LogicLevel::Low)
/// depending on the intended state of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicLevel {
    /// A high digital signal.
    ///
    /// ADI ports operate on 3.3V logic, so this value indicates a voltage of 3.3V or above.
    High,

    /// A low digital signal.
    ///
    /// ADI ports operate on 3.3V logic, so this value indicates a voltage below 3.3V.
    Low,
}

impl LogicLevel {
    /// Returns `true` if the level is [`High`](LogicLevel::High).
    #[must_use]
    pub const fn is_high(&self) -> bool {
        matches!(self, Self::High)
    }

    /// Returns `true` if the level is [`Low`](LogicLevel::Low).
    #[must_use]
    pub const fn is_low(&self) -> bool {
        matches!(self, Self::Low)
    }
}

impl core::ops::Not for LogicLevel {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Low => Self::High,
            Self::High => Self::Low,
        }
    }
}

/// Generic Digital Input over ADI
///
/// Represents an ADI port configured to receive digital input. The pin can be read to
/// determine its current [logic level](`LogicLevel`) (above or below 3.3V).
#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalIn {
    port: AdiPort,
}

impl AdiDigitalIn {
    /// Create a digital input from an ADI port.
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalIn);

        Self { port }
    }

    /// Returns the current logic level of a digital input pin.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    pub fn level(&self) -> Result<LogicLevel, PortError> {
        self.port.validate_expander()?;

        let value =
            unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returns `true` if the digital input's logic level is [`LogicLevel::High`].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    pub fn is_high(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }

    /// Returns `true` if the digital input's logic level is [`LogicLevel::Low`].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    pub fn is_low(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_low())
    }
}

impl AdiDevice<1> for AdiDigitalIn {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalIn
    }
}

/// Generic digital output over ADI.
///
/// Represents an ADI port configured to send digital signals to a device. This can be
/// used for toggling solenoids or other external devices that might need a digital signal
/// from the brain.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalOut {
    port: AdiPort,
}

impl AdiDigitalOut {
    /// Create a digital output from an [`AdiPort`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::new(peripherals.adi_a);
    ///
    ///     // Toggle the digital output every second
    ///     loop {
    ///         _ = digital_out.toggle();
    ///         sleep(Duration::from_millis(1000)).await;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalOut);

        Self { port }
    }

    /// Create a digital output from an [`AdiPort`] with an initial logic level.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::with_initial_level(peripherals.adi_a, LogicLevel::High);
    ///
    ///     // The digital output is now set to high
    ///     assert_eq!(digital_out.level().expect("couldn't get level"), LogicLevel::High);
    /// }
    /// ```
    #[must_use]
    pub fn with_initial_level(port: AdiPort, initial_level: LogicLevel) -> Self {
        port.configure(AdiDeviceType::DigitalOut);

        unsafe {
            vexDeviceAdiValueSet(
                port.device_handle(),
                port.index(),
                i32::from(initial_level.is_high()),
            );
        }

        Self { port }
    }

    /// Sets the digital logic level (high or low) of a pin.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::new(peripherals.adi_a);
    ///
    ///     // Set the digital output to high
    ///     _ = digital_out.set_level(LogicLevel::High);
    ///
    ///     // Let's check if the universe isn't broken
    ///     assert_eq!(digital_out.level().expect("couldn't get level"), LogicLevel::High);
    /// }
    /// ```
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.index(),
                i32::from(level.is_high()),
            );
        }

        Ok(())
    }

    /// Returns the current set logic level of a digital output pin.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::new(peripherals.adi_a);
    ///
    ///     assert_eq!(digital_out.level().expect("couldn't get level"), LogicLevel::Low);
    /// }
    /// ```
    pub fn level(&self) -> Result<LogicLevel, PortError> {
        self.port.validate_expander()?;

        let value =
            unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returns `true` if the port's logic level is set to [`LogicLevel::High`].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    pub fn is_high(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }

    /// Returns `true` if the port's logic level is set to [`LogicLevel::Low`].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    pub fn is_low(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_low())
    }

    /// Set the digital logic level to [`LogicLevel::High`]. Analogous to
    /// [`Self::set_level(LogicLevel::High)`](Self::set_level).
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::new(peripherals.adi_a);
    ///
    ///     // Set the digital output to high
    ///     _ = digital_out.set_high();
    ///
    ///     // Let's check if the universe isn't broken
    ///     assert_eq!(digital_out.level().expect("couldn't get level"), LogicLevel::High);
    /// }
    /// ```
    pub fn set_high(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::High)
    }

    /// Set the digital logic level to [`LogicLevel::Low`]. Analogous to [`Self::set_level(LogicLevel::Low)`](Self::set_level).
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::new(peripherals.adi_a);
    ///
    ///     // Set the digital output to low
    ///     _ = digital_out.set_low();
    ///
    ///     // Let's check if the universe isn't broken
    ///     assert_eq!(digital_out.level().expect("couldn't get level"), LogicLevel::High);
    /// }
    /// ```
    pub fn set_low(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::Low)
    }

    /// Sets the digital logic level to the inverse of its previous state.
    ///
    /// - If the port was previously set to [`LogicLevel::Low`], then the level will be set to [`LogicLevel::High`].
    /// - If the port was previously set to [`LogicLevel::High`], then the level will be set to [`LogicLevel::Low`].
    ///
    /// This is analogous to `self.set_level(!self.level()?)?` and is useful for toggling devices like solenoids.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::new(peripherals.adi_a);
    ///
    ///     // Toggle the digital output every second
    ///     loop {
    ///         _ = digital_out.toggle();
    ///         sleep(Duration::from_millis(1000)).await;
    ///     }
    /// }
    /// ```
    pub fn toggle(&mut self) -> Result<(), PortError> {
        self.set_level(!self.level()?)
    }
}

impl AdiDevice<1> for AdiDigitalOut {
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
