#ifndef OBSERVE_API_HPP
#define OBSERVE_API_HPP

#include "observe_api.h"
#include <string>
#include <string_view>
#include <vector>

namespace observe_api {
void span_enter(std::string_view name);
void span_exit();
void metric(enum DO_METRIC_FMT format, std::string_view metric);
void log(enum DO_LOG_LEVEL level, std::string_view message);
void span_tags(std::string_view tags);
void statsd(std::string_view metric);
void span_tags(std::vector<std::string> &tags);
}; // namespace observe_api

#endif // OBSERVE_API_HPP

// avoid greying out the implementation section
#if defined(Q_CREATOR_RUN) || defined(__INTELLISENSE__) ||                     \
    defined(_CDT_PARSER__)
#define OBSERVE_API_CPP_IMPLEMENTATION
#endif

#ifdef OBSERVE_API_CPP_IMPLEMENTATION
#ifndef OBSERVE_API_CPP
#define OBSERVE_API_CPP

#include "observe_api.h"
#include <iterator>
#include <sstream>
#include <string>
#include <string_view>
#include <vector>

namespace observe_api {

void span_enter(std::string_view name) {
  observe_api_span_enter_n(name.data(), name.size());
}

void span_exit() { observe_api_span_exit(); }

void metric(enum DO_METRIC_FMT format, std::string_view metric) {
  observe_api_metric_n(format, metric.data(), metric.size());
}

void log(enum DO_LOG_LEVEL level, std::string_view message) {
  observe_api_log_n(level, message.data(), message.size());
}

void span_tags(std::string_view tags) {
  observe_api_span_tags_n(tags.data(), tags.size());
}

void statsd(std::string_view metric) {
  observe_api_statsd_n(metric.data(), metric.size());
}

void span_tags(std::vector<std::string> &tags) {
  const char *delim = ",";
  std::ostringstream imploded;
  std::copy(tags.begin(), tags.end(),
            std::ostream_iterator<std::string>(imploded, delim));
  std::string str = imploded.str();
  if (str.size() > 0) {
    observe_api_span_tags_n(str.data(), str.size() - 1);
  }
}

}; // namespace observe_api

#endif // OBSERVE_API_CPP
#endif // OBSERVE_API_CPP_IMPLEMENTATION
