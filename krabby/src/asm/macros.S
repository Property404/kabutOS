// vim: syntax=asm
.set TF_GP_REG_STORE_OFFSET, 0
.set TF_KERNEL_FRAME_OFFSET, 32 * REG_SIZE
.set REG_SIZE, 8 // in bytes

// mstatus register
.set MSTATUS_SIE, (1<<1)
.set MSTATUS_MIE, (1<<3)
.set MSTATUS_SPIE, (1<<5)
.set MSTATUS_MPP_SV_MODE, (0x01<<11)

// mie register
.set MIE_SSOFT, (1<<1)
.set MIE_MTIMER, (1<<7)
.set MIE_MEXT, (1<<11)

// mip register
.set MIP_SSOFT, (1<<1)
.set MIP_MTIMER, (1<<7)
.set MIP_MEXT, (1<<11)

.macro switch_to_kernel_stack frame
    ld \frame, (TF_KERNEL_FRAME_OFFSET)(\frame)
    // Frame is is now kernel frame
    // x2 is stack pointer
    ld sp, (TF_GP_REG_STORE_OFFSET + 2 * REG_SIZE)(\frame)
.endm

.macro save_reg b frame
    sd x\()\b, (TF_GP_REG_STORE_OFFSET + \b * REG_SIZE)(\frame)
.endm

.macro load_reg b frame
    ld x\()\b, (TF_GP_REG_STORE_OFFSET + \b * REG_SIZE)(\frame)
.endm

.macro save_x1_to_x30 frame=x31
    save_reg 1 \frame
    save_reg 2 \frame
    save_reg 3 \frame
    save_reg 4 \frame
    save_reg 5 \frame
    save_reg 6 \frame
    save_reg 7 \frame
    save_reg 8 \frame
    save_reg 9 \frame
    save_reg 10 \frame
    save_reg 11 \frame
    save_reg 12 \frame
    save_reg 13 \frame
    save_reg 14 \frame
    save_reg 15 \frame
    save_reg 16 \frame
    save_reg 17 \frame
    save_reg 18 \frame
    save_reg 19 \frame
    save_reg 20 \frame
    save_reg 21 \frame
    save_reg 22 \frame
    save_reg 23 \frame
    save_reg 24 \frame
    save_reg 25 \frame
    save_reg 26 \frame
    save_reg 27 \frame
    save_reg 28 \frame
    save_reg 29 \frame
    save_reg 30 \frame
.endm

.macro load_x1_to_x30 frame=x31
    load_reg 1  \frame
    load_reg 2  \frame
    load_reg 3  \frame
    load_reg 4  \frame
    load_reg 5  \frame
    load_reg 6  \frame
    load_reg 7  \frame
    load_reg 8  \frame
    load_reg 9  \frame
    load_reg 10 \frame
    load_reg 11 \frame
    load_reg 12 \frame
    load_reg 13 \frame
    load_reg 14 \frame
    load_reg 15 \frame
    load_reg 16 \frame
    load_reg 17 \frame
    load_reg 18 \frame
    load_reg 19 \frame
    load_reg 20 \frame
    load_reg 21 \frame
    load_reg 22 \frame
    load_reg 23 \frame
    load_reg 24 \frame
    load_reg 25 \frame
    load_reg 26 \frame
    load_reg 27 \frame
    load_reg 28 \frame
    load_reg 29 \frame
    load_reg 30 \frame
.endm
