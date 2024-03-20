ENTRY(start);

. = 0xF0000000;

SECTIONS {
    /* Include entry point at start of binary */
    .text : ALIGN(4K) {
        *(.text);
    }
    .data : ALIGN(4K) {
        *(.data*);
    }
    .rodata : ALIGN(4K) {
        /*
         * Solves issue of rodata* being put before .text
         * https://stackoverflow.com/questions/43727214/linker-seems-to-be-placing-data-in-wrong-section
         */
        *(.rodata*);
    }
    /* Nothing past this point needs to exist in the binary file */
    .bss (NOLOAD) : ALIGN(4K) {
        *(.bss*);
        PROVIDE(global_pointer = .);
    }
}
