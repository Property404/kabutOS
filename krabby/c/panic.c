#include "panic.h"
#include "stdio.h"
#include "uart.h"
#include <stddef.h>

void halt() {
    while (true) {
    }
}

void kpanic(const char* fmt, ...) {
    // Let's provide the option to not print anything
    // if serial isn't set up
    if (fmt != NULL) {
        puts("PANIC! ");

        va_list args;
        va_start(args, fmt);
        vprintf(fmt, args);
        va_end(args);

        putchar('\n');
    }

    halt();
}
