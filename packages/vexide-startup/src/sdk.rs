//! Brings the appropriate SDK backend into scope to allow `vex-sdk` to
//! resolve the symbols.

// vex-sdk-mock takes priority over other SDKs to satisfy the resolver.
#[cfg(all(not(target_os = "vexos"), feature = "vex-sdk-mock"))]
use vex_sdk_mock as _;

// Ensure only one SDK implementation is used. If more than one is used,
// this will cause a linker error anyways, but we want to fail as early
// as possible here with an actually comprehensible error message.
//
// The exception is vex-sdk-mock, which may take priority over another
// SDK backend.
#[allow(dead_code)]
const _: () = {
    if (cfg!(feature = "vex-sdk-jumptable") as usize)
        + (cfg!(feature = "vex-sdk-download") as usize)
        + (cfg!(feature = "vex-sdk-pros") as usize)
        > 1
    {
        panic!("Only one `vex-sdk` backend may be used at a time.");
    }
};

// If an unsupported SDK is specified on a non-VEXos target and we aren't
// using vex-sdk-mock, throw a compiler error.
#[cfg(all(
    any(
        feature = "vex-sdk-jumptable",
        feature = "vex-sdk-download",
        feature = "vex-sdk-pros"
    ),
    not(target_os = "vexos"),
    not(feature = "vex-sdk-mock")
))]
compile_error!("The specified `vex-sdk` backend is unsupported on this target.");

// vex-sdk-jumptable and vex-sdk-pros may only be used on vexos targets or
// if vex-sdk-mock isn't specified already.
#[cfg(all(
    feature = "vex-sdk-jumptable",
    target_os = "vexos",
    not(feature = "vex-sdk-mock")
))]
use vex_sdk_jumptable as _;
#[cfg(all(
    feature = "vex-sdk-pros",
    target_os = "vexos",
    not(feature = "vex-sdk-mock")
))]
use vex_sdk_pros as _;
