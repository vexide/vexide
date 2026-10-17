#![allow(missing_docs)]

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // We need to adjust some linkerscript behavior based on whether or not the
    // `talc` allocator is enabled as a cargo feature so that users get the full
    // System allocator heap when it is disabled.
    //
    // Because dependency crates cannot transitively adjust linker args directly
    // (go yell at a brick wall in https://github.com/rust-lang/cargo/issues/9554),
    // we instead switch out the search path to use a DIFFERENT vexide.ld based on
    // if the `allocator` feature is enabled.
    //
    // NOTE: Upstream users of vexide pass -Tvexide.ld as link args from their
    //       project. vexide-startup only has control over which folder's
    //       vexide.ld is used.
    if std::env::var("CARGO_FEATURE_ALLOCATOR").is_ok() {
        println!("cargo:rustc-link-search=native={manifest_dir}/link+talc");
    } else {
        println!("cargo:rustc-link-search=native={manifest_dir}/link");
    }

    #[cfg(feature = "vex-sdk-vexcode")]
    {
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        if target_os == "vexos" {
            vex_sdk_vexcode::link_sdk("V5_20240802_15_00_00");
        }
    }
}
