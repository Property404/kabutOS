#pragma once
#include <stdbool.h>

// Stop forever
void halt();

// Kernel panic!
void kpanic(const char* fmt, ...);
