fn main() {
    cc::Build::new()
        .file("c/console.c")
        .file("c/demo.c")
        .file("c/entry.S")
        .file("c/functions.c")
        .file("c/panic.c")
        .file("c/readline.c")
        .file("c/rust_helpers.c")
        .file("c/stdio.c")
        .file("c/string.c")
        .flag("-ffreestanding")
        .compile("foo");
    println!("cargo:rustc-link-arg=-g");
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg=./linker.ld");
    println!("cargo:rustc-link-arg=-nostdlib");
}
