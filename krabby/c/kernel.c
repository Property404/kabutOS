#include <stdint.h>
#include <stddef.h>
#include "uart.h"
#include "console.h"
#include "stdio.h"

int32_t snorkel(void);

void kmain(void) {
    uart_init();

    puts("Starting console...\r\n");
    printf("Test: %02x\n", snorkel());
    while(1) {
        run_console();
    }
    return;
}
