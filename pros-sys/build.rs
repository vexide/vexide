use std::env;

fn main() {
    if let Some(docs_rs) = env::var_os("DOCS_RS") {
        if docs_rs == "1" {
            return;
        }
    }

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
        .extract(&out_dir)
        .unwrap();

    println!("cargo:rustc-link-search=native={out_dir}/firmware");
}
