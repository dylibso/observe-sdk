#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso_observe", "metric")
extern void metric(uint32_t, uint32_t, uint32_t);
IMPORT("dylibso_observe", "log")
extern void log_write(uint32_t, uint32_t, uint32_t);
IMPORT("dylibso_observe", "span_enter")
extern void span_enter(uint32_t, uint32_t);
IMPORT("dylibso_observe", "span_exit") extern void span_exit();

void custom_span_enter(char name[]) {
  uintptr_t ptr = (uintptr_t)name;
  uint64_t uint64_ptr = (uint64_t)ptr;
  uint64_t uint64_length = (uint64_t)(strlen(name));

  span_enter(uint64_ptr, uint64_length);
}

void custom_span_exit() { span_exit(); }

void write_stat() {
  char stat[] = "vowels.count:1|c";

  uintptr_t ptr = (uintptr_t)stat;
  uint64_t uint64_ptr = (uint64_t)ptr;
  uint64_t uint64_length = (uint64_t)(strlen(stat));

  custom_span_enter("statsd");
  metric((uint64_t)1, uint64_ptr, uint64_length);
  custom_span_exit();
}

void write_log() {
  char stat[] = "Vowels Counted: 3\n";

  uintptr_t ptr = (uintptr_t)stat;
  uint64_t uint64_ptr = (uint64_t)ptr;
  uint64_t uint64_length = (uint64_t)(strlen(stat));
  uint64_t level = (uint64_t)1;

  custom_span_enter("log_write");
  log_write(level, uint64_ptr, uint64_length);
  custom_span_exit();
}

void run() {
  custom_span_enter("printf");
  printf("Hello from Wasm!\n");
  custom_span_exit();

  custom_span_enter("write_stat");
  write_stat();
  custom_span_exit();

  custom_span_enter("write_log");
  write_log();
  custom_span_exit();
}

int main(int argc, char *argv[]) {
  custom_span_enter("run");
  run();
  custom_span_exit();

  return 0;
}
