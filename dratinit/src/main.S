.option norvc
.section .text

.type start, @function
.global start

start:
ecall
ecall
ecall
ecall
1:
    nop
j 1b
