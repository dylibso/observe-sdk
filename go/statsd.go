package observe

import (
	"fmt"
	"strconv"
	"strings"
)

type StatsdCounterType int

const (
	StatsdCounter StatsdCounterType = iota
	StatsdGauge
	StatsdTiming
	StatsdHistogram
	StatsdSet
)

type StatsdDataGram struct {
	Name          string
	Type          StatsdCounterType
	Value         float64
	SampleRate    float64
	HasSampleRate bool
	Tags          map[string]string
}

func parseStatsdDataGram(message string) (StatsdDataGram, error) {
	// https://docs.datadoghq.com/developers/dogstatsd/datagram_shell/?tab=metrics#the-dogstatsd-protocol
	// <METRIC_NAME>:<VALUE>|<TYPE>|@<SAMPLE_RATE>|#<TAG_KEY_1>:<TAG_VALUE_1>,<TAG_2>

	m := StatsdDataGram{
		Tags: make(map[string]string),
	}

	parts := strings.SplitN(message, ":", 2)
	if len(parts) != 2 {
		return m, fmt.Errorf("Expected a ':' in %s", message)
	}

	// 1. read the metric name
	m.Name = parts[0]

	parts = strings.Split(parts[1], "|")
	if len(parts) < 2 {
		return m, fmt.Errorf("Expected '|' in %s", message)
	}

	// 2. read the metric type
	rawType := parts[1]
	var rawValue string

	// 3. read the metric value
	// TODO: add support for int64 metrics
	// TODO: add support for rate and distribution metrics: https://docs.datadoghq.com/metrics/types/?tab=gauge#metric-types
	switch rawType {
	case "c":
		m.Type = StatsdCounter
		rawValue = parts[0]
	case "g":
		m.Type = StatsdGauge
		part := parts[0]

		// Guages can have a + or - prefix
		if part[0] == '+' || part[0] == '-' {
			rawValue = part[1:]
		} else {
			rawValue = part
		}
	case "ms":
		m.Type = StatsdTiming
		rawValue = parts[0]
	case "h":
		m.Type = StatsdHistogram
		rawValue = parts[0]
	case "s":
		m.Type = StatsdSet
		rawValue = parts[0]
	default:
		return m, fmt.Errorf("Unknown type %s", rawType)
	}

	value, err := strconv.ParseFloat(rawValue, 64)
	if err != nil {
		return m, fmt.Errorf("Failed to parse %s as a float: %e", rawValue, err)
	}

	m.Value = value

	// 4. read the sample rate and tags
	for _, part := range parts[2:] {
		if part[0] == '@' {
			sampleRate, err := strconv.ParseFloat(part[1:], 64)
			if err != nil {
				return m, fmt.Errorf("Failed to parse %s as a float: %e", part[1:], err)
			}

			m.SampleRate = sampleRate
			m.HasSampleRate = true
		} else if part[0] == '#' {
			tags := strings.Split(part[1:], ",")
			for _, tag := range tags {
				tagParts := strings.Split(tag, ":")
				if len(tagParts) != 2 {
					return m, fmt.Errorf("Expected a ':' in %s", tag)
				}

				m.Tags[tagParts[0]] = tagParts[1]
			}
		}
	}

	return m, nil
}
