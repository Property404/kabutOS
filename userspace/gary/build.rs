// This is a build script - it runs before any Rust code is compiled

const LINKER_SCRIPT: &str = "./linker.ld";

fn main() {
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg={LINKER_SCRIPT}");
    println!("cargo:rerun-if-changed={LINKER_SCRIPT}");
}
