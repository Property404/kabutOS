#pragma once

// Function table for use in callbacks with console.c

struct function_pointers {
    char* commandName;
    int (*fnptr)(int, char**);
};

int sys_demo(int, char**);
