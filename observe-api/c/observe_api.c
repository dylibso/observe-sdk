#include "observe_api.h"
#include <stdint.h>
#include <stdio.h>
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
  char *tags_buf = NULL;
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
