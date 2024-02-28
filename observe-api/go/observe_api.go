package observe_api

import (
	"reflect"
	"strings"
	"unsafe"
)

//go:wasmimport dylibso:observe/api metric
func metric(uint32, uint32, uint32)

//go:wasmimport dylibso:observe/api log
func log(uint32, uint32, uint32)

//go:wasmimport dylibso:observe/api span-tags
func span_tags(uint32, uint32)

//go:wasmimport dylibso:observe/api span-enter
func span_enter(uint32, uint32)

//go:wasmimport dylibso:observe/api span-exit
func span_exit()

func stringPointer(s *string) uint32 {
	header := (*reflect.StringHeader)(unsafe.Pointer(s))
	return uint32(header.Data)
}

type MetricFormat int

const (
	Statsd MetricFormat = 1 + iota
)

type LogLevel int

const (
	Error LogLevel = 1 + iota
	Warn
	Info
	Debug
	Trace
)

func Metric(format MetricFormat, m string) {
	ptr := stringPointer(&m)
	metric(uint32(format), ptr, uint32(len(m)))
}

func Log(level LogLevel, msg string) {
	ptr := stringPointer(&msg)
	log(uint32(level), ptr, uint32(len(msg)))
}

func SpanTags(tags []string) {
	s := strings.Join(tags[:], ",")
	ptr := stringPointer(&s)
	span_tags(ptr, uint32(len(s)))
}

type span struct {
	name string
	tags []string
}

func NewSpan(name string) span {
	ptr := stringPointer(&name)
	span_enter(ptr, uint32(len(name)))
	tags := []string{}
	return span{name, tags}
}

func (s span) End() {
	span_exit()
}

func (s span) AddTags(tags ...string) {
	SpanTags(tags)
}
