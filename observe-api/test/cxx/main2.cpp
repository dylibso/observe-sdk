#define OBSERVE_API_CPP_IMPLEMENTATION
#include "observe_api.hpp"
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <vector>

void run() {
  auto span = observe_api::Span("printf");
  span.statsd("ok:aaaaa");
  observe_api::log(DO_LL_INFO, "bbbbb");
  span.tags("abbc:def,(another:tag");
  std::vector<std::string> tags = {"taga:one", "tagb:two"};
  span.tags(tags);
  printf("Hello from Wasm!\n");
}

int main(int argc, char *argv[]) {
  auto span = observe_api::Span("run");
  run();
  return 0;
}
