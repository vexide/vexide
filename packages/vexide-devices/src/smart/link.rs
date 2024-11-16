//! VEXlink
//!
//! This module provides support for VEXlink, a point-to-point wireless communications protocol between
//! two VEXnet radios.
//!
//! # Hardware Overview
//!
//! There are two types of radios in a VEXnink connection: "manager" and "worker". A "manager" radio can transmit data at up to 1040 bytes/s
//! while a "worker" radio can transmit data at up to 520 bytes/s.
//! A connection should only ever have both types of radios.
//!
//! In order to connect to a radio, VEXos hashes a given link name and uses it as an ID to verify the connection.
//! For this reason, you should try to create a unique name for each radio link
//! to avoid accidentally interfering, or being interfered with by, an unrelated VEXlink connection.
//! Ideally, you want a name that will never be used by another team.
//!
//! The lights on the radio can be used as a status indicator:
//! - Blinking red: The radio is waiting for a connection to be established.
//! - Alternating red and green quickly: The radio is connected to another radio and is the "manager" radio.
//! - Alternating red and green slowly: The radio is connected to another radio and is the "worker" radio.
//!
//! For further information, see <https://www.vexforum.com/t/vexlink-documentaton/84538>

use alloc::ffi::{CString, NulError};
use core::time::Duration;

use no_std_io::io;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceGenericRadioConnection, vexDeviceGenericRadioLinkStatus, vexDeviceGenericRadioReceive,
    vexDeviceGenericRadioReceiveAvail, vexDeviceGenericRadioTransmit,
    vexDeviceGenericRadioWriteFree, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// VEXLink Wireless Radio Link
///
/// VEXLink is a point-to-point wireless communications protocol between
/// two VEXNet radios. For further information, see <https://www.vexforum.com/t/vexlink-documentaton/84538>
#[derive(Debug, Eq, PartialEq)]
pub struct RadioLink {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for RadioLink {}
unsafe impl Sync for RadioLink {}

impl RadioLink {
    /// Opens a radio link from a VEXNet radio plugged into a Smart Port. Once
    /// opened, other VEXNet functionality such as controller tethering on this
    /// specific radio will be disabled.
    /// Other radios connected to the Brain can take over this functionality.
    ///
    /// # Errors
    ///
    /// - A [`LinkError::Port`] error is returned if a radio device is not currently connected to the specified port.
    /// - A [`LinkError::Nul`] error is returned if a NUL (0x00) character was found anywhere in the specified `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let link = RadioLink::open(port_1, "643A", LinkType::Manager).unwrap();
    /// }
    /// ```
    pub fn open(port: SmartPort, id: &str, link_type: LinkType) -> Result<Self, LinkError> {
        // Ensure that a radio is plugged into the requested port.
        //
        // Once we call [`vexDeviceGenericRadioConnection`], this type
        // will be changed to be generic serial, but we haven't called
        // it yet.
        port.validate_type(SmartDeviceType::Radio)?;

        let id = CString::new(id)?;

        // That this constructor literally has to be fallible unlike others.
        unsafe {
            vexDeviceGenericRadioConnection(
                port.device_handle(),
                id.as_ptr().cast_mut(),
                match link_type {
                    LinkType::Worker => 0,
                    LinkType::Manager => 1,
                },
                true,
            );
        }

        Ok(Self {
            device: unsafe { port.device_handle() },
            port,
        })
    }

    /// Returns the number of bytes that are waiting to be read from the radio's input buffer.
    ///
    /// # Errors
    ///
    /// - A [`LinkError::Port`] error is returned if a radio device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut link = RadioLink::open(port_1, "643A", LinkType::Manager).unwrap();
    ///
    ///     let mut buffer = vec![0; 2048];
    ///
    ///     // Read into `buffer` if there are unread bytes.
    ///     if link.unread_bytes().is_ok_and(|bytes| bytes > 0) {
    ///         _ = link.read(&mut buffer);
    ///     }
    /// }
    /// ```
    pub fn unread_bytes(&self) -> Result<usize, LinkError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGenericRadioReceiveAvail(self.device) } as usize)
    }

    /// Returns the number of bytes free in the radio's output buffer.
    ///
    /// # Errors
    ///
    /// - A [`LinkError::Port`] error is returned if a radio device is not currently connected to the Smart Port.
    /// - A [`LinkError::ReadFailed`] error is returned if the output buffer could not be accessed.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut link = RadioLink::open(port_1, "643A", LinkType::Manager).unwrap();
    ///
    ///     // Write a byte if there's free space in the buffer.
    ///     if link.available_write_bytes().is_ok_and(|available| available > 0) {
    ///         _ = link.write(0x80);
    ///     }
    /// }
    /// ```
    pub fn available_write_bytes(&self) -> Result<usize, LinkError> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericRadioWriteFree(self.device) } {
            // TODO: This check may not be necessary, since PROS doesn't do it,
            //		 but we do it just to be safe.
            -1 => Err(LinkError::ReadFailed),
            available => Ok(available as usize),
        }
    }

    /// Returns `true` if there is a link established with another radio.
    ///
    /// # Errors
    ///
    /// - A [`LinkError::Port`] error is returned if a radio device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut link = RadioLink::open(port_1, "643A", LinkType::Manager).unwrap();
    ///
    ///     // Write a byte if we are connected to another radio.
    ///     if link.is_linked() == Ok(true) {
    ///         _ = link.write(0x80);
    ///     }
    /// }
    /// ```
    pub fn is_linked(&self) -> Result<bool, LinkError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGenericRadioLinkStatus(self.device) })
    }
}

