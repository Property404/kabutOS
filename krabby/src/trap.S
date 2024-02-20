.option norvc
.section .init

.type asm_exception_handler, @function
.global asm_exception_handler
.global arr

asm_exception_handler:
	.cfi_startproc

    call exception_handler
	.cfi_endproc

.section .data_asm
reg_store:
.fill 32 * 8 * 4

