package observe_api

package main

import (
	"fmt"
	"reflect"
	"strings"
	"unsafe"
)

//go:wasmimport dylibso_observe metric
func metric(uint32, uint64, uint32)

//go:wasmimport dylibso_observe log
func log(uint32, uint64, uint32)

//go:wasmimport dylibso_observe span_tags
func span_tags(uint64, uint32)

//go:wasmimport dylibso_observe span_enter
func span_enter(uint64, uint32)

//go:wasmimport dylibso_observe span_exit
func span_exit()

func stringPointer(s *string) uint64 {
	header := (*reflect.StringHeader)(unsafe.Pointer(s))
	return uint64(header.Data)
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
	return span { name }
}

func (s span) End() {
	span_exit()
}

func (s span) AddTags(tags ...string) {
	SpanTags(tags...)
}