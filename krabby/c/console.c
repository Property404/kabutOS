#include <stdint.h>
#include "console.h"
#include "readline.h"
#include "functions.h"
#include "string.h"
#include "stdio.h"
#include <stdbool.h>

// TODO, use array of arguments and pass that
// aaray of char ptrs
void parseArray(char* input_array) {
    // return on empty enter press
    if (input_array[0] == '\0') {
        return;
    }

    // TODO: fix this issue so I can set to 0
    int numArgs = -1;
    char* str;

    // had really weird issues referencing
    // array directly, so cast to ptr here
    char* arrayptr = input_array;

    // using a dowhile as str would start null otherwise
    do {
        str = strsep(&arrayptr, " ");
        numArgs++;
    }
    while(str != NULL);

    // DEBUG
    // TODO, we should pass this out
    printf("[DEBUG] Num of arguments: %x\n", numArgs);

    // the value of input_array we're left with
    // points to just the first word
    // the spaces have been replaced with nulls
    // so we don't need to copy the command
    // instead we can just investigate the
    // value of that word
    // so we can add a command here to
    // parseCommand, which will use the
    // function lookup table
}


void run_console() {
     static char input_array[256];
     const int numbytes = readline(input_array, 256);
     printf("[DEBUG]%02x|%s|\n", numbytes, input_array);
     parseArray(input_array);
}
