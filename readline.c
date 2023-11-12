#include <stdint.h>
#include "console.h"
#include "uart.h"
#include "functions.h"
#include "string.h"
#include "stdio.h"
#include <stdbool.h>

char nextchar() {
    while (!char_available());
    return getchar();
}

size_t readline(char* array, size_t max_size) {
    size_t ptr = 0;
    size_t length = 0;
    while (true) {
        const char c = nextchar();
        switch (c) {
            case '\r':{
                putchar('\n');
                array[length] = '\0';
                return length;
            };
            case 0x7f:
                if (ptr > 0) {
                    ptr--;
                    length--;
                    puts("\b \b");
                }
                break;
            // Escape
            case 0x1b:
                // Arrow keys
                if (nextchar() == '[') {
                    const char direction = nextchar();
                    // Left
                    /*
                    if (direction == 'D' && ptr > 0) {
                        ptr--;
                        putchar('\b');
                    } else if (direction == 'C' && ptr < length) {
                        ptr++;
                        putchar(' ');
                    }
                    */
                }
                break;
            default:
                if (c >= 0x20 && c < 0x7F && length < max_size) { 
                    putchar(c);
                    array[length] = c;
                    length++;
                    ptr++;
                } else {
                    printf("(%02x)", c);
                }
        }
    }
}
