use std::io::{self, Read};

/// VarIntProcessor encapsulates the logic for decoding a VarInt byte-by-byte.
#[derive(Default)]
pub struct VarIntProcessor {
    buf: [u8; 10],
    maxsize: usize,
    i: usize,
}

pub(crate) trait VarIntMaxSize {
    fn varint_max_size() -> usize;
}

impl<VI: VarInt> VarIntMaxSize for VI {
    fn varint_max_size() -> usize {
        (size_of::<VI>() * 8 + 7) / 7
    }
}

impl VarIntProcessor {
    fn new<VI: VarIntMaxSize>() -> VarIntProcessor {
        VarIntProcessor {
            maxsize: VI::varint_max_size(),
            ..VarIntProcessor::default()
        }
    }
    fn push(&mut self, b: u8) -> io::Result<()> {
        if self.i >= self.maxsize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unterminated varint",
            ));
        }
        self.buf[self.i] = b;
        self.i += 1;
        Ok(())
    }
    const fn finished(&self) -> bool {
        self.i > 0 && (self.buf[self.i - 1] & MSB == 0)
    }
    fn decode<VI: VarInt>(&self) -> Option<VI> {
        Some(VI::decode_var(&self.buf[0..self.i])?.0)
    }
}

pub const MSB: u8 = 0b1000_0000;
const DROP_MSB: u8 = 0b0111_1111;

/// A trait for reading VarInts from any other `Reader`.
///
/// It's recommended to use a buffered reader, as many small reads will happen.
pub trait VarIntReader {
    /// Returns either the decoded integer, or an error.
    ///
    /// In general, this always reads a whole varint. If the encoded varint's value is bigger
    /// than the valid value range of `VI`, then the value is truncated.
    ///
    /// On EOF, an io::Error with io::ErrorKind::UnexpectedEof is returned.
    fn read_varint<VI: VarInt>(&mut self) -> io::Result<VI>;
}

impl<R: Read> VarIntReader for R {
    fn read_varint<VI: VarInt>(&mut self) -> io::Result<VI> {
        let mut buf = [0_u8; 1];
        let mut p = VarIntProcessor::new::<VI>();

        while !p.finished() {
            let read = self.read(&mut buf)?;

            // EOF
            if read == 0 && p.i == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Reached EOF"));
            }
            if read == 0 {
                break;
            }

            p.push(buf[0])?;
        }

        p.decode()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Reached EOF"))
    }
}

/// Varint (variable length integer) encoding, as described in
/// https://developers.google.com/protocol-buffers/docs/encoding.
///
/// Uses zigzag encoding (also described there) for signed integer representation.
pub trait VarInt: Sized + Copy {
    /// Decode a value from the slice. Returns the value and the number of bytes read from the slice
    /// (can be used to read several consecutive values from a big slice) return None if all bytes
    /// has MSB set.
    fn decode_var(src: &[u8]) -> Option<(Self, usize)>;
}

// see: http://stackoverflow.com/a/2211086/56332
// casting required because operations like unary negation cannot be performed on unsigned integers
#[inline]
const fn zigzag_decode(from: u64) -> i64 {
    ((from >> 1) ^ (-((from & 1) as i64)) as u64) as i64
}

impl VarInt for u64 {
    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let mut result: u64 = 0;
        let mut shift = 0;

        let mut success = false;
        for b in src {
            let msb_dropped = b & DROP_MSB;
            result |= u64::from(msb_dropped) << shift;
            shift += 7;

            if b & MSB == 0 || shift > (9 * 7) {
                success = b & MSB == 0;
                break;
            }
        }

        if success {
            Some((result, shift / 7))
        } else {
            None
        }
    }
}

impl VarInt for i64 {
    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        if let Some((result, size)) = u64::decode_var(src) {
            Some((zigzag_decode(result) as Self, size))
        } else {
            None
        }
    }
}

macro_rules! impl_varint {
    ($t:ty, unsigned) => {
        impl VarInt for $t {
            fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
                let (n, s) = u64::decode_var(src)?;
                Some((n as Self, s))
            }
        }
    };
    ($t:ty, signed) => {
        impl VarInt for $t {
            fn required_space(self) -> usize {
                required_encoded_space_signed(self as i64)
            }

            fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
                let (n, s) = i64::decode_var(src)?;
                Some((n as Self, s))
            }

            fn encode_var(self, dst: &mut [u8]) -> usize {
                (self as i64).encode_var(dst)
            }
        }
    };
}

impl_varint!(usize, unsigned);
