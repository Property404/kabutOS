#pragma once
#include <stddef.h>
#include <stdint.h>

// XXD-like memory dump
// Arguments:
//  ptr - pointer to memory
//  size - number of bytes to dump
int dump_memory(const uint8_t* ptr, size_t size);
