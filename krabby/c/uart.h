#pragma once
#include <stdbool.h>

void uart_init(void);

int putchar(char c);

char getchar();

bool char_available();
