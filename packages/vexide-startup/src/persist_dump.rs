use crc::{CRC_32_ISO_HDLC, Crc, Digest};

use crate::{
    __gnu_build_id_start, abort_handler::{Fault, FaultExceptionData}};

static CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

const CRASH_DUMP_MAGIC: u32 = 0xDEADC0DE;

#[repr(C)]
pub struct CrashDumpHeader {
    /// This field must be CRASH_DUMP_MAGIC
    magic: u32,
    /// SHA-1 build id, from .note.gnu.build-id
    build_id: [u8; 20],
    /// CRC-32 of data and backtrace fields
    pub crc: u32,
}

impl CrashDumpHeader {
    pub fn new() -> Self {
        CrashDumpHeader {
            magic: CRASH_DUMP_MAGIC,
            // See <https://interrupt.memfault.com/blog/gnu-build-id-for-firmware> for offset explanation
            build_id: unsafe { *(&raw mut __gnu_build_id_start).add(16).cast::<[u8; 20]>() },
            crc: 0,
        }
    }
}

#[repr(C)]
pub enum CrashDumpData {
    Fault {
        fault: Fault,
        fault_data: FaultExceptionData,
    },
    Panic {
        message: tinyvec::ArrayVec<[u8; 256]>,
    }
}

#[repr(C)]
pub struct CrashDump {
    pub header: CrashDumpHeader,
    pub data: CrashDumpData,
    pub backtrace: tinyvec::ArrayVec<[u32; 128]>,
}

impl CrashDump {
    pub fn new(data: CrashDumpData) -> Self {
        Self {
            header: CrashDumpHeader::new(),
            data: data,
            backtrace: Default::default()
        }
    }
}
