//! This crate provides a minimal startup routine for user code on the VEX V5 Brain.
//!
//! - User code begins at an assembly routine called `_boot`, which sets up the stack
//!   section before jumping to a user-provided `_start` symbol, which should be your
//!   rust entrypoint.
//!
//! - From there, the Rust entrypoint may call the [`startup`] function to finish the
//!   startup process by clearing the `.bss` section (intended for uninitialized data)
//!   and initializing vexide's heap allocator.
//!
//! This crate is NOT a crt0 implementation. No global constructors are called.

#![no_std]
#![allow(clippy::needless_doctest_main)]

extern crate alloc;

use banner::themes::BannerTheme;
use bitflags::bitflags;
use varint_slop::VarIntReader;
use vexide_core::io::{Cursor, Read, Seek, SeekFrom};

pub mod banner;
pub(crate) mod varint_slop;

/// Identifies the type of binary to VEXos.
#[repr(u32)]
#[non_exhaustive]
pub enum ProgramType {
    /// User program binary.
    User = 0,
}

/// The owner (originator) of the user program
#[repr(u32)]
pub enum ProgramOwner {
    /// Program is a system binary.
    System = 0,

    /// Program originated from VEX.
    Vex = 1,

    /// Program originated from a partner developer.
    Partner = 2,
}

bitflags! {
    /// Program Flags
    ///
    /// These bitflags are part of the [`CodeSignature`] that determine some small
    /// aspects of program behavior when running under VEXos. This struct contains
    /// the flags with publicly documented behavior.
    #[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ProgramFlags: u32 {
        /// Inverts the background color to pure white.
        const INVERT_DEFAULT_GRAPHICS = 1 << 0;

        /// VEXos scheduler simple tasks will be killed when the program requests exit.
        const KILL_TASKS_ON_EXIT = 1 << 1;

        /// If VEXos is using the Light theme, inverts the background color to pure white.
        const THEMED_DEFAULT_GRAPHICS = 1 << 2;
    }
}

/// Program Code Signature
///
/// The first 16 bytes of a VEX user code binary contain a user code signature,
/// containing some basic metadata and startup flags about the program. This
/// signature must be at the start of the binary for booting to occur.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CodeSignature(vex_sdk::vcodesig, [u32; 4]);

impl CodeSignature {
    /// Creates a new signature given a program type, owner, and flags.
    #[must_use]
    pub const fn new(program_type: ProgramType, owner: ProgramOwner, flags: ProgramFlags) -> Self {
        Self(
            vex_sdk::vcodesig {
                magic: vex_sdk::V5_SIG_MAGIC,
                r#type: program_type as _,
                owner: owner as _,
                options: flags.bits(),
            },
            [0; 4],
        )
    }
}

unsafe extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;

    // These symbols don't have real types, so this is a little bit of a hack.
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

// This is the true entrypoint of vexide, containing the first two
// instructions of user code executed before anything else.
//
// This function loads the stack pointer to the stack region specified
// in our linkerscript, then immediately branches to the Rust entrypoint
// created by our macro.
core::arch::global_asm!(
    r#"
.section .boot, "ax"
.global _boot

_boot:
    ldr sp, =__stack_top @ Load the user stack.
    b _start             @ Jump to the Rust entrypoint.
"#
);

// Assembly implementation of the patch overwriter (`__patcher_overwrite`).
//
// The overwriter is responsible for self-modifying the currently running code
// in memory with the new version on the heap built by the first patcher stage.
//
// In other words, this code is responsible for actually "applying" the patch.
core::arch::global_asm!(include_str!("./overwriter_aeabi_memcpy.S"));
core::arch::global_asm!(include_str!("./overwriter.S"));

/// Zeroes the `.bss` section
///
/// # Arguments
///
/// - `sbss`. Pointer to the start of the `.bss` section.
/// - `ebss`. Pointer to the open/non-inclusive end of the `.bss` section.
///   (The value behind this pointer will not be modified)
/// - Use `T` to indicate the alignment of the `.bss` section.
///
/// # Safety
///
/// - Must be called exactly once
/// - `mem::size_of::<T>()` must be non-zero
/// - `ebss >= sbss`
/// - `sbss` and `ebss` must be `T` aligned.
#[inline]
#[allow(clippy::similar_names)]
#[cfg(target_vendor = "vex")]
unsafe fn zero_bss<T>(mut sbss: *mut T, ebss: *mut T)
where
    T: Copy,
{
    while sbss < ebss {
        // NOTE: volatile to prevent this from being transformed into `memclr`
        unsafe {
            core::ptr::write_volatile(sbss, core::mem::zeroed());
            sbss = sbss.offset(1);
        }
    }
}

#[derive(Debug)]
enum PatcherState {
    Initial,
    Add(usize),
    Copy(usize),
}

