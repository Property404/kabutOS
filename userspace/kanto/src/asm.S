.option norvc
.section .text

.type asm_syscall, @function
.global asm_syscall

asm_syscall:
    ecall
    ret
