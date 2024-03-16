target remote localhost:1234
break exception_handler
break asm_exception_handler
break run_process
break enter_user_mode
break mtrap
break kmain
break *0xf0000000
layout asm
continue
