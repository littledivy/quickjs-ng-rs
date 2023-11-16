// Copyright 2023 Divy Srivastava. All rights reserved. MIT license.

use std::env;
use std::path::PathBuf;

fn main() {
    let dst = cmake::build("quickjs-ng");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=qjs");

    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let header = format!("{}/quickjs-ng/quickjs.h", root);
    let bindings = bindgen::Builder::default()
        .header(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(root);
    bindings
        .write_to_file(out_path.join("src/bindings.rs"))
        .expect("Couldn't write bindings!");
}
