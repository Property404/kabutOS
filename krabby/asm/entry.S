.section .init

# Disable compression
.option norvc

.type start, @function
.global start
start:
	.cfi_startproc

.option push
# Disables relaxation. The RISC-V assembler and linker opportunistically relax some code sequences, but sometimes this behavior is not desirable.
#  Linker relaxation is the process of finding shorted opcodes for jumps to addresses nearby.
#
# For example, an instruction set might define two jump operations. One with a 32 bits offset, and one with an 8 bits offset. Most likely the compiler will generate conservative code, so always 32 bits branches. During the relaxation phase, the code is scanned for possible replacements of the 32 bits jump by an 8 bit jump.
#
# Possible issues that might occur during this phase:
#    alignment of code. Code that was previously aligned might be shifted.
#    Linker relaxations might cause the opposite effect on jumps whose distance increases due to relaxation. This occurs when jumping over a memory whole between sections.
.option norelax
	la gp, global_pointer
.option pop

	/* Reset satp */
	csrw satp, zero

	/* Setup stack */
	la sp, stack_top

	/* Clear the BSS section */
	la t5, bss_start
	la t6, bss_end
bss_clear:
	sd zero, (t5)
	addi t5, t5, 8
	bltu t5, t6, bss_clear

	la t0, kmain
	csrw mepc, t0

    // Set exception handler
    la t0, asm_exception_handler
    csrw mtvec, t0

	/* Jump to kernel! */
	tail kmain

	.cfi_endproc

asm_exception_handler:
	.cfi_startproc
    // Todo: Preserve registers on stack
    call exception_handler
	.cfi_endproc

.end
