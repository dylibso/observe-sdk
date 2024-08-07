#ifndef OBSERVE_API_H
#define OBSERVE_API_H

#include <stdint.h>

#ifndef OBSERVE_API_ENUM
#define OBSERVE_API_ENUM
enum DO_LOG_LEVEL {
  DO_LL_ERROR = 1,
  DO_LL_WARN = 2,
  DO_LL_INFO = 3,
  DO_LL_DEBUG = 4,
  DO_LL_TRACE = 5
};

enum DO_METRIC_FMT { DO_MF_STATSD = 1 };
#endif

#define IMPORT(a, b) __attribute__((import_module(a), import_name(b)))

IMPORT("dylibso:observe/api", "metric")
extern void observe_api_metric_n(enum DO_METRIC_FMT format, const char *metric,
                                 size_t metric_length);
IMPORT("dylibso:observe/api", "log")
extern void observe_api_log_n(enum DO_LOG_LEVEL level, const char *message,
                              size_t message_length);
IMPORT("dylibso:observe/api", "span-enter")
extern void observe_api_span_enter_n(const char *name, size_t name_length);
IMPORT("dylibso:observe/api", "span-exit")
extern void observe_api_span_exit(void);
IMPORT("dylibso:observe/api", "span-tags")
extern void observe_api_span_tags_n(const char *tags, size_t tags_length);

#undef IMPORT

#ifdef __cplusplus
extern "C" {
#endif

void observe_api_span_enter(const char *name);
void observe_api_metric(enum DO_METRIC_FMT format, const char *message);
void observe_api_statsd_n(const char *metric, const size_t metric_length);
void observe_api_statsd(const char *metric);
void observe_api_log(const enum DO_LOG_LEVEL level, const char *msg);
void observe_api_span_tags(const char *tags);
void observe_api_span_tags_from_array(const char *const tags[],
                                      size_t num_tags);

#ifdef __cplusplus
}
#endif

#endif // OBSERVE_API_H

// avoid greying out the implementation section
#if defined(Q_CREATOR_RUN) || defined(__INTELLISENSE__) ||                     \
    defined(_CDT_PARSER__)
#define OBSERVE_API_IMPLEMENTATION
#endif

#ifdef OBSERVE_API_IMPLEMENTATION
#ifndef OBSERVE_API_C
#define OBSERVE_API_C

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

void observe_api_span_enter(const char *name) {
  const size_t name_length = strlen(name);
  observe_api_span_enter_n(name, name_length);
}

void observe_api_metric(enum DO_METRIC_FMT format, const char *metric) {
  const size_t metric_length = strlen(metric);
  observe_api_metric_n(format, metric, metric_length);
}

void observe_api_statsd_n(const char *metric, const size_t metric_length) {
  observe_api_metric_n(DO_MF_STATSD, metric, metric_length);
}

void observe_api_statsd(const char *metric) {
  observe_api_metric(DO_MF_STATSD, metric);
}

void observe_api_log(const enum DO_LOG_LEVEL level, const char *msg) {
  const size_t msg_length = strlen(msg);
  observe_api_log_n(level, msg, msg_length);
}

void observe_api_span_tags(const char *tags) {
  const size_t tags_length = strlen(tags);
  observe_api_span_tags_n(tags, tags_length);
}

void observe_api_span_tags_from_array(const char *const tags[],
                                      size_t num_tags) {
  char *tags_buf = 0;
  size_t tags_buf_size = 0;
  for (size_t i = 0; i < num_tags; i++) {
    size_t new_tag_length = strlen(tags[i]);
    size_t new_tags_buf_size = tags_buf_size + new_tag_length + 1;
    char *new_tags_buf = (char *)realloc(tags_buf, new_tags_buf_size);
    if (!new_tags_buf) {
      break;
    }
    memcpy(new_tags_buf + tags_buf_size, tags[i], new_tag_length);
    new_tags_buf[new_tags_buf_size - 1] = ',';
    tags_buf = new_tags_buf;
    tags_buf_size = new_tags_buf_size;
  }
  if (tags_buf_size > 0) {
    observe_api_span_tags_n(tags_buf, tags_buf_size - 1);
    free(tags_buf);
  }
}

#endif // OBSERVE_API_C
#endif // OBSERVE_API_IMPLEMENTATION
