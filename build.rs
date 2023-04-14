use std::{process::Command, path::PathBuf};
use bindgen::{self, Builder};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let pros_bytes = reqwest::blocking::get("https://github.com/purduesigbots/pros/releases/download/3.8.0/kernel@3.8.0.zip").unwrap().bytes().unwrap();
    let pros_bytes: Vec<_> = pros_bytes.into_iter().collect();
    // let mut pros_file = std::fs::File::create("{out_dir}/pros.zip").unwrap();
    // pros_file.write_all(&pros_bytes);
    std::fs::write("deps/pros.zip", pros_bytes).unwrap();

    Command::new("unzip")
        .args([
            &format!("deps/pros.zip"),
            "-d",
            &out_dir,
        ])
        .spawn()
        .expect("could not unzip pros library. is it installed?");

    let bindings = Builder::default()
            .header("{out_dir}/include/main.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("could not generate bindings");

        bindings
        .write_to_file(PathBuf::from(out_dir.clone()).join("bindings.rs"))
        .expect("could not write bindings");
    
    println!("cargo:rustc-link-arg=-T{out_dir}/v5.ld");
    println!("cargo:rustc-link-search=native={out_dir}/firmware");
    println!("cargo:rustc-link-lib=static=libpros.a");
    println!("cargo:rerun-if-changed=build.rs");
}
