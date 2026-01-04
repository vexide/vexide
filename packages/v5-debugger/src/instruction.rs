/// An instruction-set independent CPU instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// An instruction from the ARM32 instruction set.
    Arm(u32),
    /// An instruction from the Thumb instruction set.
    Thumb(u16),
}

impl Instruction {
    /// Returns whether this is a thumb instruction.
    #[must_use]
    pub const fn is_thumb(self) -> bool {
        matches!(self, Self::Thumb(_))
    }

    /// Returns the size of the instruction in bytes.
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::Arm(instr) => size_of_val(&instr),
            Self::Thumb(instr) => size_of_val(&instr),
        }
    }

    /// Returns the integer representation of the instruction casted to a usize.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        match self {
            Self::Arm(i) => i as usize,
            Self::Thumb(i) => i as usize,
        }
    }

    /// Reads either a thumb or ARM instruction from the given pointer.
    ///
    /// # Safety
    ///
    /// The address must be valid for reads.
    #[must_use]
    pub unsafe fn read(addr: *const u32, thumb: bool) -> Self {
        debug_assert!(!addr.is_null());
        if thumb {
            Self::Thumb(unsafe { core::ptr::read_volatile(addr.cast()) })
        } else {
            Self::Arm(unsafe { core::ptr::read_volatile(addr) })
        }
    }

    /// Writes this instruction to the given pointer.
    ///
    /// # Safety
    ///
    /// The address must be valid for writes. The caller must handle flushing the CPU instruction
    /// cache after calling this method.
    pub unsafe fn write_to(self, addr: *mut u32) {
        debug_assert!(!addr.is_null());
        match self {
            Self::Arm(instr) => unsafe {
                core::ptr::write_volatile(addr, instr);
            },
            Self::Thumb(instr) => unsafe {
                core::ptr::write_volatile(addr.cast(), instr);
            },
        }
    }
}
