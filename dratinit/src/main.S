.option norvc
.section .text

.type start, @function
.global start

start:
li a7, 1
li a0, 'h'
ecall
li a0, 'e'
ecall
li a0, 'l'
ecall
ecall
li a0, 'o'
ecall
li a0, '!'
ecall
li a0, '\n'
ecall

1:
    nop
j 1b
