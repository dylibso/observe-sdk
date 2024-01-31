#include "observe_api.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void span_enter(const char *name) {
  const uint32_t uint32_ptr = (uint32_t)name;
  const uint32_t uint32_length = strlen(name);
  _span_enter(uint32_ptr, uint32_length);
}

void span_exit(void) { _span_exit(); }

void metric(const char *metric) {
  const uint32_t uint32_ptr = (uint32_t)metric;
  const uint32_t uint32_length = strlen(metric);
  _metric(1, uint32_ptr, uint32_length);
}

void write_log(const enum DO_LOG_LEVEL level, const char *msg) {
  const uint32_t uint32_ptr = (uint32_t)msg;
  const uint32_t uint32_length = strlen(msg);
  const uint32_t uint32_level = level;
  _log(uint32_level, uint32_ptr, uint32_length);
}
