// This is a build script - it runs before any Rust code is compiled
fn main() {
    const LINKER_SCRIPT: &str = "./linker.ld";
    // Set linker script
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg={LINKER_SCRIPT}");
    println!("cargo:rerun-if-changed={LINKER_SCRIPT}");
}
