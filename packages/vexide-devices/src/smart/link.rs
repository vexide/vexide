//! Generic serial device module.
//!
//! Provides support for using [`SmartPort`]s as generic serial communication devices.

use alloc::ffi::CString;
use core::future::Future;

use embedded_io_async::{ErrorKind, ErrorType, Read, Write};
use snafu::Snafu;
use vex_sdk::{
    vexDeviceGenericRadioConnection, vexDeviceGenericRadioLinkStatus, vexDeviceGenericRadioReceive,
    vexDeviceGenericRadioReceiveAvail, vexDeviceGenericRadioTransmit,
    vexDeviceGenericRadioWriteFree, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// Represents a smart port configured as a VEXLink radio.
///
/// VEXLink is a point-to-point wireless communications protocol between
/// two VEXNet radios. For further information, see <https://www.vexforum.com/t/vexlink-documentaton/84538>
#[derive(Debug, Eq, PartialEq)]
pub struct RadioLink {
    port: SmartPort,
    device: V5_DeviceT,
}
impl ErrorType for RadioLink {
    type Error = ErrorKind;
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
        // Ensure that a radio is plugged into the requested port.
        //
        // Once we call [`vexDeviceGenericRadioConnection`], this type
        // will be changed to be generic serial, but we haven't called
        // it yet.
        port.validate_type(SmartDeviceType::Radio)?;

        // That this constructor literally has to be fallible unlike others.
        unsafe {
            vexDeviceGenericRadioConnection(
                port.device_handle(),
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

        Ok(Self {
            device: unsafe { port.device_handle() },
            port,
        })
    }

    /// Returns the number of bytes available to be read in the the radio's input buffer.
    pub fn unread_bytes(&self) -> Result<usize, LinkError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGenericRadioReceiveAvail(self.device) } as usize)
    }

    /// Returns the number of bytes free in the radio's output buffer.
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
    pub fn is_linked(&self) -> Result<bool, LinkError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGenericRadioLinkStatus(self.device) })
    }
}

impl Read for RadioLink {
    /// Read some bytes sent to the radio into the specified buffer, returning
    /// how many bytes were read.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let is_linked = self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => match source {
                PortError::Disconnected => ErrorKind::AddrNotAvailable,
                PortError::IncorrectDevice => ErrorKind::AddrInUse,
            },
            _ => unreachable!(),
        })?;

        if !is_linked {
            return Err(ErrorKind::NotConnected);
        }

        match unsafe {
            vexDeviceGenericRadioReceive(self.device, buf.as_mut_ptr(), buf.len() as u16)
        } {
            -1 => Err(ErrorKind::InvalidData),
            recieved => Ok(recieved as usize),
        }
    }
}

struct RadioBufferFullFuture<'a> {
    device: &'a RadioLink,
}
impl<'a> Future for RadioBufferFullFuture<'a> {
    type Output = Result<(), ErrorKind>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if !self.device.is_linked().map_err(|e| match e {
            LinkError::Port { source } => match source {
                PortError::Disconnected => ErrorKind::AddrNotAvailable,
                PortError::IncorrectDevice => ErrorKind::AddrInUse,
            },
            _ => unreachable!(),
        })? {
            return core::task::Poll::Ready(Err(ErrorKind::NotConnected));
        }
        if unsafe { vexDeviceGenericRadioWriteFree(self.device.device) } == 0 {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(Ok(()))
        }
    }
}

impl Write for RadioLink {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        RadioBufferFullFuture {
            device: self,
        }
        .await?;

        match unsafe { vexDeviceGenericRadioTransmit(self.device, buf.as_ptr(), buf.len() as u16) }
        {
            -1 => Err(ErrorKind::Other),
            written => Ok(written as usize),
        }
    }

    /// This function does nothing.
    ///
    /// VEXLink immediately sends and clears data sent into the write buffer.
    async fn flush(&mut self) -> Result<(), Self::Error> {
        if !self.is_linked().map_err(|e| match e {
            LinkError::Port { source } => match source {
                PortError::Disconnected => ErrorKind::AddrNotAvailable,
                PortError::IncorrectDevice => ErrorKind::AddrInUse,
            },
            _ => unreachable!(),
        })? {
            return Err(ErrorKind::NotConnected);
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
