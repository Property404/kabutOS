#include <stdint.h>
#include "console.h"
#include "uart.h"
#include "functions.h"
#include "string.h"
#include "stdio.h"

#define INDEX_SIZE 256

static char current_char;
static uint8_t index = 0;
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

void parseArray() {
    print("Parsing Array...\r\n");
    char* str;
    int numArgs = 0;
    do {
	    // grab the first command
        printArray();
        print ("Debug 2\r\n");
        str = strsep((char**)(input_array), " ");
        numArgs++;
        print("DEBUG!\r\n");
    }
    while(str != NULL);

    print("Num of arguments: ");
    printf("%x", numArgs);
    print("\r\n");
}

void storeArray(unsigned char c) {
	current_char = c;

        // check for ENTER key
        if (current_char == '\r') {
            parseArray();
            resetArray();
        }

        // max size of the array.
        if (index >= INDEX_SIZE) {
                print("Array maxed out. Oops!\r\n");
		resetArray();
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
