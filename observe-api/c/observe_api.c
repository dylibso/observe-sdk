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
