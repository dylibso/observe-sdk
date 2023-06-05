#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
  const char *name = "world";
  if (argc > 1) {
    name = argv[1];
  }

  void *memory[10];
  for (int i = 0; i < 10; i++) {
    printf("Hello, %s!\n", name);
    memory[i] = malloc(65536 * 3);
  }

  for (int i = 0; i < 10; i++) {
    free(memory[i]);
  }

  return 0;
}
