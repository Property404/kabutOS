fn main() {
    cc::Build::new()
        .file("c/foo.c")
        .file("c/console.c")
        .file("c/demo.c")
        .file("c/dump.c")
        .file("c/foo.c")
        .file("c/functions.c")
        .file("c/kernel.c")
        .file("c/panic.c")
        .file("c/readline.c")
        .file("c/stdio.c")
        .file("c/string.c")
        .file("c/uart.c")
        .flag("-ffreestanding")
        .compile("foo");
}
