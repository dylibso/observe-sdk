#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso_observe", "statsd") extern void statsd(uint32_t, uint32_t);
IMPORT("dylibso_observe", "log")
extern void log_write(uint32_t, uint32_t, uint32_t);

void write_stat() {
  char stat[] = "vowels.count:1|c";

  uintptr_t ptr = (uintptr_t)stat;
  uint64_t uint64_ptr = (uint64_t)ptr;
  uint64_t uint64_length = (uint64_t)(strlen(stat));

  statsd(uint64_ptr, uint64_length);
}

void write_log() {
  char stat[] = "Vowels Counted: 3\n";

  uintptr_t ptr = (uintptr_t)stat;
  uint64_t uint64_ptr = (uint64_t)ptr;
  uint64_t uint64_length = (uint64_t)(strlen(stat));
  uint64_t level = (uint64_t)1;

  log_write(level, uint64_ptr, uint64_length);
}

void run() {
  printf("Hello from Wasm!\n");
  write_stat();
  write_log();
}

int main(int argc, char *argv[]) {
  run();

  return 0;
}
