//! Generic Serial Communication
//!
//! This module provides an interface for using V5 Smart Ports as serial communication
//! ports over RS-485. It allows bidirectional communication with any device that speaks
//! serial over the V5's RS-485 interface.
//!
//! # Hardware Description
//!
//! V5 Smart Ports provide half-duplex RS-485 serial communication at up to an allowed
//! 921600 baud for user programs.
//!
//! The ports supply 12.8V VCC nominally (VCC is wired directly to the V5's battery lines,
//! providing voltage somewhere in the range of 12-14V). Writes to the serial port are buffered,
//! but are automatically flushed by VEXos as fast as possible (down to ~10µs or so).

use core::{
    mem::ManuallyDrop,
    pin::Pin,
    task::{Context, Poll},
};

use snafu::Snafu;
use vex_sdk::{
    vexDeviceGenericSerialBaudrate, vexDeviceGenericSerialEnable, vexDeviceGenericSerialFlush,
    vexDeviceGenericSerialPeekChar, vexDeviceGenericSerialReadChar, vexDeviceGenericSerialReceive,
    vexDeviceGenericSerialReceiveAvail, vexDeviceGenericSerialTransmit,
    vexDeviceGenericSerialWriteChar, vexDeviceGenericSerialWriteFree, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};

/// A Smart Port configured as a generic RS-485 serial port.
///
/// This struct implements the [`Read`] and [`Write`] traits from vexide's `io` module
/// for reading/writing to the serial port.
///
/// [`Read`]: vexide_core::io::Read
/// [`Write`]: vexide_core::io::Write
#[derive(Debug, Eq, PartialEq)]
pub struct SerialPort {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for SerialPort {}
unsafe impl Sync for SerialPort {}

impl SerialPort {
    /// The maximum user-configurable baud rate for generic serial under normal conditions.
    pub const MAX_BAUD_RATE: u32 = 921_600;

    /// The length of the serial FIFO input and output buffers.
    pub const INTERNAL_BUFFER_SIZE: usize = 1024;

    /// Open and configure a generic serial port on a [`SmartPort`].
    ///
    /// This configures a [`SmartPort`] to act as a generic serial controller capable of sending/receiving
    /// data. Providing a baud rate, or the transmission rate of bits is required. The maximum allowed
    /// baud rate is 921600.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200).await;
    /// }
    /// ```
    pub fn open(port: SmartPort, baud_rate: u32) -> SerialPortOpenFuture {
        SerialPortOpenFuture {
            state: SerialPortOpenState::Configure { baud_rate },
            serial: ManuallyDrop::new(Self {
                device: unsafe { port.device_handle() },
                port,
            }),
        }
    }

    /// Configures the baud rate of the serial port.
    ///
    /// Baud rate determines the speed of communication over the data channel. Under normal conditions, user code is limited
    /// to a maximum baudrate of 921600.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     // Change to 9600 baud
    ///     serial.set_baud_rate(9600);
    /// }
    /// ```
    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        unsafe {
            vexDeviceGenericSerialBaudrate(self.device, baud_rate as i32);
        }
    }

    /// Clears the internal input and output FIFO buffers.
    ///
    /// This can be useful to reset state and remove old, potentially unneeded data
    /// from the input FIFO buffer or to cancel sending any data in the output FIFO
    /// buffer.
    ///
    /// # This is not the same thing as "flushing".
    ///
    /// This function does not cause the data in the output buffer to be
    /// written. It simply clears the internal buffers. Unlike stdout, generic
    /// serial does not use buffered IO (the FIFO buffers are written as soon
    /// as possible).
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     _ = serial.clear_buffers();
    ///     _ = serial.write(b"Buffers are clear!");
    /// }
    /// ```
    pub fn clear_buffers(&mut self) {
        unsafe {
            vexDeviceGenericSerialFlush(self.device);
        }
    }

    /// Read the next byte available in the serial port's input buffer, or `None` if the input buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     loop {
    ///         if let Some(byte) = serial.read_byte() {
    ///             println!("Got byte: {}", byte);
    ///         }
    ///
    ///         sleep(core::time::Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    pub fn read_byte(&mut self) -> Option<u8> {
        let byte = unsafe { vexDeviceGenericSerialReadChar(self.device) };

