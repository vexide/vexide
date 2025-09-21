#![allow(missing_docs)]

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("cargo:rustc-link-search=native={manifest_dir}/link");

    #[cfg(all(target_os = "vexos", feature = "vex-sdk-download"))]
    vex_sdk_download::link_sdk("V5_20240802_15_00_00");
}
