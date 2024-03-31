//! Generic serial device module.
//!
//! Provides support for using [`SmartPort`]s as generic serial communication devices.

use alloc::ffi::CString;

use no_std_io::io;
use pros_core::error::PortError;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceGenericRadioConnection, vexDeviceGenericRadioLinkStatus, vexDeviceGenericRadioReceive,
    vexDeviceGenericRadioReceiveAvail, vexDeviceGenericRadioTransmit,
    vexDeviceGenericRadioWriteFree,
};

use super::{validate_port, SmartDevice, SmartDeviceInternal, SmartDeviceType, SmartPort};

/// Represents a smart port configured as a VEXLink radio.
///
/// VEXLink is a point-to-point wireless communications protocol between
/// two VEXNet radios. For further information, see <https://www.vexforum.com/t/vexlink-documentaton/84538>
#[derive(Debug, Eq, PartialEq)]
pub struct RadioLink {
    port: SmartPort,
}

impl RadioLink {
    /// Opens a radio link from a VEXNet radio plugged into a smart port. Once
    /// opened, other VEXNet functionality such as controller tethering on this
    /// radio will be disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// let link = RadioLink::open(port_1, "643A", LinkType::Manager)?;
    /// ```
    pub fn open(port: SmartPort, id: &str, link_type: LinkType) -> Result<Self, LinkError> {
        let link = Self { port };

        // Ensure that a radio is plugged into the requested port.
        //
        // Once we call [`vexDeviceGenericRadioConnection`], this type
        // will be changed to be generic serial, but we haven't called
        // it yet.
        validate_port(link.port_index(), SmartDeviceType::Radio);

        unsafe {
            vexDeviceGenericRadioConnection(
                link.device_handle(),
                CString::new(id)
                    .map_err(|_| LinkError::NonTerminatingNul)?
                    .into_raw(),
                match link_type {
                    LinkType::Worker => 0,
                    LinkType::Manager => 1,
                },
                true,
            );
        }

        Ok(link)
    }

    /// Returns the number of bytes available to be read in the the radio's input buffer.
    pub fn unread_bytes(&self) -> Result<usize, LinkError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGenericRadioReceiveAvail(self.device_handle()) } as usize)
    }

    /// Returns the number of bytes free in the radio's output buffer.
    pub fn available_write_bytes(&self) -> Result<usize, LinkError> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericRadioWriteFree(self.device_handle()) } {
            // TODO: This check may not be necessary, since PROS doesn't do it,
            //		 but we do it just to be safe.
            -1 => Err(LinkError::ReadFailed),
            available => Ok(available as usize),
        }
    }

    /// Returns `true` if there is a link established with another radio.
    pub fn is_linked(&self) -> Result<bool, LinkError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGenericRadioLinkStatus(self.device_handle()) })
    }
}

impl io::Read for RadioLink {
    /// Read some bytes sent to the radio into the specified buffer, returning
    /// how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let is_linked = self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => match source {
                PortError::Disconnected => {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, "Port does not exist.")
                }
                PortError::IncorrectDevice => io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "Port is in use as another device.",
                ),
            },
            _ => unreachable!(),
        })?;

        if !is_linked {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Radio is not linked!",
            ));
        }

        match unsafe {
            vexDeviceGenericRadioReceive(self.device_handle(), buf.as_mut_ptr(), buf.len() as u16)
        } {
            -1 => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Internal read error occurred.",
            )),
            recieved => Ok(recieved as usize),
        }
    }
}

impl io::Write for RadioLink {
    /// Write a buffer into the radio's output buffer, returning how many bytes
    /// were written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let io_error_handler = |e| match e {
            LinkError::Port { source } => match source {
                PortError::Disconnected => {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, "Port does not exist.")
                }
                PortError::IncorrectDevice => io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "Port is in use as another device.",
                ),
            },
            _ => unreachable!(),
        };

        if !self.is_linked().map_err(io_error_handler)? {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Radio is not linked!",
            ));
        }

        if buf.len() > self.available_write_bytes().map_err(io_error_handler)? {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Buffer length exceeded available bytes in write buffer.",
            ));
        }

        match unsafe {
            vexDeviceGenericRadioTransmit(self.device_handle(), buf.as_mut_ptr(), buf.len() as u16)
        } {
            -1 => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal write error occurred.",
            )),
            0 => Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "Transmit function returned zero written bytes.",
            )),
            written => Ok(written as usize),
        }
    }

    /// This function does nothing.
    ///
    /// VEXLink immediately sends and clears data sent into the write buffer.
    fn flush(&mut self) -> io::Result<()> {
        if !self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => match source {
                PortError::Disconnected => {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, "Port does not exist.")
                }
                PortError::IncorrectDevice => io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "Port is in use as another device.",
                ),
            },
            _ => unreachable!(),
        })? {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Radio is not linked!",
            ));
        }

        Ok(())
    }
}

impl SmartDevice for RadioLink {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::GenericSerial
    }
}

/// The type of a radio link being established.
///
/// VEXLink is a point-to-point connection, with one "manager" robot and
/// one "worker" robot.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LinkType {
    /// Manager Radio
    ///
    /// This end of the link has a 1040 bytes/sec data rate when
    /// communicating with a worker radio.
    Manager,

    /// Worker Radio
    ///
    /// This end of the link has a 520 bytes/sec data rate when
    /// communicating with a manager radio.
    Worker,
}

/// Errors that can occur when interacting with a [`SerialPort`].
#[derive(Debug, Snafu)]
pub enum LinkError {
    /// Not linked with another radio.
    NotLinked,

    /// Internal write error occurred.
    WriteFailed,

    /// Internal read error occurred.
    ReadFailed,

    /// CString::new encountered NUL (U+0000) byte in non-terminating position.
    NonTerminatingNul,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
