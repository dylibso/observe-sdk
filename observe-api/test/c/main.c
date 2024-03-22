#include "../../c/observe_api.h"
#include <stdio.h>
#include <stdlib.h>

void run() {
  observe_api_span_enter("printf");
  printf("Hello from Wasm!\n");
  observe_api_span_exit();
}

int main(int argc, char *argv[]) {
  observe_api_span_enter("run");
  run();
  observe_api_span_exit();

  return 0;
}
