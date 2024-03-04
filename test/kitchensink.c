#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso:observe/api", "metric")
extern void metric(uint32_t, uint32_t, uint32_t);
IMPORT("dylibso:observe/api", "log")
extern void log_write(uint32_t, uint32_t, uint32_t);
IMPORT("dylibso:observe/api", "span-enter")
extern void span_enter(uint32_t, uint32_t);
IMPORT("dylibso:observe/api", "span-exit") extern void span_exit();

void custom_span_enter(const char name[]) {
  const uintptr_t ptr = (uintptr_t)name;
  const uint32_t uint32_ptr = ptr;
  const uint32_t uint32_length = strlen(name);

  span_enter(uint32_ptr, uint32_length);
}

void custom_span_exit() { span_exit(); }

void write_stat() {
  static const char stat[] = "vowels.count:1|c";

  const uintptr_t ptr = (uintptr_t)stat;
  const uint32_t uint32_ptr = ptr;
  const uint32_t uint32_length = strlen(stat);

  custom_span_enter("statsd");
  metric(1, uint32_ptr, uint32_length);
  custom_span_exit();
}

void write_log() {
  static const char stat[] = "Vowels Counted: 3\n";

  const uintptr_t ptr = (uintptr_t)stat;
  const uint32_t uint32_ptr = ptr;
  const uint32_t uint32_length = strlen(stat);
  const uint32_t uint32_level = 1;

  custom_span_enter("log_write");
  log_write(uint32_level, uint32_ptr, uint32_length);
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
