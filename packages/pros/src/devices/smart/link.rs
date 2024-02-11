//! Connect to VEXLink radios for robot-to-robot communication.
//!
//! There are two types of links: [`TxLink`] (transmitter radio module) and [`RxLink`] (receiver radio module).
//! both implement a shared trait [`Link`] as well as a no_std version of `Write` and `Read` from [`no_std_io`] respectively.

use alloc::{ffi::CString, string::String};
use core::ffi::CStr;

use no_std_io::io;
use pros_sys::{link_receive, link_transmit, E_LINK_RECEIVER, E_LINK_TRANSMITTER};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::error::{bail_errno, bail_on, map_errno, FromErrno, PortError};

/// Types that implement Link can be used to send data to another robot over VEXLink.
pub trait Link: SmartDevice {
    /// The identifier of this link.
    fn id(&self) -> &CStr;

    /// Check whether this link is connected to another robot.
    fn connected(&self) -> bool {
        unsafe { pros_sys::link_connected(self.port_index()) }
    }

    /// Create a new link ready to send or recieve data.
    fn new(port: SmartPort, id: String, vexlink_override: bool) -> Result<Self, LinkError>
    where
        Self: Sized;
}

/// A recieving end of a VEXLink connection.
#[derive(Debug)]
pub struct RxLink {
    port: SmartPort,
    id: CString,
}

impl RxLink {
    /// Get the number of bytes in the incoming buffer.
    /// Be aware that the number of incoming bytes can change between when this is called
    /// and when you read from the link.
    /// If you create a buffer of this size, and then attempt to read into it
    /// you may encounter panics or data loss.
    pub fn num_incoming_bytes(&self) -> Result<u32, LinkError> {
        let num = unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                pros_sys::link_raw_receivable_size(self.port.index())
            )
        };

        Ok(num)
    }

    /// Clear all bytes in the incoming buffer.
    /// All data in the incoming will be lost and completely unrecoverable.
    pub fn clear_incoming_buf(&self) -> Result<(), LinkError> {
        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                pros_sys::link_clear_receive_buf(self.port.index())
            )
        };

        Ok(())
    }

    /// Receive data from the link incoming buffer into a buffer.
    pub fn receive(&self, buf: &mut [u8]) -> Result<u32, LinkError> {
        const PROS_ERR_U32: u32 = pros_sys::PROS_ERR as _;

        match unsafe { link_receive(self.port.index(), buf.as_mut_ptr().cast(), buf.len() as _) } {
            PROS_ERR_U32 => {
                bail_errno!();
                unreachable!("Expected errno to be set");
            }
            0 => Err(LinkError::Busy),
            n => Ok(n),
        }
    }
}

impl Link for RxLink {
    fn id(&self) -> &CStr {
        &self.id
    }
    fn new(port: SmartPort, id: String, vexlink_override: bool) -> Result<Self, LinkError> {
        let id = CString::new(id).unwrap();
        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                if vexlink_override {
                    pros_sys::link_init(port.index(), id.as_ptr().cast(), E_LINK_RECEIVER)
                } else {
                    pros_sys::link_init_override(port.index(), id.as_ptr().cast(), E_LINK_RECEIVER)
                }
            )
        };
        Ok(Self { port, id })
    }
}

impl SmartDevice for RxLink {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Radio
    }
}

impl io::Read for RxLink {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self
            .receive(dst)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to read from link"))?;
        Ok(bytes_read as _)
    }
}

/// A transmitting end of a VEXLink connection.
#[derive(Debug)]
pub struct TxLink {
    port: SmartPort,
    id: CString,
}

impl TxLink {
    // I have literally no idea what the purpose of this is,
    // there is no way to push to the transmission buffer without transmitting it.
    /// Get the number of bytes to be sent over this link.
    pub fn num_outgoing_bytes(&self) -> Result<u32, LinkError> {
        let num = unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                pros_sys::link_raw_transmittable_size(self.port.index())
            )
        };

        Ok(num)
    }

    /// Transmit a buffer of data over the link.
    pub fn transmit(&self, buf: &[u8]) -> Result<u32, LinkError> {
        const PROS_ERR_U32: u32 = pros_sys::PROS_ERR as _;

        match unsafe { link_transmit(self.port.index(), buf.as_ptr().cast(), buf.len() as _) } {
            PROS_ERR_U32 => {
                let errno = crate::error::take_errno();
                Err(FromErrno::from_errno(errno)
                    .unwrap_or_else(|| panic!("Unknown errno code {errno}")))
            }
            0 => Err(LinkError::Busy),
            n => Ok(n),
        }
    }
}

impl io::Write for TxLink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written = self
            .transmit(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to write to link"))?;
        Ok(bytes_written as _)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Link for TxLink {
    fn id(&self) -> &CStr {
        &self.id
    }
    fn new(port: SmartPort, id: String, vexlink_override: bool) -> Result<Self, LinkError> {
        let id = CString::new(id).unwrap();
        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                if vexlink_override {
                    pros_sys::link_init(port.index(), id.as_ptr().cast(), E_LINK_TRANSMITTER)
                } else {
                    pros_sys::link_init_override(
                        port.index(),
                        id.as_ptr().cast(),
                        E_LINK_TRANSMITTER,
                    )
                }
            )
        };
        Ok(Self { port, id })
    }
}

impl SmartDevice for TxLink {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Radio
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using VEXLink.
pub enum LinkError {
    /// No link is connected through the radio.
    NoLink,
    /// The transmitter buffer is still busy with a previous transmission, and there is no room in the FIFO buffer (queue) to transmit the data.
    BufferBusyFull,
    /// Invalid data: the data given was a C NULL.
    NullData,
    /// Protocol error related to start byte, data size, or checksum during a transmission or reception.
    Protocol,
    /// The link is busy.
    Busy,
    #[snafu(display("{source}"), context(false))]
    /// Generic port related error
    Port {
        /// The source of the error
        source: PortError,
    },
}

map_errno! {
    LinkError {
        ENXIO => Self::NoLink,
        EBUSY => Self::BufferBusyFull,
        EINVAL => Self::NullData,
        EBADMSG => Self::Protocol,
    }
    inherit PortError;
}
