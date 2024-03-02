target remote localhost:1234
break exception_handler
break asm_exception_handler
break run_process
break enter_user_mode
#break kmain
#break *0x10000000
layout asm
continue