        match byte {
            -1 => None,
            _ => Some(byte as u8),
        }
    }

    /// Read the next byte available in the port's input buffer without removing it. Returns
    /// `None` if the input buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     if let Some(next_byte) = serial.peek_byte() {
    ///         println!("Next byte: {}", next_byte);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn peek_byte(&self) -> Option<u8> {
        match unsafe { vexDeviceGenericSerialPeekChar(self.device) } {
            -1 => None,
            byte => Some(byte as u8),
        }
    }

    /// Write a single byte to the port's output buffer.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::WriteFailed`] error is returned if the byte could not be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     // Attempt to write 0x80 (128u8) to the output buffer
    ///     _ = serial.write_byte(0x80);
    /// }
    /// ```
    pub fn write_byte(&mut self, byte: u8) -> Result<(), SerialError> {
        match unsafe { vexDeviceGenericSerialWriteChar(self.device, byte) } {
            -1 => WriteFailedSnafu.fail(),
            _ => Ok(()),
        }
    }

    /// Returns the number of bytes available to be read in the port's FIFO input buffer.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::ReadFailed`] error is returned if the serial device's status could not be read.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     if serial.unread_bytes().is_ok_and(|bytes| bytes > 0) {
    ///         if let Ok(byte) = serial.read_byte() {
    ///             // Okay to unwrap here, since we've established that there was at least one byte to read.
    ///             println!("{}", byte.unwrap());
    ///         }
    ///     }
    /// }
    /// ```
    pub fn unread_bytes(&self) -> Result<usize, SerialError> {
        match unsafe { vexDeviceGenericSerialReceiveAvail(self.device) } {
            -1 => ReadFailedSnafu.fail(),
            available => Ok(available as usize),
        }
    }

    /// Returns the number of bytes free in the port's FIFO output buffer.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::ReadFailed`] error is returned if the serial device's status could not be read.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200).await;
    ///
    ///     // Write a byte if there's free space in the buffer.
    ///     if serial.available_write_bytes().is_ok_and(|available| available > 0) {
    ///         _ = serial.write_byte(0x80);
    ///     }
    /// }
    /// ```
    pub fn available_write_bytes(&self) -> Result<usize, SerialError> {
        match unsafe { vexDeviceGenericSerialWriteFree(self.device) } {
            -1 => ReadFailedSnafu.fail(),
            available => Ok(available as usize),
        }
    }
}

#[cfg(feature = "std")]
impl std::io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match unsafe {
            vexDeviceGenericSerialReceive(self.device, buf.as_mut_ptr(), buf.len() as i32)
        } {
            -1 => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal read error occurred.",
            )),
            received => Ok(received as usize),
        }
    }
}

#[cfg(feature = "std")]
impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match unsafe { vexDeviceGenericSerialTransmit(self.device, buf.as_ptr(), buf.len() as i32) }
        {
            -1 => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal write error occurred.",
            )),
            written => Ok(written as usize),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "embedded-io")]
impl embedded_io::ErrorType for SerialPort {
    type Error = SerialError;
}

#[cfg(feature = "embedded-io")]
impl embedded_io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SerialError> {
        match unsafe {
            vexDeviceGenericSerialReceive(self.device, buf.as_mut_ptr(), buf.len() as i32)
        } {
            -1 => Err(SerialError::ReadFailed),
            received => Ok(received as usize),
        }
    }
}

#[cfg(feature = "embedded-io")]
impl embedded_io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SerialError> {
        match unsafe { vexDeviceGenericSerialTransmit(self.device, buf.as_ptr(), buf.len() as i32) }
        {
            -1 => Err(SerialError::WriteFailed),
            written => Ok(written as usize),
        }
    }

    fn flush(&mut self) -> Result<(), SerialError> {
        Ok(())
    }
}

impl SmartDevice for SerialPort {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::GenericSerial
    }
}
impl From<SerialPort> for SmartPort {
    fn from(device: SerialPort) -> Self {
        device.port
    }
}

/// Errors that can occur when interacting with a [`SerialPort`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum SerialError {
    /// Internal write error occurred.
    WriteFailed,

    /// Internal read error occurred.
    ReadFailed,
}

#[cfg(feature = "embedded-io")]
impl embedded_io::Error for SerialError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

#[derive(Debug, Clone, Copy)]
enum SerialPortOpenState {
    Configure { baud_rate: u32 },
    Waiting,
}

/// Future that opens and configures a [`SerialPort`].
///
/// If the port was not previous configured as a generic serial port, this may
/// take a few milliseconds to complete.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
pub struct SerialPortOpenFuture {
    state: SerialPortOpenState,
    serial: ManuallyDrop<SerialPort>,
}

impl core::future::Future for SerialPortOpenFuture {
    type Output = SerialPort;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let SerialPortOpenState::Configure { baud_rate } = this.state {
            unsafe {
                vexDeviceGenericSerialEnable(this.serial.device, 0);
                vexDeviceGenericSerialBaudrate(this.serial.device, baud_rate as i32);
            }

            this.state = SerialPortOpenState::Waiting;
        }

        if this.serial.validate_port().is_ok() {
            // SAFETY: Device is not accessed from self.serial after `Poll::Ready` return.
            Poll::Ready(unsafe { ManuallyDrop::take(&mut this.serial) })
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
