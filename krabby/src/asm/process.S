.option norvc
.section .text

.type run_process, @function
.type idle, @function
.global run_process
.global enter_user_mode
.global idle

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

idle:
	.cfi_startproc
    ecall
    ecall
    ecall
    ecall
    csrrw x31, mstatus, x31
    #wfi
    nop
    nop
    nop
    nop
    j idle
	.cfi_endproc
