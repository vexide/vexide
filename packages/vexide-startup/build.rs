#![allow(missing_docs)]

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("cargo:rustc-link-search=native={manifest_dir}/link");

    #[cfg(feature = "vex-sdk-vexcode")]
    {
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        if target_os == "vexos" {
            vex_sdk_vexcode::link_sdk("V5_20240802_15_00_00");
        }
    }
}
