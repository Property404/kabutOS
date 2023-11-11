#include "dump.h"
#include "string.h"
#include "stdio.h"
#include <stdint.h>

int dump_memory(const uint8_t* ptr, size_t size) {
    const int width = 16;
    while (size > 0) {
        printf("%p: ", ptr);
        for (int minor=0; minor < width; minor+=2) {
            printf(" %02x%02x", *(ptr+minor), *(ptr+minor+1));
        }
        printf("  ");
        for (int minor=0; minor < width; minor++) {
            const char c = *(ptr+minor);
            if (c >= 0x20 && c < 0x7f) {
                printf("%c", c);
            } else {
                printf(".");
            }
        }
        printf("\n");
        size -= width;
        ptr += width;
    }

    return 0;
}
