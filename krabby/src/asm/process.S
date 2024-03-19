.option norvc
.section .text

.type run_process, @function
.global run_process

run_process:
	.cfi_startproc
    // Set user context
    csrw sepc, a0

    // Load registers from user trap frame
    csrr t6, sscratch
    load_x1_to_x30
    load_reg 31 t6

    // Branch to user mode
    sret

	.cfi_endproc
