.option norvc
.section .text

.type start, @function
.global start


start:
    call main
    1:
        nop
    j 1b
