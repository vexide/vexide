//! User program startup and runtime support.
//!
//! This crate provides runtime infrastructure for booting vexide programs from Rust code.
//! This infrastructure includes a more optimized heap allocator, differential uploading
//! support, and a panic hook that draws panic messages to the screen and captures backtraces.
//!
//! - User code begins at an assembly routine called `_vexide_boot`, which sets up the stack
//!   section before jumping to the `_start` routine defined in libstd, which then calls the
//!   `main` function.
//!
//! - From there, consumers must call the [`startup`] function to finish the startup process
//!   by applying differential upload patches, claiming heap space, and setting up this crate's
//!   custom panic hook.
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
//! use vexide_core::program::{CodeSignature, ProgramOptions, ProgramOwner, ProgramType};
//!
//! // SAFETY: This symbol is unique and is being used to start the runtime.
//! fn main() {
//!     // Setup the heap, zero bss, apply patches, etc...
//!     unsafe {
//!         vexide_startup::startup();
//!     }
//!
//!     // Rust code goes here!
//! }
//!
//! // Program header (placed at the first 20 bytes on the binary).
//! #[unsafe(link_section = ".code_signature")]
//! #[used] // This is needed to prevent the linker from removing this object in release builds
//! static CODE_SIG: CodeSignature = CodeSignature::new(
//!     ProgramType::User,
//!     ProgramOwner::Partner,
//!     ProgramOptions::empty(),
//! );
//! ```

#[cfg(feature = "allocator")]
pub mod allocator;
pub mod banner;

#[cfg(all(target_os = "vexos", feature = "abort-handler"))]
mod abort_handler;
#[cfg(feature = "panic-hook")]
mod panic_hook;
#[cfg(target_os = "vexos")]
mod patcher;
mod sdk;

// Linkerscript Symbols
//
// All of these external symbols are defined by either Rust's armv7a-vex-v5 linkerscript, our ours
// (see link/vexide.ld). These symbols don't have real types or values, but a pointer to them points
// to the address of their location defined in the linkerscript.
#[cfg(target_os = "vexos")]
unsafe extern "C" {
    pub(crate) static mut __heap_start: u8;
    pub(crate) static mut __heap_end: u8;

    static mut __user_ram_start: u8;

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
/// branches to the Rust entrypoint (_start) defined in libstd.
#[unsafe(link_section = ".vexide_boot")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg(target_os = "vexos")]
unsafe extern "C" fn _vexide_boot() {
    core::arch::naked_asm!(
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
        "ldr r0, =__patcher_base_start", // memcpy dest -> r0
        "ldr r1, =__user_ram_start", // memcpy src -> r1
        "ldr r2, =__patcher_patch_start+12", // Base binary len is stored as metadata in the patch
        "ldr r2, [r2]", // memcpy size -> r2
        // Do the memcpy if patch magic is present (we checked this in our `cmp` instruction).
        "bleq __overwriter_aeabi_memcpy",
        // Jump to the Rust entrypoint.
        "b _start",
        patch_magic = const patcher::PATCH_MAGIC,
    )
}

/// vexide runtime initialization.
///
/// This function performs some prerequisites to allow vexide programs to properly run. It
/// must be called once at startup before any heap allocation is done. When using `vexide`,
/// this function is already called for you by the `#[vexide::main]` macro, so there's no
/// need to call it yourself (doing so would cause **undefined behavior**).
///
/// This function does the following initialization:
///
/// - Sets up the global heap allocator by [claiming](crate::allocator::claim) the default
///   heap region if the `allocator` feature is specified.
/// - Applies [differential upload patches] to the program if a patch file exists in memory
///   and restarts the program if necessary.
/// - Registers a custom [panic hook] to allow panic messages to be drawn to the screen and
///   backtrace to be collected. This can be enabled/disabled using the `panic-hook` and
///   `backtrace` features.
///
/// [differential upload patches]: https://vexide.dev/docs/building-uploading/#uploading-strategies
/// [panic hook]: https://doc.rust-lang.org/std/panic/fn.set_hook.html
///
/// # Examples
///
/// ```
/// // Not using the `#[vexide::main]` macro here.
/// fn main() {
///     unsafe {
///         vexide_startup::startup(); // Call this once at the start of main.
///     }
///
///     println!("Hi.");
/// }
/// ```
///
/// # Safety
///
/// Must be called *once and only once* at the start of program execution.
#[inline]
#[allow(clippy::needless_doctest_main)]
pub unsafe fn startup() {
    #[cfg(target_os = "vexos")]
    unsafe {
        // Initialize the heap allocator in our heap region defined in the linkerscript
        #[cfg(feature = "allocator")]
        crate::allocator::claim(&raw mut __heap_start, &raw mut __heap_end);

        // If this link address is 0x03800000, this implies we were uploaded using
        // differential uploads by cargo-v5 and may have a patch to apply.
        if vexide_core::program::linked_file().addr() == (&raw const __user_ram_start).addr() {
            patcher::patch();
        }

        // Reclaim 6mb memory region occupied by patches and program copies as heap space.
        #[cfg(feature = "allocator")]
        crate::allocator::claim(&raw mut __linked_file_start, &raw mut __linked_file_end);

        #[cfg(feature = "abort-handler")]
        abort_handler::install_vector_table();
    }

    // Register custom panic hook if needed.
    #[cfg(feature = "panic-hook")]
    std::panic::set_hook(Box::new(panic_hook::hook));
}

#[cfg(any(feature = "abort-handler", feature = "panic-hook"))]
#[allow(unused)]
pub(crate) mod colors {
    use vexide_devices::rgb::Rgb;

    pub const RED: Rgb<u8> = Rgb::new(0x8B, 0x00, 0x00);
    pub const WHITE: Rgb<u8> = Rgb::new(0xFF, 0xFF, 0xFF);
    pub const BLACK: Rgb<u8> = Rgb::new(0x00, 0x00, 0x00);
}
