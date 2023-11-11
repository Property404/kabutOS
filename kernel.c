#include <stdint.h>
#include <stddef.h>
#include "uart.h"

void kmain(void) {
	print("Hello world!\r\n");
    uart_init();
	while(1) {
        if (char_available()) {
            putchar(getchar());
        }
	}
	return;
}
