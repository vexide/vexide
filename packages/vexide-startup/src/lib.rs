//! User program startup routines.
//!
//! This crate provides runtime infrastructure for booting VEX user programs from Rust code.
//!
//! - User code begins at an assembly routine called `_boot`, which sets up the stack
//!   section before jumping to a user-provided `_start` symbol, which should be your
//!   rust entrypoint.
//!
//! - From there, the Rust `_start` entrypoint may call the [`startup`] function to finish
//!   the startup process by clearing the `.bss` section (which stores uninitialized data)
//!   and initializing vexide's heap allocator.
//!
//! This crate does NOT provide a `libc` [crt0 implementation]. No `libc`-style global
//! constructors are called. This means that the [`__libc_init_array`] function must be
//! explicitly called if you wish to link to C libraries.
//!
//! [crt0 implementation]: https://en.wikipedia.org/wiki/Crt0
//! [`__libc_init_array`]: https://maskray.me/blog/2021-11-07-init-ctors-init-array
//!
//! # Example
//!
//! This is an example of a minimal user program that boots without using the main vexide
//! runtime or the `#[vexide::main]` macro.
//!
//! ```
//! #![no_main]
//! #![no_std]
//!
//! use vexide_startup::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};
//!
//! // SAFETY: This symbol is unique and is being used to start the runtime.
//! #[unsafe(no_mangle)]
//! unsafe extern "C" fn _start() -> ! {
//!     // Setup the heap, zero bss, apply patches, etc...
//!     unsafe {
//!         vexide_startup::startup();
//!     }
//!
//!     // Rust code goes here!
//!
//!     // Exit the program once we're done.
//!     vexide_core::program::exit();
//! }
//!
//! // Program header (placed at the first 20 bytes on the binary).
//! #[unsafe(link_section = ".code_signature")]
//! #[used] // This is needed to prevent the linker from removing this object in release builds
//! static CODE_SIG: CodeSignature = CodeSignature::new(
//!     ProgramType::User,
//!     ProgramOwner::Partner,
//!     ProgramFlags::empty(),
//! );
//!
//! // Panic handler (this would normally be provided by `veixde_panic`).
//! #[panic_handler]
//! const fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
//!     loop {}
//! }
//! ```

// Cannot use two SDK providers at once.
#[cfg(all(feature = "vex-sdk-build", feature = "vex-sdk-jumptable"))]
compile_error!("features `vex-sdk-jumptable` and `vex-sdk-build` are mutually exclusive");

pub mod allocator;
pub mod banner;
mod code_signature;
mod patcher;

#[cfg(feature = "panic_hook")]
mod panic_hook;

#[cfg(feature = "vex-sdk-jumptable")]
use vex_sdk_jumptable as _;

use core::arch::naked_asm;

pub use code_signature::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};
use patcher::PATCH_MAGIC;

/// Load address of user programs in memory.
const USER_MEMORY_START: u32 = 0x0380_0000;

// Linkerscript Symbols
//
// All of these external symbols are defined in our linkerscript (link/v5.ld) and don't have
// real types or values, but a pointer to them points to the address of their location defined
// in the linkerscript.
unsafe extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;

    static mut __linked_file_start: u8;
    static mut __linked_file_end: u8;

    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

/// vexide's first-stage boot routine.
///
/// This is the true entrypoint of vexide, containing the first instructions
/// of user code executed before anything else. This is written in assembly to
/// ensure that it stays the same across compilations (a requirement of the patcher),
///
/// This routine loads the stack pointer to the stack region specified in our
/// linkerscript, makes a copy of program memory for the patcher if needed, then
/// branches to the Rust entrypoint (_start) created by the #[vexide::main] macro.
#[unsafe(link_section = ".vexide_boot")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _vexide_boot() {
    naked_asm!(
        // Load the stack pointer to point to our stack section.
        //
        // This technically isn't required, as VEXos already sets up a stack for CPU1,
        // but that stack is relatively small and we have more than enough memory
        // available to us for this.
        //
        // SAFETY: Doing this should be safe, since VEXos doesn't seem to use its existing
        // stack after calling user code. This operation is safe assuming that the variables
        // on the previous stack are never read or written to during execution of the program.
        "ldr sp, =__stack_top",
        // Before any Rust code runs, we need to memcpy the currently running program in
        // memory to the `.patcher_base` section if a patch file needs to be applied.
        //
        // We do this since we need an unmodified copy of the base binary to run the
        // patcher correctly. Since Rust code will modify the binary by writing to `.data`
        // and `.bss`, we can't reference program memory in the patcher so we must make a
        // carbon copy of it before any Rust code gets the chance to modify these sections.

        // Check if a patch file is loaded into memory by reading the first four bytes
        // at the expected location (0x07A00000) and checking if they equal the magic
        // header value of 0xB1DF.
        "ldr r0, =__patcher_patch_start",
        "ldr r0, [r0]",
        "ldr r1, ={patch_magic}",
        "cmp r0, r1", // r0 == 0xB1DF?
        // Prepare to memcpy binary to 0x07C00000
        "ldr r0, =__patcher_base_start",     // memcpy dest -> r0
        "ldr r1, =__user_ram_start",         // memcpy src -> r1
        "ldr r2, =__patcher_patch_start+12", // Base binary len is stored as metadata in the patch
        "ldr r2, [r2]",                      // memcpy size -> r2
        // Do the memcpy if patch magic is present (we checked this in our `cmp` instruction).
        "bleq __overwriter_aeabi_memcpy",
        // Jump to the Rust entrypoint.
        "b _start",
        patch_magic = const PATCH_MAGIC,
    )
}

/// Rust runtime initialization.
///
/// This function performs some prerequestites to allow Rust code to properly run. It must
/// be called once before any static data access or heap allocation is done. When using
/// `vexide`, this function is already called for you by the `#[vexide::main]` macro, so
/// there's no need to call it yourself.
///
/// This function does the following initialization:
///
/// - Fills the `.bss` (uninitialized statics) section with zeroes.
/// - Sets up the global heap allocator if the `allocator` feature is specified.
/// - Applies [differential upload patches] to the program if a patch file exists in memory.
///
/// [differential upload patches]: https://vexide.dev/docs/building-uploading/#uploading-strategies
///
/// # Safety
///
/// Must be called *once and only once* at the start of program execution.
#[inline]
pub unsafe fn startup() {
    #[cfg(target_vendor = "vex")]
    unsafe {
        // Initialize the heap allocator in our heap region defined in the linkerscript
        #[cfg(feature = "allocator")]
        crate::allocator::claim(&raw mut __heap_start, &raw mut __heap_end);

        // If this link address is 0x03800000, this implies we were uploaded using
        // differential uploads by cargo-v5 and may have a patch to apply.
        if vex_sdk::vexSystemLinkAddrGet() == USER_MEMORY_START {
            patcher::patch();
        }

        // Reclaim 6mb memory region occupied by patches and program copies as heap space.
        #[cfg(feature = "allocator")]
        crate::allocator::claim(&raw mut __linked_file_start, &raw mut __linked_file_end);

        // Register custom panic hook if needed.
        #[cfg(feature = "panic_hook")]
        std::panic::set_hook(Box::new(panic_hook::hook));
    }
}
