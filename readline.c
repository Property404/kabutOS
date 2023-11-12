#include <stdint.h>
#include "panic.h"
#include "console.h"
#include "uart.h"
#include "functions.h"
#include "string.h"
#include "stdio.h"
#include <stdbool.h>

static char nextchar() {
    while (!char_available());
    return getchar();
}

static void skip(const char* array, size_t n) {
    for (size_t i=0; i < n; i++) {
        putchar(array[i]);
    }
}

size_t readline(char* array, size_t max_size) {
    if (array == NULL) {
        kpanic("Array is NULL!");
    }

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
                    // Shift contents left
                    for (size_t i=ptr - 1;i<length;i++) {
                        array[i] = array[i+1];
                    }
                    ptr--;
                    length--;
                }
                break;
            // Escape
            case 0x1b:
                // Arrow keys
                if (nextchar() == '[') {
                    const char direction = nextchar();
                    // Left
                    if (direction == 'D' && ptr > 0) {
                        ptr--;
                    } else if (direction == 'C' && ptr < length) {
                        ptr++;
                    }
                }
                break;
            // CTRL-A - Go to beginning
            case 0x01:
                ptr = 0;
                break;
            // CTRL-B - Move back one
            case 0x02:
                if (ptr > 0) {
                    ptr--;
                }
                break;
            // CTRL-E - Move to end
            case 0x05:
                ptr = length;
                break;
            // CTRL-F - Move forward one
            case 0x06:
                if (ptr < length) {
                    ptr++;
                }
                break;
            default:
                // Insert
                if (c >= 0x20 && c < 0x7F && length < max_size) { 
                    for (size_t i=length;i>ptr;i--) {
                        array[i] = array[i-1];
                    }
                    array[ptr] = c;
                    length++;
                    ptr++;
                }
        }
        array[length] = '\0';
        if (ptr > length) {
            kpanic("Bug: Ptr is greater than length");
        }
        if (strlen(array) != length) {
            kpanic("Bug: Length mismatch(0x%02x vs 0x%02x)!", strlen(array), length);
        }
        printf("\r\x1b[0K%s\r", array);
        skip(array,ptr);
    }
    kpanic("Bug: Unreachable reached");
}
