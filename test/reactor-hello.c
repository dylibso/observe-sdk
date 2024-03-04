#include <stdio.h>
#define EXPORT_AS(name) __attribute__((export_name(name)))

__attribute__((constructor)) static void constructor(void) {
  printf("constructor\n");
}

EXPORT_AS("hello") void hello(void) { printf("hello world\n"); }

EXPORT_AS("__wasm_call_dtors") void __wasm_call_dtors(void) {
  printf("real destructor\n");
}
