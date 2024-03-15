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
        *(.init);
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
    .bss : ALIGN(4K) {
        PROVIDE(bss_start = .);
        *(.bss);
        *(.bss*);
        PROVIDE(global_pointer = .);
        PROVIDE(bss_end = .);
    }
    .heap : ALIGN(4K) {
        *(.heap*);
    }
    .kernel_root_page_table : ALIGN(4K) {
        *(.kernel_root_page_table*);
    }
    .table_heap : ALIGN(4K) {
        PROVIDE(table_heap_bottom = .);
        . += 256*1024;
        PROVIDE(table_heap_top = .);
    }
    /* Stack guard should be unmapped in virtual memory */
    .stack_guard : ALIGN(4K) {
        PROVIDE(stack_guard = .);
        . += 4096;
    }
    .stack : ALIGN(4K) {
        PROVIDE(stack_bottom = .);
        . += 64*1024;
        PROVIDE(stack_top = .);
    }
}
