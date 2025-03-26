//! User program startup routines.
//!
//! This crate provides runtime infrastructure for booting VEX user programs from Rust
//! (and optionally C) code.
//!
//! - User code begins at an assembly routine called `_boot`, which sets up the stack
//!   section before jumping to a user-provided `_start` symbol, which should be your
//!   rust entrypoint. This routine can be found in `boot.S`.
//!
//! - From there, the Rust `_start` entrypoint may call the [`startup`] function to finish
//!   the startup process by clearing the `.bss` section (which stores uninitialized data)
//!   and initializing vexide's heap allocator.
//!
//! If using the `libc` feature, this crate will also provide a [crt0 implementation] that
//! calls `libc`-style global constructors for compatibility with C static libraries.
//!
//! [crt0 implementation]: https://en.wikipedia.org/wiki/Crt0
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

#![no_std]

pub mod banner;
mod code_signature;
mod patcher;

pub use code_signature::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

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

    static mut __patcher_ram_start: u8;
    static mut __patcher_ram_end: u8;

    static mut __bss_start: u32;
    static mut __bss_end: u32;

    #[cfg(feature = "libc")]
    unsafe fn __libc_init_array();
}

// Include the first-stage assembly entrypoint. This routine contains the first
// instructions executed by the user processor when the program runs.
core::arch::global_asm!(include_str!("./boot.S"));

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
        // Clear the .bss (uninitialized statics) section by filling it with zeroes.
        // This is required, since the compiler assumes it will be zeroed on first access.
        core::ptr::write_bytes(
            &raw mut __bss_start,
            0,
            (&raw mut __bss_end).offset_from(&raw mut __bss_start) as usize,
        );

        // Initialize the heap allocator in our heap region defined in the linkerscript
        #[cfg(feature = "allocator")]
        vexide_core::allocator::claim(&raw mut __heap_start, &raw mut __heap_end);

        // If this link address is 0x03800000, this implies we were uploaded using
        // differential uploads by cargo-v5 and may have a patch to apply.
        if vex_sdk::vexSystemLinkAddrGet() == USER_MEMORY_START {
            patcher::patch();
        }

        // Reclaim 6mb memory region occupied by patches and program copies as heap space.
        #[cfg(feature = "allocator")]
        vexide_core::allocator::claim(&raw mut __patcher_ram_start, &raw mut __patcher_ram_end);

        #[cfg(feature = "libc")]
        __libc_init_array();
    }
}
