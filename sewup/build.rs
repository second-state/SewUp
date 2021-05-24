use std::path::Path;
extern crate cmake;
use cmake::Config;

#[allow(dead_code)]
fn build_link_ssvm_dylib() {
    let dst = Config::new("../ssvm-evmc").no_build_target(true).build();
    let evmcssvm_path = Path::new(&dst).join("build/tools/ssvm-evmc");
    println!("cargo:rustc-link-search=native={}", evmcssvm_path.display());
    println!("cargo:rustc-link-lib=dylib=ssvm-evmc");
}

#[cfg(test)]
fn main() {
    build_link_ssvm_dylib();
}

#[cfg(not(test))]
fn main() {}
