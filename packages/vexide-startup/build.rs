#![allow(missing_docs)]

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=sysrt/vexide_sysrt.c");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_VEXIDE_TOOLCHAIN");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("cargo:rustc-link-search=native={manifest_dir}/link");

    #[cfg(feature = "vex-sdk-vexcode")]
    {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        if target_os == "vexos" {
            vex_sdk_vexcode::link_sdk("V5_20240802_15_00_00");
        }
    }

    let toolchain = env::var("CARGO_CFG_VEXIDE_TOOLCHAIN").ok();

    // Set by cargo-v5 if --toolchain=llvm-*
    if toolchain.as_deref() == Some("llvm") {
        cc::Build::new()
            .file("sysrt/vexide_sysrt.c")
            .flag("-std=gnu23")
            .compile("vexide_sysrt");
    }
}
