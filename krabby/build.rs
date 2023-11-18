// This is a build script - it runs before any Rust code is compiled
use std::{collections::HashSet, env};

fn main() {
    // Get list of target features
    // For RISC-V, the available features are:
    //  a - Atomics
    //  c - Compressed instructiosn
    //  d - Double-precision floating point
    //  f - Single-precision floating point
    //  m - Multiplication/Division
    //
    //  You can see more at <https://en.wikipedia.org/wiki/RISC-V#ISA_base_and_extensions>, but I
    //  think these are the only ones we can detect with `CARGO_CFG_TARGET_FEATURE`
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_else(|_| {
        // TODO: Figure out why target-features is empty in stable (1.74)
        println!("cargo:warning=Could not determine target features - maybe use Rust 1.75");
        String::from("a,c,m")
    });
    let target_features: HashSet<&str> = target_features.split(',').collect();

    // Compile C code
    let mut cc = cc::Build::new();
    let cc = cc
        .file("c/console.c")
        .file("c/demo.c")
        .file("c/entry.S")
        .file("c/functions.c")
        .file("c/panic.c")
        .file("c/rust_helpers.c")
        .file("c/stdio.c")
        .file("c/string.c")
        .flag("-ffreestanding");

    // We need to force gcc to use the correct ABI if the target has double-precision floating
    // points are used, otherwise we'll get linker issues.
    let cc = if target_features.contains("d") {
        let pointer_width = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
        cc.flag(&format!("-mabi=lp{pointer_width}d"))
    } else {
        cc
    };

    cc.compile("foo");

    // Set linker script
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg=./linker.ld");
}
