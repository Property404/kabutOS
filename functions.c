#include "functions.h"
#include "demo.h"

struct function_pointers fp[] = {
    {"test",sys_demo}
};

int sys_demo(int num, char** args) {
    // we're not parsing the commands here, but you can!

    demo();

    return 0;
}
