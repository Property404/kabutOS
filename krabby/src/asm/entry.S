# Disable compression
.option norvc

.section .init_zero, "ax"
.type preinit, @function
.global preinit
preinit:
    # a0 contains the hart ID and a2 contains the device tree pointer
    # Load PC into a2 and jump to start
    .cfi_startproc
    auipc a2, 0
    j start
    .cfi_endproc

.section .init

.type start, @function
.type enter_supervisor_mode, @function
.global start
.global enter_supervisor_mode
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
    # Disable interrupts and all that jazz
    li t0, 0
    csrw mstatus, t0

    # Get virtual/physical offset
    la t0, virtual_base
    sub a2, a2, t0

    # Before we do anything else, we need to fix up the global offset table
    la t0, got_start
    add t0, t0, a2
    la t1, got_end
    add t1, t1, a2
    fixup_got:
        ld t2, 0(t0)
        add t2, t2, a2
        sd t2, 0(t0)
        # Todo: I think this should be 4 for 32 bits
        addi t0, t0, 8
        bne t0, t1, fixup_got

	la gp, global_pointer
.option pop

    # Have all other harts wait on interrupt
    # (but interrupts are disabled so they'll wait forever)
    csrr a0, mhartid
    beq x0, a0, hart0
nothart0:
    wfi
    j nothart0
hart0:

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

	la t0, boot
	csrw mepc, t0

    // Set exception handler
    la t0, asm_exception_handler
    csrw mtvec, t0

	/* Jump to kernel! */
	tail boot

	.cfi_endproc

// Switch from machine mode to supervisor mode
.option norvc
enter_supervisor_mode:
	.cfi_startproc

    # Machine trap used to forward interrupts to supervisor
    # Needs to happen before fixing up GOT because we use the physical address
    # here
    la      t0, mtrap
    csrw    mtvec, t0

    # We messed up the global offset table, no now we have to fix it again
    la t0, got_start
    la t1, got_end
    refixup_got:
        ld t2, 0(t0)
        sub t2, t2, a0
        sd t2, 0(t0)
        # Todo: I think this should be 4 for 32 bits
        addi t0, t0, 8
        bne t0, t1, refixup_got

.option push
.option norelax
	la gp, global_pointer
.option pop

	la sp, stack_top

    li		t0, (1 << 11) | (1 << 5)
	csrw	mstatus, t0

    # Handle all traps in supervisor mode
    li t0, 0xFFFFFFFF
    csrw medeleg, t0
    li t0, 0xFFFFFFFF
    csrw mideleg, t0

    la t5, sv_entry
    csrw mepc, t5

    # Set initial timer value
    li      t0, 0xFFFFFFFF
    li      t1, 0x2004000
    sd      t0, 0(t1)

    # Enable interrupts
    la      t0, 0x80
    csrs    mie, t0

    mret

    sv_entry:
        la t5, asm_exception_handler
        csrw stvec, t5
        call kmain

    mtrap:
        csrrw t6, mscratch, t6

        # Set SIP
        li t6, 0x02
        csrs mip, t6

        # Reset timer
        li t6, 0x0200BFF8
        sd x0, 0(t6)

        csrrw t6, mscratch, t6
        mret

	.cfi_endproc

.end
