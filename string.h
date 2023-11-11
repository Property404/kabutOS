#pragma once
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
// Standard C library function replacements
//
// Prototypes may be slightly different
// and do not necessarily conform to C or POSIX
// standards

void* memset(void* dest, int byte, size_t length);
void* memcpy(void*__restrict__ dest, const void*__restrict__  src, size_t length);

int strncmp(const char* s1, const char* s2, size_t length);
int strcmp(const char* s1, const char* s2);
size_t strlen(const char* string);
char* strsep(char** __restrict__ stringp, const char* __restrict__ delim);

/* Check if string contains character */
bool string_has(const char* string, char character);

void* string_to_pointer(const char* str);
uint32_t string_to_u32(const char* str);
