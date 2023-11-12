#include <stdint.h>
#include <stddef.h>
#include "uart.h"
#include "console.h"
#include "stdio.h"

void kmain(void) {
    uart_init();

    puts("Starting console...\r\n");
    while(1) {
        run_console();
    }
    return;
}
