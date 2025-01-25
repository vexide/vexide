use varint_encoding::VarIntReader;
use vexide_core::io::{Cursor, Read, Seek, SeekFrom};

mod varint_encoding;

// Assembly implementation of the patch overwriter (`__patcher_overwrite`).
//
// The overwriter is responsible for self-modifying the currently running code
// in memory with the new version built at 0x07E00000 by the first patcher stage.
//
// In other words, this code is responsible for actually "applying" the patch.
core::arch::global_asm!(include_str!("./overwriter_aeabi_memcpy.S"));
core::arch::global_asm!(include_str!("./overwriter.S"));

/// Internal patcher state representing what the patcher is attempting to do.
#[derive(Debug)]
enum PatcherState {
    Initial,
    Add(usize),
    Copy(usize),
}

/// Differential Upload Patcher
///
/// This function builds a modified version of the user program in memory with a binary patch (generated
/// by `bidiff` in cargo-v5) applied to it, then overwrites (self-modifies) the current program with the
/// newer version. This allows us to only upload a binary diff of what has changed from the original (the
/// "base") file, which is significanly smaller than reuploading the entire binary every time.
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
    // First four byets of the patch file MUST be 0xB1DF for the patch to be applied.
    const PATCH_MAGIC: u32 = 0xB1DF;

    // Next four bytes of the patch have to match this version identifier for the patch to be applied.
    const PATCH_VERSION: u32 = 0x1000;

    // Address of the 2mb region containing our reference ("base") binary that the patch will be
    // built and applied over.
    //
    // This is copied to this address by vexide's `_boot` routine that can be found in `lib.rs`.
    const BASE_START: u32 = 0x07C0_0000;
    // Address of where we will builder our patched binary at and store it before we overwrite the current
    // binary with it in stage 2.
    const NEW_START: u32 = 0x07E0_0000;

    /// Load address of patch files.
    const PATCH_MEMORY: *mut u32 = 0x07A0_0000 as _;

    unsafe {
        // First few bytes contain some important metadata we'll need to setup the patch.
        let patch_magic = PATCH_MEMORY.read(); // Should be 0xB1DF if the patch needs to be applied.
        let patch_version = PATCH_MEMORY.offset(1).read(); // Shoud be 0x1000
        let patch_len = PATCH_MEMORY.offset(2).read(); // length of the patch buffer
        let base_binary_len = PATCH_MEMORY.offset(3).read(); // length of the currently running binary
        let new_binary_len = PATCH_MEMORY.offset(4).read(); // length of the new binary after the patch

        // Do not proceed with the patch if:
        // - We have an unexpected PATCH_MAGIC (this is edited after the fact to 0xB2Df intentionally break out of here).
        // - Our patch format version does not match the version in the patch.
        // - There isn't anything to patch.
        if patch_magic != PATCH_MAGIC || patch_version != PATCH_VERSION || patch_len == 0 {
            // TODO(tropix126): We could reclaim the patch as heap space maybe? Not a high priority.
            return;
        }

        // Overwrite patch magic so we don't re-apply the patch next time.
        PATCH_MEMORY.write(0xB2DF);

        // Slice representing our patch contents.
        let mut patch = core::slice::from_raw_parts(
            PATCH_MEMORY.offset(5).cast(),
            patch_len as usize - (size_of::<u32>() * 5),
        );

        // Slice of the executable portion of the currently running program (this one currently running this code).
        let mut base = Cursor::new(core::slice::from_raw_parts(
            BASE_START as *const u8,
            base_binary_len as usize,
        ));

        // We'll build our new binary using our patch and base at address 0x07E00000 (NEW_START).
        let mut new: &mut [u8] =
            core::slice::from_raw_parts_mut(NEW_START as *mut u8, new_binary_len as usize);

        // Build the new binary, using `base` and `patch` as a reference.
        //
        // This is essentially a port of <https://github.com/divvun/bidiff/blob/main/crates/bipatch/src/lib.rs>

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
                    base.read_exact(out).unwrap();

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
                        base.seek(SeekFrom::Current(seek)).unwrap();

                        PatcherState::Initial
                    } else {
                        PatcherState::Copy(copy_len - n)
                    };

                    n
                }
            };

            new = &mut new[processed..];
        }

        // Jump to the stage 2 overwriter routine to handle the rest.
        core::arch::asm!("b __patcher_overwrite", options(noreturn));
    }
}
