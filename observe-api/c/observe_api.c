#include "observe_api.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void span_enter(const char *name) {
  const uint64_t uint64_ptr = (uint64_t)name;
  const uint32_t uint32_length = strlen(name);
  _span_enter(uint64_ptr, uint32_length);
}

void span_exit(void) { _span_exit(); }

void metric(const char *metric) {
  const uint64_t uint64_ptr = (uint64_t)metric;
  const uint32_t uint32_length = strlen(metric);
  _metric(1, uint64_ptr, uint32_length);
}

void write_log(const enum DO_LOG_LEVEL level, const char *msg) {
  const uint64_t uint64_ptr = (uint64_t)msg;
  const uint32_t uint32_length = strlen(msg);
  const uint32_t uint32_level = level;
  _log(uint32_level, uint64_ptr, uint32_length);
}
