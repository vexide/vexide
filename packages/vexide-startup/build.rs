#![allow(missing_docs)]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use qrcode::{EcLevel, QrCode};

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
