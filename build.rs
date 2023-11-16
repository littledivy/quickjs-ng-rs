fn main() {
    let dst = cmake::build("libfoo");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=foo");
}