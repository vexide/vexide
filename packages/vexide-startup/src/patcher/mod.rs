use std::io::{Cursor, Read, Seek, SeekFrom};

use varint_decode::VarIntReader;

mod varint_decode;

/// First four bytes of a patch file.
pub const PATCH_MAGIC: u32 = 0xB1DF;

// Assembly implementation of the patch overwriter (`__patcher_overwrite`).
//
// The overwriter is responsible for self-modifying the currently running code
// in memory with the new version built at 0x07E00000 by the first patcher stage.
//
// In other words, this code is responsible for actually "applying" the patch.
core::arch::global_asm!(include_str!("./overwriter_aeabi_memcpy.S"));
core::arch::global_asm!(include_str!("./overwriter.S"));

// Linkerscript Symbols
//
// All of these external symbols are defined in our linkerscript (link/v5.ld) and don't have
// real types or values, but a pointer to them points to the address of their location defined
// in the linkerscript.
unsafe extern "C" {
    static mut __patcher_patch_start: u32;
    static mut __patcher_base_start: u32;
    static mut __patcher_new_start: u32;
}

/// Differential Upload Patcher
///
/// This function builds a modified version of the user program in memory with a binary patch (generated
/// by `bidiff` in cargo-v5) applied to it, then overwrites (self-modifies) the current program with the
/// newer version. This allows us to only upload a binary diff of what has changed from the original (the
/// "base") file, which is significantly smaller than reuploading the entire binary every time.
///
/// # Overview
///
/// Patching is performed in two steps. The first step is performed by this function, and involves
/// building the "newer" binary using the uploaded patch file and the current program as a reference.
/// The second step of patching is performed by the `__patcher_overwrite` assembly routine, which
/// is responsible for actually self-modifying the current program's active memory and preparing CPU1
/// to re-run the now patched program memory.
///
/// ## Stage 1 - Building the new binary
///
/// The first stage of patching involves building the new (patched) binary. When `cargo-v5` uploads a
/// new patch file to the brain, we tell it to load the file into memory at address `0x07A00000` — 6mb
/// from the end of the RWX memory block allocated to user code. This 6mb region of memory spanning
/// `0x07A00000`..`0x80000000` is reserved specifically for the patcher to use and is split into three
/// 2mb subregions (which will be discussed later).
///
/// In order to actually use the patch file to build a new binary, we need a reference to the original
/// "base binary" that it's patching over. The program running on the brain before the patch is applied
/// is always the original binary, so in the `_boot` routine (vexide's assembly entrypoint) we preemptively
/// make a copy of the currently running binary before any Rust code gets the chance to modify a writable
/// section of the binary like `.data` or `.bss` (which would corrupt the patch). This unmodified copy of
/// the old binary is copied to address `0x07C00000` (2mb after where our patch is loaded).
///
/// Finally, using the copy of the old binary and the patch, we are able to apply bidiff's [`bipatch`
/// algorithm](https://github.com/divvun/bidiff/blob/main/crates/bipatch/src/lib.rs) to build the new
/// binary file that we will run. This new binary is built at address `0x07E00000` (4mb after where our
/// patch is loaded).
///
/// ## Stage 2 - Overwriting the old binary with the new one
///
/// The second stage of the patch process is handled by the `__patcher_overwrite` assembly routine. This
/// routine is responsible for overwriting our currently running code with that new version we just built.
/// This involves memcpy-ing our new binary at `0x07E00000` to the start of user memory (`0x03800000`).
/// Doing this is far easier said than done though, and requires some important considerations as to not
/// shoot ourselves in the foot:
///
/// - When overwriting, it is *absolutely imperative* that the overwriter not depend on memory that is
///   in the process of being overwritten. This means that the overwriting routine must be done entirely
///   from a fixed address in memory and not reference any outside functions or data. We ensure this by
///   writing the entire stage-2 patch routine and memcpy implementation in assembly and placing these
///   instructions in a linker section (`.text.overwriter`) at a fixed address in memory.
///
/// - There is also a potential opportunity for soundness problems if the overwriter modifies itself while
///   running. This is more unlikely (due to the next point), but has bad implications if the patcher
///   self-modifies its own instructions while in the process running. Writing this part in assembly also
///   ensures this won't happen, because assembly will always compile down to the same instructions
///   regardless of how we tell our compiler to optimize our code. This routine will not change.
///
/// - Finally, there is the problem of cache coherency. ARMv7-A, like most modern architectures, caches
///   instructions fetched from memory into an on-chip L1 cache called the icache. There is also a similar
///   L1 cache called the dcache for memory/data accesses. ARM's cache model does not guarantee L1 cache will
///   always be synchronized (coherent) with what's actually present in physical memory when working with
///   self-modifying code, so we need to explicitly "clean" the instruction cache to bring it back to a
///   [point-of-unification (PoU)](https://developer.arm.com/documentation/den0013/d/Caches/Point-of-coherency-and-unification)
///   with physical memory, ensuring that when we jump back to the start of our program, we are actually
///   executing from the newly overwritten memory rather than stale instructions from icache. This process
///   of invalidating and cleaning instruction caches is described to further detail in
///   [ARM's documentation](https://developer.arm.com/documentation/den0013/latest/Caches/Invalidating-and-cleaning-cache-memory).
///
/// # Safety
///
/// The caller must ensure that the patch loaded at 0x07A00000 has been built using the currently running
/// binary as the basis for the patch.
pub(crate) unsafe fn patch() {
    // The first four bytes after the patch magic have to match this version identifier for the
    // patch to be applied.
    const PATCH_VERSION: u32 = 0x1000;

    /// Load address of patch files.
    const PATCH: *mut u32 = &raw mut __patcher_patch_start;

    unsafe {
        // First few bytes contain some important metadata we'll need to setup the patch.
        let patch_magic = PATCH.read(); // Should be 0xB1DF if the patch needs to be applied.
        let patch_version = PATCH.offset(1).read(); // Shoud be 0x1000
        let patch_len = PATCH.offset(2).read(); // length of the patch buffer
        let base_binary_len = PATCH.offset(3).read(); // length of the currently running binary
        let new_binary_len = PATCH.offset(4).read(); // length of the new binary after the patch

        // Do not proceed with  patch if:
        // - We have an unexpected PATCH_MAGIC (We later change this magic to 0xB2DF to intentionally
        //   trigger this check in order to break the patcher ouf of an infinite loop).
        // - Our patch format version does not match the version in the patch.
        // - There isn't anything to patch.
        if patch_magic != PATCH_MAGIC || patch_version != PATCH_VERSION || patch_len == 0 {
            return;
        }

        // Change patch magic to something invalid so we don't re-apply the patch next time.
        PATCH.write(0xB2DF);

        // Slice of the copy of user program memory we made in vexide's `_boot` routine before any
        // Rust code had the chance to modify `.bss` or `.data`. This is our base binary.
        let base = Cursor::new(core::slice::from_raw_parts(
            (&raw mut __patcher_base_start).cast(),
            base_binary_len as usize,
        ));

        // Slice of our patch contents. We offset by 20 bytes to skip the metadata inserted by cargo-v5 and bidiff.
        let patch = core::slice::from_raw_parts(
            PATCH.offset(5).cast(),
            patch_len as usize - (size_of::<u32>() * 5),
        );

        // This is a 2mb slice of uninitialized memory that we've reserved for building the new binary in.
        let new = core::slice::from_raw_parts_mut(
            (&raw mut __patcher_new_start).cast(),
            new_binary_len as usize,
        );

        // Build the new binary using `base` and `patch` as a reference.
        bipatch(base, patch, new);

        // Jump to the stage 2 overwriter routine to handle the rest.
        core::arch::asm!("b __patcher_overwrite", options(noreturn));
    }
}

/// `bipatch` Algorithm
///
/// This function writes into `new` given an older `base` reference buffer and a [bidiff]-compatible
/// differential patch buffer that is designed to be applied over the base to build a new binary.
///
/// [bidiff]: https://github.com/divvun/bidiff
///
/// This is essentially a port of <https://github.com/divvun/bidiff/blob/main/crates/bipatch/src/lib.rs>
// NOTE: LLVM should always inline this function since it's only called once.
fn bipatch<B: Read + Seek, P: Read>(mut old: B, mut patch: P, mut new: &mut [u8]) {
    /// Internal patcher state representing what the patcher is attempting to do.
    #[derive(Debug)]
    enum PatcherState {
        Initial,
        Add(usize),
        Copy(usize),
    }

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
}
