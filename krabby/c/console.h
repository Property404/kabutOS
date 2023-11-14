#pragma once
// Basic simulated kernel shell to run commands

// main for console.c
void run_console();

// return user input array
char* getArray();

void clearArray();

void printArray();

// prints and then clears array
void resetArray();

// store user input into array
void storeArray(unsigned char c);
