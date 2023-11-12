#pragma once
#include <stdbool.h>
#include <stdarg.h>

bool testchar(void);
int puts(const char* buffer);
int printf(const char* fmt, ...);
int vprintf(const char* fmt, va_list va_args);
