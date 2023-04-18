use bindgen::{self, Builder};
use std::{io::BufRead, path::PathBuf, process::Command};

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

    zip::ZipArchive::new(std::fs::File::open(format!("{out_dir}/pros.zip")).unwrap())
        .unwrap()
        .extract(format!("{out_dir}"))
        .unwrap();

    let mut bindings = Builder::default()
        .header(format!("src/pros_entrypoint.h"))
        .use_core()
        .clang_arg(format!("-I{out_dir}/include"))
        .clang_args(&["-target", "arm-none-eabi"])
        .blocklist_item("FP_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    for dir in get_gcc_arm_include_dirs() {
        bindings = bindings.clang_arg(format!("-I{}", dir));
    }

    let bindings = bindings.generate().expect("could not generate bindings");

    bindings
        .write_to_file(PathBuf::from(out_dir.clone()).join("bindings.rs"))
        .expect("could not write bindings");

    println!("cargo:rustc-link-arg=-T{out_dir}/v5-common.ld");
    println!("cargo:rustc-link-search=native={out_dir}/firmware");
    println!("cargo:rustc-link-lib=static=pros");
    println!("cargo:rerun-if-changed=build.rs");
}

// Credit goes to the other bindings for pros at https://github.com/serxka/pros-rs for this method of getting the correct headers.
fn get_gcc_arm_include_dirs() -> Vec<String> {
    let mut is_include_dir = false;
    let include_dirs: Vec<String> = Command::new("arm-none-eabi-gcc")
        .args(["-E", "-xc", "-Wp,-v", "/dev/null"])
        .output()
        .expect("Could not run 'arm-none-eabi-gcc'. Is it installed?")
        .stderr
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| {
            let result = is_include_dir;
            if line == "#include <...> search starts here:" {
                is_include_dir = true;
            }
            result
        })
        .collect();

    include_dirs
}
