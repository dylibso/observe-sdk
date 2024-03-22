#ifndef OBSERVE_API_H
#define OBSERVE_API_H

#include <stdint.h>

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso:observe/api", "metric")
extern void _metric(uint32_t, uint32_t, uint32_t);
IMPORT("dylibso:observe/api", "log")
extern void _log(uint32_t, uint32_t, uint32_t);
IMPORT("dylibso:observe/api", "span-enter")
extern void _span_enter(uint32_t, uint32_t);
IMPORT("dylibso:observe/api", "span-exit")
extern void _span_exit(void);

enum DO_LOG_LEVEL {
  DO_LL_ERROR = 0,
  DO_LL_WARN = 1,
  DO_LL_INFO = 2,
  DO_LL_DEBUG = 3,
  DO_LL_TRACE = 4
};

#ifdef __cplusplus
extern "C" {
#endif

void observe_api_span_enter(const char *name);
void observe_api_span_exit(void);
void observe_api_metric(const char *metric);
void observe_api_write_log(const enum DO_LOG_LEVEL level, const char *msg);

#ifdef __cplusplus
}
#endif

#endif
