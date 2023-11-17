// Copyright 2023 Divy Srivastava. All rights reserved. MIT license.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();

    #[cfg(unix)]
    let dst = cmake::build("quickjs-ng").join("lib");
    #[cfg(windows)]
    let dst = {
        if cfg!(target_env = "msvc") {
            panic!("MSVC is not supported. Use x86_64-pc-windows-gnu");
        }

        // - MinGW Makefiles generator does not support -Thost args
        // - `cmake` Visual Studio by default even when compile is GCC
        let command = Command::new("cmake")
            .current_dir("quickjs-ng")
            .args(["-B", "build", "-DCMAKE_C_COMPILER=gcc"])
            .status()
            .unwrap();
        assert!(command.success());

        let command = Command::new("cmake")
            .current_dir("quickjs-ng")
            .args(["--build", "build"])
            .status()
            .unwrap();
        assert!(command.success());

        PathBuf::from(&root).join("quickjs-ng/build")
    };

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static:+verbatim=libqjs.a");

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
    link_static_fns(&header, &out_dir_path);

    bindings
        .write_to_file(out_dir_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn link_static_fns(header: &str, out_dir_path: &Path) {
    let obj_path = out_dir_path.join("extern.o");

    let clang_output = Command::new("gcc")
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

    // TODO(@littledivy): MSVC support
    // #[cfg(target_os = "windows")]
    // let lib_output = Command::new("lib").arg(&obj_path).output().unwrap();
    // #[cfg(not(target_os = "windows"))]
    {
        let lib_output = Command::new("ar")
            .arg("rcs")
            .arg(out_dir_path.join("libextern.a"))
            .arg(obj_path)
            .output()
            .unwrap();
        if !lib_output.status.success() {
            panic!(
                "Could not emit library file:\n{}",
                String::from_utf8_lossy(&lib_output.stderr)
            );
        }
    }

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir_path.to_string_lossy()
    );
    println!("cargo:rustc-link-lib=static:+verbatim=libextern.a");
}
