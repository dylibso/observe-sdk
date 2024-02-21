#include <stdio.h>
#define EXPORT_AS(name) __attribute__((export_name(name)))

EXPORT_AS("hello") void hello(void) { printf("hello world\n"); }
