#define OBSERVE_API_CPP_IMPLEMENTATION
#include "observe_api.hpp"
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <vector>

void run() {
  observe_api::span_enter("printf");
  observe_api::statsd("ok:aaaaa");
  observe_api::log(DO_LL_INFO, "bbbbb");
  observe_api::span_tags("abbc:def,(another:tag");
  std::vector<std::string> tags = {"taga:one", "tagb:two"};
  observe_api::span_tags(tags);
  printf("Hello from Wasm!\n");
  observe_api::span_exit();
}

int main(int argc, char *argv[]) {
  observe_api::span_enter("run");
  run();
  observe_api::span_exit();
  return 0;
}
