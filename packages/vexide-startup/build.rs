#![allow(missing_docs)]

use std::env;

fn main() {
    println!(
        r#"cargo::rustc-check-cfg=cfg(vexide_upload_strategy, values("monolith", "differential"))"#
    );
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let upload_strategy =
        env::var("CARGO_CFG_VEXIDE_UPLOAD_STRATEGY").unwrap_or("differential".into());
    match upload_strategy.as_str() {
        "differential" => {} // Link script needs no overriding for differential uploads
        "monolith" => println!("cargo:rustc-link-arg=--defsym=__patcher_section_length=0"),
        value => panic!(
            r#"Unknown value "{value}" for vexide_upload_strategy! Valid values are "differential" and "monolith""#
        ),
    };
    println!("cargo:rustc-link-search=native={manifest_dir}/link");
}
