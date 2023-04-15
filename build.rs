use bindgen::{self, Builder};
use std::{path::PathBuf, process::Command};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let pros_bytes = reqwest::blocking::get(
        "https://github.com/purduesigbots/pros/releases/download/3.8.0/kernel@3.8.0.zip",
    )
    .unwrap()
    .bytes()
    .unwrap();
    let pros_bytes: Vec<_> = pros_bytes.into_iter().collect();

    std::fs::write(format!("{out_dir}/pros.zip"), pros_bytes).unwrap();

    Command::new("unzip")
        .args([&format!("{out_dir}/pros.zip"), "-d", &out_dir])
        .spawn()
        .expect("could not unzip pros library. is unzip installed?");

    let bindings = Builder::default()
        .header(format!("{out_dir}/include/main.h"))
        .use_core()
        .clang_arg(format!("-I{out_dir}/include"))
        .blocklist_item("FP_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("could not generate bindings");

    bindings
        .write_to_file(PathBuf::from(out_dir.clone()).join("bindings.rs"))
        .expect("could not write bindings");

    println!("cargo:rustc-link-arg=-T{out_dir}/v5-common.ld");
    println!("cargo:rustc-link-search=native={out_dir}/firmware");
    println!("cargo:rustc-link-lib=static=pros");
    println!("cargo:rerun-if-changed=build.rs");
}
