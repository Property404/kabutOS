// This is a build script - it runs before any Rust code is compiled
fn main() {
    // Set linker script
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg=./linker.ld");
}
