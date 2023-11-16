// Copyright 2023 Divy Srivastava. All rights reserved. MIT license.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let dst = cmake::build("quickjs-ng");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=qjs");

    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let header = format!("{}/quickjs-ng/quickjs.h", root);
    let bindings = bindgen::Builder::default()
        .header(header.clone())
        .generate_inline_functions(true)
        // experimental
        .wrap_static_fns(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    link_static_fns(&header, out_dir_path);

    let out_path = PathBuf::from(root);
    bindings
        .write_to_file(out_path.join("src/bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn link_static_fns(header: &str, out_dir_path: PathBuf) {
    let obj_path = out_dir_path.join("extern.o");

    let clang_output = Command::new("clang")
        .arg("-O")
        .arg("-c")
        .arg("-o")
        .arg(&obj_path)
        .arg(env::temp_dir().join("bindgen").join("extern.c"))
        .arg("-include")
        .arg(header)
        .output()
        .unwrap();

    if !clang_output.status.success() {
        panic!(
            "Could not compile object file:\n{}",
            String::from_utf8_lossy(&clang_output.stderr)
        );
    }

    #[cfg(not(target_os = "windows"))]
    let lib_output = Command::new("ar")
        .arg("rcs")
        .arg(out_dir_path.join("libextern.a"))
        .arg(obj_path)
        .output()
        .unwrap();
    #[cfg(target_os = "windows")]
    let lib_output = Command::new("lib").arg(&obj_path).output().unwrap();

    if !lib_output.status.success() {
        panic!(
            "Could not emit library file:\n{}",
            String::from_utf8_lossy(&lib_output.stderr)
        );
    }

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir_path.to_string_lossy()
    );
    println!("cargo:rustc-link-lib=static=extern");
}
