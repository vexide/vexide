//! Connect to VEXLink for robot-to-robot communication.

use core::ffi::CStr;

use alloc::{ffi::CString, string::String};
use no_std_io::io;
use pros_sys::{link::E_LINK_RECEIVER, link_receive, link_transmit, E_LINK_TRANSMITTER};
use snafu::Snafu;

use crate::error::{bail_errno, bail_on, map_errno, FromErrno, PortError};

pub trait Link {
    fn port(&self) -> u8;
    fn id(&self) -> &CStr;
    fn connected(&self) -> bool {
        unsafe { pros_sys::link_connected(self.port()) }
    }
    fn new(port: u8, id: String, vexlink_override: bool) -> Result<Self, LinkError>
    where
        Self: Sized;
}

pub struct RxLink {
    port: u8,
    id: CString,
}

impl RxLink {
    pub fn num_incoming_bytes(&self) -> Result<u32, LinkError> {
        let num = unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                pros_sys::link_raw_receivable_size(self.port)
            )
        };

        Ok(num)
    }

    pub fn clear_incoming_buf(&self) -> Result<(), LinkError> {
        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                pros_sys::link_clear_receive_buf(self.port)
            )
        };

        Ok(())
    }

    pub fn receive(&self, buf: &mut [u8]) -> Result<u32, LinkError> {
        const PROS_ERR_U32: u32 = pros_sys::PROS_ERR as _;

        match unsafe { link_receive(self.port, buf.as_mut_ptr().cast(), buf.len() as _) } {
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
    fn port(&self) -> u8 {
        self.port
    }
    fn new(port: u8, id: String, vexlink_override: bool) -> Result<Self, LinkError> {
        let id = CString::new(id).unwrap();
        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                if vexlink_override {
                    pros_sys::link_init(port, id.as_ptr().cast(), E_LINK_RECEIVER)
                } else {
                    pros_sys::link_init_override(port, id.as_ptr().cast(), E_LINK_RECEIVER)
                }
            )
        };
        Ok(Self { port, id })
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

pub struct TxLink {
    port: u8,
    id: CString,
}

impl TxLink {
    // I have literally no idea what the purpose of this is,
    // there is no way to push to the transmission buffer without transmitting it.
    pub fn num_outgoing_bytes(&self) -> Result<u32, LinkError> {
        let num = unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                pros_sys::link_raw_transmittable_size(self.port)
            )
        };

        Ok(num)
    }

    pub fn transmit(&self, buf: &[u8]) -> Result<u32, LinkError> {
        const PROS_ERR_U32: u32 = pros_sys::PROS_ERR as _;

        match unsafe { link_transmit(self.port, buf.as_ptr().cast(), buf.len() as _) } {
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
    fn port(&self) -> u8 {
        self.port
    }
    fn new(port: u8, id: String, vexlink_override: bool) -> Result<Self, LinkError> {
        let id = CString::new(id).unwrap();
        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                if vexlink_override {
                    pros_sys::link_init(port, id.as_ptr().cast(), E_LINK_TRANSMITTER)
                } else {
                    pros_sys::link_init_override(port, id.as_ptr().cast(), E_LINK_TRANSMITTER)
                }
            )
        };
        Ok(Self { port, id })
    }
}

#[derive(Debug, Snafu)]
pub enum LinkError {
    #[snafu(display("No link is connected through the radio."))]
    NoLink,
    #[snafu(display("The transmitter buffer is still busy with a previous transmission, and there is no room in the FIFO buffer (queue) to transmit the data."))]
    BufferBusyFull,
    #[snafu(display("The data given was a C NULL."))]
    NullData,
    #[snafu(display("Protocol error related to start byte, data size, or checksum during a transmission or reception."))]
    Protocol,
    #[snafu(display("The link is busy."))]
    Busy,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
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
