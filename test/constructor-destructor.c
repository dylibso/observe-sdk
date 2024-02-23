#include <stdio.h>

__attribute__((constructor)) static void constructor(void) {
  printf("constructor\n");
}

int main(void) {}

__attribute__((destructor)) static void destructor(void) {
  printf("destructor\n");
}
