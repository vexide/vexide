use cfg_if::cfg_if;

fn main() {
    cfg_if! {
        if #[cfg(not(feature = "no-link"))] {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

            #[cfg(not(feature = "no-link"))]
            println!("cargo:rustc-link-search=native={manifest_dir}/link");
        }
    }
}
