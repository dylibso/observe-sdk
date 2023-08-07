#ifndef OBSERVE_API_H
#define OBSERVE_API_H

#include <inttypes.h>

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso_observe", "metric")
extern void _metric(uint32_t, uint64_t, uint32_t);
IMPORT("dylibso_observe", "log")
extern void _log(uint32_t, uint64_t, uint32_t);
IMPORT("dylibso_observe", "span_enter")
extern void _span_enter(uint64_t, uint32_t);
IMPORT("dylibso_observe", "span_exit") extern void _span_exit();

void span_enter(char *metric);
void span_exit();
void metric(char *metric);
void write_log(int level, char *msg);

#endif
