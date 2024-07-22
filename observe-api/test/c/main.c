#define OBSERVE_API_IMPLEMENTATION
#include "observe_api.h"
#include <stdio.h>
#include <stdlib.h>

void run() {
  observe_api_span_enter("printf");
  observe_api_statsd("ok:aaaaa");
  observe_api_log(DO_LL_INFO, "bbbbb");
  observe_api_span_tags("abbc:def,(another:tag");
  const char *const tags[] = {"taga:one", "tagb:two"};
  observe_api_span_tags_from_array(tags, sizeof(tags) / sizeof(tags[0]));
  printf("Hello from Wasm!\n");
  observe_api_span_exit();
}

int main(int argc, char *argv[]) {
  observe_api_span_enter("run");
  run();
  observe_api_span_exit();

  return 0;
}
