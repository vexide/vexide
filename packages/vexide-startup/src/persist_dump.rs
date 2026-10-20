use std::fmt::Write;

use crc::{CRC_32_ISO_HDLC, Crc, Digest};

static CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub(crate) struct CrashDump {
    len: usize,
    base: *mut u8,
    crc_digest: Digest<'static, u32>,
}

impl CrashDump {
    pub(crate) unsafe fn new(location: *mut u8) -> Self {
        let mut crash_dump = Self {
            len: 0,
            base: location,
            crc_digest: CRC.digest(),
        };
        unsafe {
            let base = crash_dump.base.cast::<u32>();
            // Magic value for identifying later
            *base.add(0) = 0xDEADC0DEu32;
            crash_dump.write_header_length(0);
            crash_dump.write_header_crc(0);
        }
        crash_dump
    }
    fn write_header_length(&mut self, len: usize) {
        unsafe {
            *self.base.cast::<u32>().add(1) = len as u32;
        }
    }
    fn write_header_crc(&mut self, crc: u32) {
        unsafe {
            *self.base.cast::<u32>().add(2) = crc;
        }
    }
}

impl Write for CrashDump {
    fn write_str(&mut self, string: &str) -> Result<(), std::fmt::Error> {
        let buffer = string.as_bytes();
        unsafe {
            buffer
                .as_ptr()
                .copy_to(self.base.add(self.len + 12), buffer.len());
        }
        self.crc_digest.update(buffer);
        self.len += buffer.len();
        self.write_header_length(self.len);
        self.write_header_crc(self.crc_digest.clone().finalize());
        Ok(())
    }
}
