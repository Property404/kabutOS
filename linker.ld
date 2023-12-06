ENTRY(start);

. = 0x80000000;

SECTIONS {
	/* Include entry point at start of binary */
	.text : ALIGN(4K) {
		*(.init);
		*(.text);
	}
	.bss : ALIGN(4K) {
		PROVIDE(bss_start = .);
		*(.bss);
		. += 8192;
		PROVIDE(stack_top = .);
		. += 8192;
		PROVIDE(global_pointer = .);
		PROVIDE(bss_end = .);
	}
	.rodata : ALIGN(4K) {
		*(.rodata);
        /*
         * Solves issue of rodata* being put before .text
         * https://stackoverflow.com/questions/43727214/linker-seems-to-be-placing-data-in-wrong-section
         */
		*(.rodata*);
	}
	.data : ALIGN(4K) {
		*(.data);
	}
}
