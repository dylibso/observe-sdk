#include "observe_api.h"
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso_observe", "metric")
extern void _metric(uint32_t, uint64_t, uint32_t);
IMPORT("dylibso_observe", "log")
extern void _log(uint32_t, uint64_t, uint32_t);
IMPORT("dylibso_observe", "span_enter")
extern void _span_enter(uint64_t, uint32_t);
IMPORT("dylibso_observe", "span_exit") extern void _span_exit();

void span_enter(char *name) {
  uint64_t uint64_ptr = (uint64_t)name;
  uint32_t uint32_length = (uint32_t)(strlen(name));
  _span_enter(uint64_ptr, uint32_length);
}

void span_exit() { _span_exit(); }

void metric(char *metric) {
  uint64_t uint64_ptr = (uint64_t)metric;
  uint32_t uint32_length = (uint32_t)(strlen(metric));
  _metric((uint64_t)1, uint64_ptr, uint32_length);
}

void write_log(int level, char *msg) {
  uint64_t uint64_ptr = (uint64_t)msg;
  uint32_t uint32_length = (uint32_t)(strlen(msg));
  uint32_t uint32_level = (uint32_t)level;
  _log(uint32_level, uint64_ptr, uint32_length);
}