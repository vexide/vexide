use alloc::string::String;
use pros_sys::{link_receive, link_transmit};
use snafu::Snafu;

use crate::error::{bail_on, map_errno, FromErrno, PortError};

#[derive(Clone, Copy)]
#[repr(u32)]
enum LinkType {
    Receiver = pros_sys::E_LINK_RECEIVER,
    Transmitter = pros_sys::E_LINK_TRANSMITTER,
}

pub struct Link<const RECEIVER: bool> {
    pub port: u8,
    pub id: String,
}

impl<const RECEIVER: bool> Link<{ RECEIVER }> {
    pub fn new(port: u8, id: String, vexlink_override: bool) -> Result<Self, LinkError> {
        let link_type = match RECEIVER {
            true => LinkType::Receiver,
            false => LinkType::Transmitter,
        };

        unsafe {
            bail_on!(
                pros_sys::PROS_ERR as _,
                if vexlink_override {
                    pros_sys::link_init(port, id.as_ptr().cast(), link_type as _)
                } else {
                    pros_sys::link_init_override(port, id.as_ptr().cast(), link_type as _)
                }
            )
        };
        Ok(Self { port, id })
    }

    pub fn connected(&self) -> bool {
        unsafe { pros_sys::link_connected(self.port) }
    }
}

impl Link<true> {
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
                let errno = crate::error::take_errno();
                Err(FromErrno::from_errno(errno)
                    .unwrap_or_else(|| panic!("Unknown errno code {errno}")))
            }
            0 => Err(LinkError::Busy),
            n => Ok(n),
        }
    }
}

impl Link<false> {
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
impl core::error::Error for LinkError {}

map_errno! {
    LinkError {
        ENXIO => Self::NoLink,
        EBUSY => Self::BufferBusyFull,
        EINVAL => Self::NullData,
        EBADMSG => Self::Protocol,
    }
    inherit PortError;
}
