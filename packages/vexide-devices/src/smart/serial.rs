//! Generic serial device module.
//!
//! Provides support for using [`SmartPort`]s as generic serial communication devices.

use no_std_io::io;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceGenericSerialBaudrate, vexDeviceGenericSerialEnable, vexDeviceGenericSerialFlush,
    vexDeviceGenericSerialPeekChar, vexDeviceGenericSerialReadChar, vexDeviceGenericSerialReceive,
    vexDeviceGenericSerialReceiveAvail, vexDeviceGenericSerialTransmit,
    vexDeviceGenericSerialWriteChar, vexDeviceGenericSerialWriteFree, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// Represents a smart port configured as a generic serial controller.
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
    pub const MAX_BAUD_RATE: u32 = 921600;

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
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    /// ```
    pub fn open(port: SmartPort, baud_rate: u32) -> Self {
        let device = unsafe { port.device_handle() };

        // These can't fail so we don't call validate_port.
        //
        // Unlike other devices, generic serial doesn't need a dedicated device plugged in,
        // we we don't care about validating device types before configuration.
        unsafe {
            vexDeviceGenericSerialEnable(device, 0);
            vexDeviceGenericSerialBaudrate(device, baud_rate as i32);
        }

        Self { device, port }
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
    /// ```
    /// let mut serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// buffer.write(b"some bytes")?;
    /// buffer.flush()?;
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
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// loop {
    ///     if let Some(byte) = serial.read_byte()? {
    ///         println!("Got byte: {}", byte);
    ///     }
    ///     sleep(core::time::Duration::from_millis(10)).await;
    /// }
    /// ```
    pub fn read_byte(&self) -> Result<Option<u8>, SerialError> {
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
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// if let Some(next_byte) = serial.peek_byte()? {
    ///     println!("Next byte: {}", next_byte);
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
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// // Write 0x80 (128u8) to the output buffer
    /// serial.write_byte(0x80)?;
    /// ```
    pub fn write_byte(&mut self, byte: u8) -> Result<(), SerialError> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericSerialWriteChar(self.device, byte) } {
            -1 => Err(SerialError::WriteFailed),
            _ => Ok(()),
        }
    }

    /// Returns the number of bytes available to be read in the the port's FIFO input buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// if serial.bytes_to_read()? > 0 {
    ///     println!("{}", serial.read_byte()?.unwrap());
    /// }
    /// ```
    pub fn unread_bytes(&self) -> Result<usize, SerialError> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericSerialReceiveAvail(self.device) } {
            // TODO: This check may not be necessary, since PROS doesn't do it,
            //		 but we do it just to be safe.
            -1 => Err(SerialError::ReadFailed),
            available => Ok(available as usize),
        }
    }

    /// Returns the number of bytes free in the port's FIFO output buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// if serial.available_write_bytes()? > 0 {
    ///     serial.write_byte(0x80)?;
    /// }
    /// ```
    pub fn available_write_bytes(&self) -> Result<usize, SerialError> {
        self.validate_port()?;

        match unsafe { vexDeviceGenericSerialWriteFree(self.device) } {
            // TODO: This check may not be necessary, since PROS doesn't do it,
            //		 but we do it just to be safe.
            -1 => Err(SerialError::ReadFailed),
            available => Ok(available as usize),
        }
    }
}

impl io::Read for SerialPort {
    /// Read some bytes from this serial port into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// let mut buffer = Vec::new();
    ///
    /// loop {
    ///     serial.read(&mut buffer);
    ///     sleep(core::time::Duration::from_millis(10)).await;
    /// }
    /// ```
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.validate_port().map_err(|e| match e {
            PortError::Disconnected => {
                io::Error::new(io::ErrorKind::AddrNotAvailable, "Port does not exist.")
            }
            PortError::IncorrectDevice => io::Error::new(
                io::ErrorKind::AddrInUse,
                "Port is in use as another device.",
            ),
        })?;

        match unsafe {
            vexDeviceGenericSerialReceive(self.device, buf.as_mut_ptr(), buf.len() as i32)
        } {
            -1 => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal read error occurred.",
            )),
            received => Ok(received as usize),
        }
    }
}

impl io::Write for SerialPort {
    /// Write a buffer into the serial port's output buffer, returning how many bytes
    /// were written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.validate_port().map_err(|e| match e {
            PortError::Disconnected => {
                io::Error::new(io::ErrorKind::AddrNotAvailable, "Port does not exist.")
            }
            PortError::IncorrectDevice => io::Error::new(
                io::ErrorKind::AddrInUse,
                "Port is in use as another device.",
            ),
        })?;

        match unsafe { vexDeviceGenericSerialTransmit(self.device, buf.as_ptr(), buf.len() as i32) }
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
    /// Generic serial does not use traditional buffers, so data in the output
    /// buffer is immediately sent.
    ///
    /// If you wish to *clear* both the read and write buffers, you can use
    /// `Self::clear_buffers`.
    fn flush(&mut self) -> io::Result<()> {
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

/// Errors that can occur when interacting with a [`SerialPort`].
#[derive(Debug, Snafu)]
pub enum SerialError {
    /// Internal write error occurred.
    WriteFailed,

    /// Internal read error occurred.
    ReadFailed,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
