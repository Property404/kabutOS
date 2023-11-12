#include <stdint.h>
#include "console.h"
#include "uart.h"
#include "functions.h"
#include "string.h"
#include "stdio.h"
#include <stdbool.h>

#define INDEX_SIZE 256

static char current_char;
static unsigned index = 0;
static char input_array[INDEX_SIZE];

// TODO: make it impossible to backspace off the line

// sets the first element to 0
// this will stop the array from being printed
void clearArray() {
        input_array[0] = '\0';
        return;
}

void printArray() {
        print(input_array);
	print("\r\n");
}

void resetArray() {
    input_array[index] = '\0';
	printArray();
	clearArray();
	index = 0;
}

// TODO, use array of arguments and pass that
// aaray of char ptrs

void parseArray() {
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
    print("Num of arguments: ");
    printf("%x", numArgs);

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

void storeArray(unsigned char c) {
	current_char = c;

        // check for ENTER key
        if (current_char == '\r') {
            // TODO: because we parse the array
            // before we print it, we end up 
            // printing just the first word
            // which is unintuitive
            // but i'm leaving this as is while we
            // get console.c function call working :)
            parseArray();
            resetArray();
            return;
        }

        // max size of the array.
        if (index >= INDEX_SIZE) {
            print("Array maxed out. Oops!\r\n");
		    resetArray();
            return;
        }

        input_array[index] = current_char;
        index++;
        return;
}

void run_console() {
    if (char_available()) {
         const char c = getchar();
         putchar(c);
         storeArray(c);
    }
}
