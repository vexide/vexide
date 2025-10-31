//! Store and read persistent crash dumps across executions of different programs.

use std::fmt::Write;

use bytemuck::{Pod, Zeroable};
use crc::{Crc, NoTable};

const MAGIC: u32 = u32::from_be_bytes(*b"VXCD");
const VERSION: u32 = 1;
const CRC: Crc<u32, NoTable> = Crc::<u32, NoTable>::new(&crc::CRC_32_ISCSI);

/// Create a pointer to a crash dump where the auxiliary file is.
///
/// In the context of `vexide-startup`, this region of memory is usually used to store patches for
/// differential uploading. This function will return `None` if the auxiliary file's memory region
/// is not valid for storing a crash dump.
#[cfg(target_os = "vexos")]
#[must_use]
pub(crate) fn annex_auxiliary_file() -> Option<*mut CrashDump> {
    let linked_file_start = &raw mut crate::__linked_file_start;
    let linked_file_end = &raw mut crate::__linked_file_end;

    let size = unsafe { linked_file_end.byte_offset_from_unsigned(linked_file_start) };
    if size < size_of::<CrashDump>() {
        return None;
    }

    let dump: *mut CrashDump = linked_file_start.cast();
    if !dump.is_aligned() {
        return None;
    }

    Some(dump)
}

/// Reads a crash dump from the auxiliary file region, treating it as persistent memory.
///
/// # Safety
///
/// The program must not hold access to other data in the auxiliary file region now or in the
/// future.
#[cfg(target_os = "vexos")]
#[must_use]
pub unsafe fn read_persistent_crash_dump() -> Option<CrashDump> {
    use std::ptr;

    // A volatile read is used here to prevent the compiler from trying to predict
    // what is stored in the crash dump. The auxiliary file memory is treated as I/O
    // provided by the operating system of whatever the previous program left there.

    // This serves the goal of recovering whatever data was stored here during previous program
    // executions. CrashDump has no type invariants, so it's okay if it's scrambled.
    let dump = unsafe { ptr::read_volatile(annex_auxiliary_file()?) };

    println!("Raw dump: {dump:?}");

    let is_valid = dump.verify_seal();
    if !is_valid {
        return None;
    }

    Some(dump)
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct CrashDump {
    pub magic: u32,
    pub version: u32,
    pub checksum: u32,
    pub payload: CrashPayload,
}

impl CrashDump {
    pub fn seal(&mut self) {
        self.magic = MAGIC;
        self.version = VERSION;

        let payload_bytes = bytemuck::bytes_of(&self.payload);
        self.checksum = CRC.checksum(payload_bytes);
    }

    pub fn verify_seal(&self) -> bool {
        let payload_bytes = bytemuck::bytes_of(&self.payload);

        self.magic == MAGIC
            && self.version == VERSION
            && self.checksum == CRC.checksum(payload_bytes)
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct CrashPayload {
    pub message: StringBuf<256>,
    pub backtrace: [u32; 128],
}

impl Default for CrashPayload {
    fn default() -> Self {
        Self {
            message: StringBuf::default(),
            backtrace: [0; _],
        }
    }
}

#[derive(Debug, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct StringBuf<const N: usize> {
    pub length: u32,
    pub data: [u8; N],
}

impl<const N: usize> StringBuf<N> {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        // Adding "N" is used when encoding to mark the buffer as full
        // even if it seems like more bytes can be added.
        let mut length = self.length as usize;
        length = length.checked_sub(N).unwrap_or(length);

        &self.data[..length]
    }
}

impl<const N: usize> Write for StringBuf<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self.length as usize >= N {
            return Ok(());
        }

        for char in s.chars() {
            let mut buf = [0; 4];
            let encoded = char.encode_utf8(&mut buf).as_bytes();

            // Avoid writing partial characters by checking if this will cause
            // an overflow.
            let new_length = self.length as usize + encoded.len();
            if new_length >= N {
                self.length += N as u32; // seal to prevent further writes
                break;
            }

            self.data[self.length as usize..new_length].copy_from_slice(encoded);
            self.length += 1;
        }

        Ok(())
    }
}

impl<const N: usize> Default for StringBuf<N> {
    fn default() -> Self {
        Self {
            length: 0,
            data: [0; _],
        }
    }
}

unsafe impl<const N: usize> Pod for StringBuf<N> {}