const RADIO_NOT_LINKED: &str = "The radio has not established a link with another radio.";

impl io::Read for RadioLink {
    /// Read some bytes sent to the radio into the specified buffer, returning how many bytes were read.
    ///
    ///
    /// # Errors
    ///
    /// - An error with the kind [`io::ErrorKind::AddrNotAvailable`] is returned if there is no device connected.
    /// - An error with the kind [`io::ErrorKind::AddrInUse`] is returned a device other than a radio is connected.
    /// - An error with the kind [`io::ErrorKind::NotConnected`] is returned if a connection with another radio has not been
    ///   established. Use [`RadioLink::is_linked`] to check this if needed.
    /// - An error with the kind [`io::ErrorKind::Other`] is returned if an unexpected internal read error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut link = RadioLink::open(port_1, "643A", LinkType::Manager).unwrap();
    ///
    ///     let mut buffer = vec![0; 2048];
    ///
    ///     loop {
    ///         _ = link.read(&mut buffer);
    ///         sleep(core::time::Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let is_linked = self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => source,
            _ => unreachable!(),
        })?;

        if !is_linked {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                RADIO_NOT_LINKED,
            ));
        }

        match unsafe {
            vexDeviceGenericRadioReceive(self.device, buf.as_mut_ptr(), buf.len() as u16)
        } {
            -1 => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Internal read error occurred.",
            )),
            received => Ok(received as usize),
        }
    }
}

impl io::Write for RadioLink {
    /// Write a buffer into the radio's output buffer, returning how many bytes were written.
    ///
    /// # Errors
    ///
    /// - An error with the kind [`io::ErrorKind::AddrNotAvailable`] is returned if there is no device connected.
    /// - An error with the kind [`io::ErrorKind::AddrInUse`] is returned a device other than a radio is connected.
    /// - An error with the kind [`io::ErrorKind::NotConnected`] is returned if a connection with another radio has not been
    ///   established. Use [`RadioLink::is_linked`] to check this if needed.
    /// - An error with the kind [`io::ErrorKind::Other`] is returned if an unexpected internal write error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut link = RadioLink::open(port_1, "643A", LinkType::Manager).unwrap();
    ///
    ///     _ = link.write(b"yo");
    /// }
    /// ```
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let is_linked = self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => source,
            _ => unreachable!(),
        })?;

        if !is_linked {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                RADIO_NOT_LINKED,
            ));
        }

        match unsafe { vexDeviceGenericRadioTransmit(self.device, buf.as_ptr(), buf.len() as u16) }
        {
            -1 => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal write error occurred.",
            )),
            written => Ok(written as usize),
        }
    }

    /// This function does nothing.
    ///
    /// VEXLink immediately sends and clears data sent into the write buffer.
    ///
    /// # Errors
    ///
    /// - An error with the kind [`io::ErrorKind::AddrNotAvailable`] is returned if there is no device connected.
    /// - An error with the kind [`io::ErrorKind::AddrInUse`] is returned a device other than a radio is connected.
    /// - An error with the kind [`io::ErrorKind::NotConnected`] is returned if a connection with another radio has not been
    ///   established. Use [`RadioLink::is_linked`] to check this if needed.
    /// - An error with the kind [`io::ErrorKind::Other`] is returned if the data could not be written to the radio.
    fn flush(&mut self) -> io::Result<()> {
        if !self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => source,
            _ => unreachable!(),
        })? {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                RADIO_NOT_LINKED,
            ));
        }

        Ok(())
    }
}

impl SmartDevice for RadioLink {
    const UPDATE_INTERVAL: Duration = Duration::from_millis(25);

    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::GenericSerial
    }
}
impl From<RadioLink> for SmartPort {
    fn from(device: RadioLink) -> Self {
        device.port
    }
}

/// The type of radio link being established.
///
/// VEXLink is a point-to-point connection, with one "manager" robot and
/// one "worker" robot.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LinkType {
    /// Manager Radio
    ///
    /// This end of the link has a 1040-bytes/sec data rate when
    /// communicating with a worker radio.
    Manager,

    /// Worker Radio
    ///
    /// This end of the link has a 520-bytes/sec data rate when
    /// communicating with a manager radio.
    Worker,
}

/// Errors that can occur when interacting with a [`RadioLink`].
#[derive(Debug, Snafu)]
pub enum LinkError {
    /// Not linked with another radio.
    NotLinked,

    /// Internal write error occurred.
    WriteFailed,

    /// Internal read error occurred.
    ReadFailed,

    /// A NUL (0x00) character was found in a string that may not contain NUL characters.
    #[snafu(transparent)]
    Nul {
        /// The source of the error.
        source: NulError,
    },

    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
