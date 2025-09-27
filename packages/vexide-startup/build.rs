#![allow(missing_docs)]

use std::{fs::File, io::Write, path::PathBuf};

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

    // This logic is for encoding the QR code used on the data abort screen.
    // See: ./src/abort_handler/report.rs

    let data = "https://vexide.dev/docs/aborts";

    let code = QrCode::with_error_correction_level(data, EcLevel::L).unwrap();
    let width = code.width();

    let bits: Vec<u8> = code
        .to_colors()
        .into_iter()
        .map(|cell| cell.select(1, 0))
        .collect();

    let packed: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| {
            let mut byte = 0;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << i;
            }

            byte
        })
        .collect();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // The format here is a BE u16 indicating width and then a bunch of packed bits.
    let mut file = File::create(out_dir.join("abort_qrcode.bin")).unwrap();
    file.write_all(&(width as u16).to_be_bytes()).unwrap();
    file.write_all(&packed).unwrap();
}
