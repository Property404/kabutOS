#include "functions.h"
#include "demo.h"
#include "dump.h"
#include "stdio.h"
#include "string.h"
static int sys_demo(int num, char** argv);
static int sys_dump(int num, char** argv);

struct function_pointers fp[] = {
    {"test",sys_demo},
    {"dump",sys_dump},
};

static int sys_demo(int num, char** argv) {
    // we're not parsing the commands here, but you can!
    (void)num;
    (void)argv;

    demo();

    return 0;
}

static int sys_dump(int num, char** argv) {
    if (num < 3) {
        printf("Not enough arguments, dummy\n");
    }
    const uint8_t* pointer = string_to_pointer(argv[1]);
    const size_t length = string_to_u64(argv[2]);
    dump_memory(pointer, length);
    return 0;
}
