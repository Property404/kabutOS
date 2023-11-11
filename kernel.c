#include <stdint.h>
#include <stddef.h>
#include "uart.h"
#include "console.h"

void kmain(void) {
    uart_init();

    print("Starting console...\r\n");
    while(1) {
        run_console();
    }
    return;
}
