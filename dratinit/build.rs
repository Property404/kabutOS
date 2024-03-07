// This is a build script - it runs before any Rust code is compiled
fn main() {
    println!("cargo:rustc-link-arg=-e");
    println!("cargo:rustc-link-arg=start");
}