/// Startup Routine
///
/// - Sets up the heap allocator if necessary.
/// - Zeroes the `.bss` section if necessary.
/// - Prints the startup banner with a specified theme, if enabled.
///
/// # Safety
///
/// Must be called once at the start of program execution after the stack has been setup.
#[inline]
#[allow(clippy::too_many_lines)]
pub unsafe fn startup<const BANNER: bool>(theme: BannerTheme) {
    #[cfg(target_vendor = "vex")]
    unsafe {
        // Fill the `.bss` section of our program's memory with zeroes to ensure that uninitialized data is allocated properly.
        zero_bss(
            core::ptr::addr_of_mut!(__bss_start),
            core::ptr::addr_of_mut!(__bss_end),
        );
    }

    'patcher: {
        const PATCH_MAGIC: u32 = 0xB1DF;
        const PATCH_VERSION: u32 = 0x1000;
        const USER_MEMORY_START: u32 = 0x0380_0000;
        const PATCH_MEMORY_START: u32 = 0x0780_0000;

        let link_addr = unsafe { vex_sdk::vexSystemLinkAddrGet() };

        // This means we might potentially have a patch that needs to be applied.
        if link_addr == USER_MEMORY_START {
            // Pointer to the linked file in memory.
            let patch_ptr = PATCH_MEMORY_START as *mut u32;

            unsafe {
                // First few bytes contain some important metadata we'll need to setup the patch.
                let patch_magic = patch_ptr.read(); // Should be 0xB1DF if the patch needs to be applied.
                let patch_version = patch_ptr.offset(1).read(); // Shoud be 0x1000
                let patch_len = patch_ptr.offset(2).read(); // length of the patch buffer
                let old_binary_len = patch_ptr.offset(3).read(); // length of the currently running binary
                let new_binary_len = patch_ptr.offset(4).read(); // length of the new binary after the patch
                let new_heap_start = patch_ptr.offset(5).read(); // address of the __heap_start address in the new binary

                // Do not proceed with the patch if:
                // - We have an unexpected PATCH_MAGIC (this is edited after the fact to 0xB2Df intentionally break out of here).
                // - Our patch format version does not match the version in the patch.
                // - There isn't anything to patch.
                if patch_magic != PATCH_MAGIC || patch_version != PATCH_VERSION || patch_len == 0 {
                    // TODO(tropix126): We could reclaim the patch as heap space maybe? Not a high priority.
                    break 'patcher;
                }

                // Overwrite patch magic so we don't re-apply the patch next time.
                patch_ptr.write(0xB2DF);

                // Create a temporary heap at a location that will not overlap the binary after being patched.
                vexide_core::allocator::claim(
                    // .max(__heap_start) handles the case where the new binary is smaller than the old.
                    (new_heap_start).max(&raw const __heap_start as u32) as *mut u8,
                    &raw mut __heap_end,
                );

                // Slice representing our patch contents.
                let mut patch = core::slice::from_raw_parts(
                    patch_ptr.offset(6).cast(),
                    patch_len as usize - (size_of::<u32>() * 6),
                );

                // Slice of the executable portion of the currently running program (this one currently running this code).
                let mut old = Cursor::new(core::slice::from_raw_parts_mut(
                    USER_MEMORY_START as *mut u8,
                    old_binary_len as usize,
                ));

                // `bidiff` does not patch in-place, meaning we need a copy of our currently running binary onto the heap
                // that we will apply our patch to using our actively running binary as a reference point for the "old" bits.
                // After that, `apply_patch` will handle safely overwriting user code with our "new" version on the heap.
                let mut new_vec = alloc::vec![0; new_binary_len as usize];
                let mut new: &mut [u8] = new_vec.as_mut_slice();

                // Apply the patch onto `new`, using `old` as a reference.
                //
                // This is basically a port of <https://github.com/divvun/bidiff/blob/main/crates/bipatch/src/lib.rs>

                let mut buf = [0u8; 4096];

                let mut state = PatcherState::Initial;

                while !new.is_empty() {
                    let processed = match state {
                        PatcherState::Initial => {
                            state = PatcherState::Add(patch.read_varint().unwrap());
                            0
                        }
                        PatcherState::Add(add_len) => {
                            let n = add_len.min(new.len()).min(buf.len());

                            let out = &mut new[..n];
                            old.read_exact(out).unwrap();

                            let dif = &mut buf[..n];
                            patch.read_exact(dif).unwrap();

                            for i in 0..n {
                                out[i] = out[i].wrapping_add(dif[i]);
                            }

                            state = if add_len == n {
                                let copy_len: usize = patch.read_varint().unwrap();
                                PatcherState::Copy(copy_len)
                            } else {
                                PatcherState::Add(add_len - n)
                            };

                            n
                        }
                        PatcherState::Copy(copy_len) => {
                            let n = copy_len.min(new.len());

                            let out = &mut new[..n];
                            patch.read_exact(out).unwrap();

                            state = if copy_len == n {
                                let seek: i64 = patch.read_varint().unwrap();
                                old.seek(SeekFrom::Current(seek)).unwrap();

                                PatcherState::Initial
                            } else {
                                PatcherState::Copy(copy_len - n)
                            };

                            n
                        }
                    };

                    new = &mut new[processed..];
                }

                // Jump to the overwriter to handle the rest.
                core::arch::asm!(
                    "mov r0, #0x03800000",
                    "b __patcher_overwrite",
                    in("r1") new_vec.as_ptr(),
                    in("r2") new_vec.len(),
                    options(noreturn)
                );
            }
        }
    }

    // Initialize the heap allocator using normal bounds
    unsafe {
        vexide_core::allocator::claim(&raw mut __heap_start, &raw mut __heap_end);
    }

    // Print the banner
    if BANNER {
        banner::print(theme);
    }
}
