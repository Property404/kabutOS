target remote localhost:1234
#symbol-file
#add-symbol-file target/riscv64gc-unknown-none-elf/debug/krabby 0x80000000
break exception_handler
break asm_exception_handler
break kmain
layout asm
