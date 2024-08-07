#ifndef OBSERVE_API_HPP
#define OBSERVE_API_HPP

#include <string>
#include <string_view>
#include <vector>

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

namespace observe_api {
void span_enter(std::string_view name);
void span_exit();
void metric(enum DO_METRIC_FMT format, std::string_view mtc);
void log(enum DO_LOG_LEVEL level, std::string_view message);
void span_tags(std::string_view tags);
void statsd(std::string_view mtc);
void span_tags(std::vector<std::string> &tags);

class Span {
public:
  Span(std::string_view name) { span_enter(name); }
  ~Span() { span_exit(); }
  void metric(enum DO_METRIC_FMT format, std::string_view mtc) {
    observe_api::metric(format, mtc);
  }
  void tags(std::string_view tags) { span_tags(tags); }
  void statsd(std::string_view mtc) { observe_api::statsd(mtc); }
  void tags(std::vector<std::string> &tags) { span_tags(tags); }
};
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

#include <iterator>
#include <sstream>
#include <string>
#include <string_view>
#include <vector>

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

namespace observe_api {

void span_enter(std::string_view name) {
  observe_api_span_enter_n(name.data(), name.size());
}

void span_exit() { observe_api_span_exit(); }

void metric(enum DO_METRIC_FMT format, std::string_view mtc) {
  observe_api_metric_n(format, mtc.data(), mtc.size());
}

void log(enum DO_LOG_LEVEL level, std::string_view message) {
  observe_api_log_n(level, message.data(), message.size());
}

void span_tags(std::string_view tags) {
  observe_api_span_tags_n(tags.data(), tags.size());
}

void statsd(std::string_view mtc) {
  observe_api_metric_n(DO_MF_STATSD, mtc.data(), mtc.size());
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
