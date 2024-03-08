.option norvc
.section .text

.type start, @function
.type asm_syscall, @function
.global start
.global asm_syscall


start:
    call main
    1:
        nop
    j 1b

asm_syscall:
    ecall
    ret
