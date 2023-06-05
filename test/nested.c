#include <stdio.h>
#include <stdlib.h>

void three() {
  printf("Hello from Wasm!\n");
}

void two() {
  three();
}

void one() {
  two();
}

int main(int argc, char *argv[]) {
  one();
  
  return 0;
}
