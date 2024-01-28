ENTRY(start);

. = 0x80000000;

SECTIONS {
    /* Include entry point at start of binary */
    .text : ALIGN(4K) {
        PROVIDE(kernel_start = .);
        *(.init);
        *(.text);
    }
    .rodata : ALIGN(4K) {
        /*
         * Solves issue of rodata* being put before .text
         * https://stackoverflow.com/questions/43727214/linker-seems-to-be-placing-data-in-wrong-section
         */
        *(.rodata*);
    }
    .data : ALIGN(4K) {
        *(.data);
    }
    /* Nothing past this point needs to exist in the binary file */
    .bss : ALIGN(4K) {
        PROVIDE(bss_start = .);
        *(.bss);
        *(.bss*);
        PROVIDE(global_pointer = .);
        PROVIDE(bss_end = .);
    }
    .heap : ALIGN(4K) {
        PROVIDE(heap_bottom = .);
        . += 32768;
        PROVIDE(heap_top = .);
    }
    /* Stack guard should be unmapped in virtual memory */
    .stack_guard : ALIGN(4K) {
        . += 4096;
    }
    .stack : ALIGN(4K) {
        PROVIDE(stack_bottom = .);
        . += 16384;
        PROVIDE(stack_top = .);
    }
}
