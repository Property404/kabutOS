ENTRY(preinit);

. = 0x80000000;

SECTIONS {
    /* This is not mapped to virtual memory. It contains some initial code to
     * jump to .text, but after entering supervisor mode, this is just unmapped
     * to catch null dereferences */
    .init_zero : ALIGN(4K) {
        PROVIDE(virtual_base = .);
        *(.init_zero)
    }
    /* Include entry point at start of binary */
    .text : ALIGN(4K) {
        PROVIDE(kernel_start = .);
        *(.text);
    }
    .data : ALIGN(4K) {
        *(.data*);
    }
    .got : ALIGN(4K) {
        PROVIDE(got_start = .);
        *(.got*);
        PROVIDE(got_end = .);
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
        PROVIDE(bss_start = .);
        *(.bss*);
        PROVIDE(global_pointer = .);
        PROVIDE(bss_end = .);
    }
    .table_heap (NOLOAD) : ALIGN(4K) {
        PROVIDE(table_heap_bottom = .);
        . += 512*1024;
        PROVIDE(table_heap_top = .);
    }
    /* Stack guard should be unmapped in virtual memory */
    .stack_guard (NOLOAD) : ALIGN(4K) {
        PROVIDE(stack_guard = .);
        . += 4096;
    }
    .stack (NOLOAD) : ALIGN(4K) {
        PROVIDE(stack_bottom = .);
        . += 128*1024;
        PROVIDE(stack_top = .);
    }
}
