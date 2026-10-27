use core::hint::unreachable_unchecked;

use crc::{CRC_32_ISO_HDLC, Crc};

use crate::{
    __gnu_build_id_start,
    abort_handler::Fault,
};

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
    fn new(data: &CrashDumpData2) -> Self {
        CrashDumpHeader {
            magic: CRASH_DUMP_MAGIC,
            // See <https://interrupt.memfault.com/blog/gnu-build-id-for-firmware> for offset explanation
            build_id: unsafe { *(&raw mut __gnu_build_id_start).add(16).cast::<[u8; 20]>() },
            crc: data.crc(),
        }
    }
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct CrashDumpData2 {
    /// The fault data, if there is one (see `data_type` field).
    pub fault: Fault,
    /// Data specific to a fault type, if there is one (see `data_type` field).
    ///
    /// See [`crate::abort_handler::FaultException`] for decoding.
    pub fault_exception_data: u128,
    /// The panic message, if there is one (see `data_type` field).
    pub panic_msg: [u8; 128],
    /// The length of the panic message, if there is one (see `data_type` field).
    pub panic_length: u8,
    /// Indicates whether crash dump is due to a panic (`data_type == 0`) or an abort (`data_type
    /// == 1`)
    ///
    /// If the crash dump is due to a panic then the `fault` and `fault_exception_data` fields do
    /// not contain any meaningful data. If the crash dump is due to an abort then the
    /// `panic_msg` and `panic_length` fields do not contain any meaningful data.
    pub data_type: u8,
    /// This field is just to keep `bytemuck::Pod` happy.
    pub _pad: [u8; 10],
    /// The backtrace at the time of the panic/abort.
    pub backtrace: [u32; 27],
    /// The length of `backtrace`.
    pub backtrace_length: u8,
    /// This field is just to keep `bytemuck::Pod` happy.
    pub _pad2: [u8; 7],
}

impl Default for CrashDumpData2 {
    fn default() -> Self {
        Self {
            ..bytemuck::Zeroable::zeroed()
        }
    }
}

impl CrashDumpData2 {
    fn crc(&self) -> u32 {
        let bytes = bytemuck::try_cast_slice(core::slice::from_ref(self));
        match bytes {
            Ok(bytes) => CRC.checksum(bytes),
            Err(
                bytemuck::PodCastError::AlignmentMismatch
                | bytemuck::PodCastError::OutputSliceWouldHaveSlop
                | bytemuck::PodCastError::SizeMismatch
                | bytemuck::PodCastError::TargetAlignmentGreaterAndInputNotAligned,
            ) => unsafe { unreachable_unchecked() },
        }
    }
}

#[repr(C)]
pub struct CrashDump {
    /// The header of the crash dump
    pub header: CrashDumpHeader,
    /// The data of the crash dump
    pub data: CrashDumpData2,
}

impl CrashDump {
    /// Construct a [`CrashDump`] from the given crash dump data.
    pub fn new(data: CrashDumpData2) -> Self {
        Self {
            header: CrashDumpHeader::new(&data),
            data,
        }
    }
    pub fn check_crc(&self) -> bool {
        self.header.crc == self.data.crc()
    }
}
