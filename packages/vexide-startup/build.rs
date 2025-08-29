#![allow(missing_docs)]

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("cargo:rustc-link-search=native={manifest_dir}/link");

    #[cfg(not(feature = "vex-sdk-build"))]
    vex_sdk_build::link_sdk("V5_20240802_15_00_00");
}
