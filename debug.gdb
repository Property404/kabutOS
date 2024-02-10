target remote localhost:1234
#symbol-file
#add-symbol-file target/riscv64gc-unknown-none-elf/debug/krabby 0x80000000
break exception_handler
break asm_exception_handler
break enter_supervisor_mode
break *0x800010a8
break *0x8000107c
break *0x8001a30
break *0x000010a8
break *0x0000107c
break *0x0001a30
layout asm
