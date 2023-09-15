use bindgen::{self, Builder};
use std::{io::BufRead, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/pros_entrypoint.h");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    println!("cargo:rerun-if-env-changed=CPATH");

    // macOS only: include system sdk headers for errno.h values, etc.
    // if we had the (proprietary) vex v5 sdk this probably wouldn't be necessary
    #[cfg(target_os = "macos")]
    let macos_arg = {
        let Ok(sdk_command) = Command::new("xcrun")
            .args(["--sdk", "macosx", "--show-sdk-path"])
            .output()
        else {
            // prompt user to install xcode command line tools
            Command::new("xcode-select")
                .arg("--install")
                .status()
                .unwrap();
            panic!("macosx sdk not installed - please install Xcode Command Line Tools");
        };
        let sdk_path = std::str::from_utf8(&sdk_command.stdout).expect("could not parse sdk path");
        format!("-I{}/usr/include", sdk_path.trim_end())
    };

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

    println!("cargo:rustc-link-search=native={out_dir}/firmware");

    #[cfg_attr(feature = "xapi", allow(unused_mut))]
    let mut bindings = Builder::default()
        .header("src/pros_entrypoint.h")
        .use_core()
        .clang_arg(format!("-I{out_dir}/include"))
        .clang_args(&["-target", "arm-none-eabi"])
        .blocklist_item("FP_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    #[cfg(feature = "xapi")]
    let mut bindings = bindings.header(format!("{out_dir}/include/pros/apix.h"));

    #[cfg(target_os = "macos")]
    let mut bindings = bindings.clang_arg(macos_arg);

    for dir in get_gcc_arm_include_dirs() {
        bindings = bindings.clang_arg(format!("-I{}", dir));
    }

    let bindings = bindings.generate().expect("could not generate bindings");

    bindings
        .write_to_file(PathBuf::from(out_dir.clone()).join("bindings.rs"))
        .expect("could not write bindings");
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
