#include "uart.h"
#include <stdbool.h>
#include <stdint.h>
struct Ns16550a {
    uint8_t    dr;        /* 0x00 Data register */
    uint8_t ier;//1
    uint8_t fifo;//2
    uint8_t lcr;//3
    uint8_t _reserved;//4
    uint8_t lsr;//5
} __attribute__((packed));
typedef struct Ns16550a Ns16550a;


static volatile Ns16550a * const uart = (void*)0x10000000;

void uart_init() {
    uart->lcr = 0x3;
    uart->fifo = 0x1;
    uart->ier = 0x1;
}


void putchar(char c) {
    uart->dr = c;
}

char getchar() {
    return uart->dr;
}

bool char_available() {
    if (uart->lsr != 0)
        return true;
    uart->lsr = 0;
    return false;
}

void print(const char * str) {
	while(*str != '\0') {
		putchar(*str);
		str++;
	}
	return;
}
