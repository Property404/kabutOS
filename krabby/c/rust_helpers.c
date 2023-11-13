#include <stdint.h>
uint8_t read_unaligned_volatile_u8(const volatile uint8_t* ptr) {
    return *ptr;
}

void write_unaligned_volatile_u8(volatile uint8_t* ptr, uint8_t value) {
    *ptr = value;
}
