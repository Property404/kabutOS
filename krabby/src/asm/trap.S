// Some of this code is taken from https://osblog.stephenmarz.com/ch4.html
.option norvc
.section .text

.type asm_exception_handler, @function
.global asm_exception_handler

.type enter_user_mode, @function
.global enter_user_mode

asm_exception_handler:
    .cfi_startproc
    // Save off registers in current trap frame
    csrrw t6, sscratch, t6
    save_x1_to_x30
    mv s5, t6
    csrrw t6, sscratch, t6 // Incoming trap frame has been restored
    save_reg 31 s5

    // Set global pointer
    .option push
    .option norelax
    la gp, global_pointer
    .option pop

    // Use kernel stack
    mv t6, s5
    switch_to_kernel_stack t6

    // Branch to Rust
    call exception_handler

// This is really a separate function
// But we put it together with `asm_exception_handler`
// for optimization
enter_user_mode:

    // Restore registers and return
    csrr t6, sscratch
    load_x1_to_x30
    load_reg 31 t6

    sret
    .cfi_endproc
