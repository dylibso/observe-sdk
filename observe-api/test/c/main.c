#include "../../c/observe_api.h"
#include <stdio.h>
#include <stdlib.h>

void run() {
  span_enter("printf");
  printf("Hello from Wasm!\n");
  span_exit();
}

int main(int argc, char *argv[]) {
  span_enter("run");
  run();
  span_exit();

  return 0;
}
