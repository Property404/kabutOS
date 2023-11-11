#include <stdint.h>
#include "console.h"
#include "uart.h"

static char current_char;
static uint8_t index;
static const uint8_t INDEX_SIZE = 64;
static char input_array[64];

// TODO: make it impossible to backspace off the line

// TODO, put while loop for console here 

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

char* getArray() {
    // TODO get array after user enters?
}

void storeArray(unsigned char c) {
	current_char = c;

        // check for ENTER key
        if (current_char == '\r') {
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
