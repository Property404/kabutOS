// Some of this code is taken from https://osblog.stephenmarz.com/ch4.html
.option norvc
.section .init

.type asm_exception_handler, @function
.global asm_exception_handler

.set TF_GP_REG_STORE_SIZE, 0
.set REG_SIZE, 8 // in bytes

asm_exception_handler:
	.cfi_startproc
    csrrw t6, sscratch, t6

    sd x1, (TF_GP_REG_STORE_SIZE + 1 * REG_SIZE)(t6)
    sd x2, (TF_GP_REG_STORE_SIZE + 2 * REG_SIZE)(t6)
    sd x3, (TF_GP_REG_STORE_SIZE + 3 * REG_SIZE)(t6)
    sd x4, (TF_GP_REG_STORE_SIZE + 4 * REG_SIZE)(t6)
    sd x5, (TF_GP_REG_STORE_SIZE + 5 * REG_SIZE)(t6)
    sd x6, (TF_GP_REG_STORE_SIZE + 6 * REG_SIZE)(t6)
    sd x7, (TF_GP_REG_STORE_SIZE + 7 * REG_SIZE)(t6)
    sd x8, (TF_GP_REG_STORE_SIZE + 8 * REG_SIZE)(t6)
    sd x9, (TF_GP_REG_STORE_SIZE + 9 * REG_SIZE)(t6)
    sd x10, (TF_GP_REG_STORE_SIZE + 10 * REG_SIZE)(t6)
    sd x11, (TF_GP_REG_STORE_SIZE + 11 * REG_SIZE)(t6)
    sd x12, (TF_GP_REG_STORE_SIZE + 12 * REG_SIZE)(t6)
    sd x13, (TF_GP_REG_STORE_SIZE + 13 * REG_SIZE)(t6)
    sd x14, (TF_GP_REG_STORE_SIZE + 14 * REG_SIZE)(t6)
    sd x15, (TF_GP_REG_STORE_SIZE + 15 * REG_SIZE)(t6)
    sd x16, (TF_GP_REG_STORE_SIZE + 16 * REG_SIZE)(t6)
    sd x17, (TF_GP_REG_STORE_SIZE + 17 * REG_SIZE)(t6)
    sd x18, (TF_GP_REG_STORE_SIZE + 18 * REG_SIZE)(t6)
    sd x19, (TF_GP_REG_STORE_SIZE + 19 * REG_SIZE)(t6)
    sd x20, (TF_GP_REG_STORE_SIZE + 20 * REG_SIZE)(t6)
    sd x21, (TF_GP_REG_STORE_SIZE + 21 * REG_SIZE)(t6)
    sd x22, (TF_GP_REG_STORE_SIZE + 22 * REG_SIZE)(t6)
    sd x23, (TF_GP_REG_STORE_SIZE + 23 * REG_SIZE)(t6)
    sd x24, (TF_GP_REG_STORE_SIZE + 24 * REG_SIZE)(t6)
    sd x25, (TF_GP_REG_STORE_SIZE + 25 * REG_SIZE)(t6)
    sd x26, (TF_GP_REG_STORE_SIZE + 26 * REG_SIZE)(t6)
    sd x27, (TF_GP_REG_STORE_SIZE + 27 * REG_SIZE)(t6)
    sd x28, (TF_GP_REG_STORE_SIZE + 28 * REG_SIZE)(t6)
    sd x29, (TF_GP_REG_STORE_SIZE + 29 * REG_SIZE)(t6)
    sd x30, (TF_GP_REG_STORE_SIZE + 30 * REG_SIZE)(t6)

    // Store our temp register
    mv s5, t6
    csrrw t6, sscratch, t6
    sd x31, (TF_GP_REG_STORE_SIZE + 31 * REG_SIZE)(s5)

    call exception_handler

    // Restore registers and return
    mv t6, s5
    ld x1, (TF_GP_REG_STORE_SIZE + 1 * REG_SIZE)(t6)
    ld x2, (TF_GP_REG_STORE_SIZE + 2 * REG_SIZE)(t6)
    ld x3, (TF_GP_REG_STORE_SIZE + 3 * REG_SIZE)(t6)
    ld x4, (TF_GP_REG_STORE_SIZE + 4 * REG_SIZE)(t6)
    ld x5, (TF_GP_REG_STORE_SIZE + 5 * REG_SIZE)(t6)
    ld x6, (TF_GP_REG_STORE_SIZE + 6 * REG_SIZE)(t6)
    ld x7, (TF_GP_REG_STORE_SIZE + 7 * REG_SIZE)(t6)
    ld x8, (TF_GP_REG_STORE_SIZE + 8 * REG_SIZE)(t6)
    ld x9, (TF_GP_REG_STORE_SIZE + 9 * REG_SIZE)(t6)
    ld x10, (TF_GP_REG_STORE_SIZE + 10 * REG_SIZE)(t6)
    ld x11, (TF_GP_REG_STORE_SIZE + 11 * REG_SIZE)(t6)
    ld x12, (TF_GP_REG_STORE_SIZE + 12 * REG_SIZE)(t6)
    ld x13, (TF_GP_REG_STORE_SIZE + 13 * REG_SIZE)(t6)
    ld x14, (TF_GP_REG_STORE_SIZE + 14 * REG_SIZE)(t6)
    ld x15, (TF_GP_REG_STORE_SIZE + 15 * REG_SIZE)(t6)
    ld x16, (TF_GP_REG_STORE_SIZE + 16 * REG_SIZE)(t6)
    ld x17, (TF_GP_REG_STORE_SIZE + 17 * REG_SIZE)(t6)
    ld x18, (TF_GP_REG_STORE_SIZE + 18 * REG_SIZE)(t6)
    ld x19, (TF_GP_REG_STORE_SIZE + 19 * REG_SIZE)(t6)
    ld x20, (TF_GP_REG_STORE_SIZE + 20 * REG_SIZE)(t6)
    ld x21, (TF_GP_REG_STORE_SIZE + 21 * REG_SIZE)(t6)
    ld x22, (TF_GP_REG_STORE_SIZE + 22 * REG_SIZE)(t6)
    ld x23, (TF_GP_REG_STORE_SIZE + 23 * REG_SIZE)(t6)
    ld x24, (TF_GP_REG_STORE_SIZE + 24 * REG_SIZE)(t6)
    ld x25, (TF_GP_REG_STORE_SIZE + 25 * REG_SIZE)(t6)
    ld x26, (TF_GP_REG_STORE_SIZE + 26 * REG_SIZE)(t6)
    ld x27, (TF_GP_REG_STORE_SIZE + 27 * REG_SIZE)(t6)
    ld x28, (TF_GP_REG_STORE_SIZE + 28 * REG_SIZE)(t6)
    ld x29, (TF_GP_REG_STORE_SIZE + 29 * REG_SIZE)(t6)
    ld x30, (TF_GP_REG_STORE_SIZE + 30 * REG_SIZE)(t6)

    sret
	.cfi_endproc


