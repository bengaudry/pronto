#ifndef INLINE_UTILS_H
#define INLINE_UTILS_H

#include <stdio.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))

static inline void print_inline_hello(void) {
    printf("Hello from inline header!\n");
}

#endif
