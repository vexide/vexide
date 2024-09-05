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

use snafu::Snafu;
use vex_sdk::{
    vexDeviceGenericSerialBaudrate, vexDeviceGenericSerialEnable, vexDeviceGenericSerialFlush,
    vexDeviceGenericSerialPeekChar, vexDeviceGenericSerialReadChar, vexDeviceGenericSerialReceive,
    vexDeviceGenericSerialReceiveAvail, vexDeviceGenericSerialTransmit,
    vexDeviceGenericSerialWriteChar, vexDeviceGenericSerialWriteFree, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

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
    /// The maximum allowed baud rate that generic serial can be configured to
    /// use by user programs.
    pub const MAX_BAUD_RATE: u32 = 921_600;

    /// The maximum length of the serial FIFO input and output buffer.
    pub const INTERNAL_BUFFER_SIZE: usize = 1024;

    /// Open and configure a serial port on a [`SmartPort`].
    ///
    /// This configures a [`SmartPort`] to act as a generic serial controller capable of sending/receiving
    /// data. Providing a baud rate, or the transmission rate of bits is required. The maximum theoretical
    /// baud rate is 921600.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200);
    /// }
    /// ```
    #[must_use]
    pub fn open(port: SmartPort, baud_rate: u32) -> Self {
        let device = unsafe { port.device_handle() };

        // These can't fail so we don't call validate_port.
        //
        // Unlike other devices, generic serial doesn't need a dedicated device plugged in,
        // we don't care about validating device types before configuration.
        unsafe {
            vexDeviceGenericSerialEnable(device, 0);
            vexDeviceGenericSerialBaudrate(device, baud_rate as i32);
        }

        Self { port, device }
    }

    /// Configures the baud rate of the serial port.
    ///
    /// Baud rate determines the speed of communication over the data channel. Under normal conditions, user code is limited
    /// to a maximum baudrate of 921600.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     // Change to 9600 baud
    ///     _ = serial.set_baud_rate(9600);
    /// }
    /// ```
    pub fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), SerialError> {
        self.validate_port()?;

        unsafe {
            vexDeviceGenericSerialBaudrate(self.device, baud_rate as i32);
        }

        Ok(())
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
    /// # Errors
    ///
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     _ = serial.clear_buffers();
    ///     _ = serial.write(b"Buffers are clear!");
    /// }
    /// ```
    pub fn clear_buffers(&mut self) -> Result<(), SerialError> {
        self.validate_port()?;

        unsafe {
            vexDeviceGenericSerialFlush(self.device);
        }

        Ok(())
    }

    /// Read the next byte available in the serial port's input buffer, or `None` if the input
    /// buffer is empty.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     loop {
    ///         if let Ok(Some(byte)) = serial.read_byte() {
    ///             println!("Got byte: {}", byte);
    ///         }
    ///
    ///         sleep(core::time::Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    pub fn read_byte(&mut self) -> Result<Option<u8>, SerialError> {
        self.validate_port()?;

        let byte = unsafe { vexDeviceGenericSerialReadChar(self.device) };

        Ok(match byte {
            -1 => None,
            _ => Some(byte as u8),
        })
    }

    /// Read the next byte available in the port's input buffer without removing it. Returns
    /// `None` if the input buffer is empty.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     if let Ok(Some(next_byte)) = serial.peek_byte() {
    ///         println!("Next byte: {}", next_byte);
    ///     }
    /// }
    /// ```
    pub fn peek_byte(&self) -> Result<Option<u8>, SerialError> {
        self.validate_port()?;

        Ok(
            match unsafe { vexDeviceGenericSerialPeekChar(self.device) } {
                -1 => None,
                byte => Some(byte as u8),
            },
        )
    }

    /// Write a single byte to the port's output buffer.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::WriteFailed`] error is returned if the byte could not be written.
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     // Attempt to write 0x80 (128u8) to the output buffer
    ///     _ = serial.write_byte(0x80);
    /// }
    /// ```
    pub fn write_byte(&mut self, byte: u8) -> Result<(), SerialError> {
        self.validate_port()?;

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
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200);
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
        self.validate_port()?;

        match unsafe { vexDeviceGenericSerialReceiveAvail(self.device) } {
            // TODO: This check may not be necessary, since PROS doesn't do it,
            //		 but we do it just to be safe.
            -1 => ReadFailedSnafu.fail(),
            available => Ok(available as usize),
        }
    }

    /// Returns the number of bytes free in the port's FIFO output buffer.
    ///
    /// # Errors
    ///
    /// - A [`SerialError::ReadFailed`] error is returned if the serial device's status could not be read.
    /// - A [`SerialError::Port`] error is returned if a generic serial device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     // Write a byte if there's free space in the buffer.
    ///     if serial.available_write_bytes().is_ok_and(|available| available > 0) {
    ///         _ = serial.write_byte(0x80);
    ///     }
    /// }
    /// ```
    pub fn available_write_bytes(&self) -> Result<usize, SerialError> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericSerialWriteFree(self.device) } {
            // TODO: This check may not be necessary, since PROS doesn't do it,
            //		 but we do it just to be safe.
            -1 => ReadFailedSnafu.fail(),
            available => Ok(available as usize),
        }
    }
}

#[cfg(feature = "std")]
impl std::io::Read for SerialPort {
    /// Read some bytes from this serial port into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// # Errors
    ///
    /// - An error with the kind [`io::ErrorKind::AddrNotAvailable`] is returned if there is no device connected.
    /// - An error with the kind [`io::ErrorKind::AddrInUse`] is returned if the serial port is configured as another Smart device.
    /// - An error with the kind [`io::ErrorKind::Other`] is returned if an unexpected internal read error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     let mut buffer = vec![0; 2048];
    ///
    ///     loop {
    ///         _ = serial.read(&mut buffer);
    ///         sleep(core::time::Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.validate_port()?;

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
    /// Write a buffer into the serial port's output buffer, returning how many bytes
    /// were written.
    ///
    /// # Errors
    ///
    /// - An error with the kind [`io::ErrorKind::AddrNotAvailable`] is returned if there is no device connected.
    /// - An error with the kind [`io::ErrorKind::AddrInUse`] is returned if the serial port is configured as another Smart device.
    /// - An error with the kind [`io::ErrorKind::Other`] is returned if an unexpected internal write error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut serial = SerialPort::open(peripherals.port_1, 115200);
    ///
    ///     _ = serial.write(b"yo");
    /// }
    /// ```
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericSerialTransmit(self.device, buf.as_ptr(), buf.len() as i32) }
        {
            -1 => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal write error occurred.",
            )),
            written => Ok(written as usize),
        }
    }

    /// This function does nothing.
    ///
    /// Generic serial does not use traditional buffers, so data in the output
    /// buffer is immediately sent.
    ///
    /// If you wish to *clear* both the read and write buffers, you can use
    /// `Self::clear_buffers`.
    fn flush(&mut self) -> std::io::Result<()> {
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
#[derive(Debug, Snafu)]
pub enum SerialError {
    /// Internal write error occurred.
    WriteFailed,

    /// Internal read error occurred.
    ReadFailed,

    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
