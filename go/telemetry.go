package observe

import (
	"encoding/hex"
	"fmt"
	"log"
	"math/rand"
	"strconv"
	"time"
)

// This is a shared type for a span or trace id.
// It's represented by 2 uint64s and can be transformed
// to different string or int representations where needed.
type TelemetryId struct {
	lsb uint64
	msb uint64
}

var rng rand.Source

func init() {
	rng = rand.NewSource(time.Now().UnixNano())
}

// Create a new trace id
func NewTraceId() TelemetryId {
	return TelemetryId{
		msb: uint64(rng.Int63()),
		lsb: uint64(rng.Int63()),
	}
}

// Create a new span id
func NewSpanId() TelemetryId {
	return TelemetryId{
		msb: uint64(rng.Int63()),
		lsb: uint64(rng.Int63()),
	}
}

func TelemetryIdFromString(tid string) (TelemetryId, error) {
	id, err := strconv.ParseInt(tid, 10, 64)
	if err != nil {
		return TelemetryId{}, nil
	}

	return TelemetryId{
		msb: uint64(id) << 4,
		lsb: uint64(id) << 4,
	}, nil
}

type TraceId struct{ TelemetryId }
type SpanId struct{ TelemetryId }

func (id TelemetryId) Msb() uint64 {
	return id.msb
}

func (id TelemetryId) Lsb() uint64 {
	return id.lsb
}

// Encode this id into an 8 byte hex (16 chars)
// Just uses the least significant of the 16 bytes
func (t TelemetryId) ToHex8() string {
	return fmt.Sprintf("%016x", t.lsb)
}

// Encode this id into a 16 byte hex (32 chars)
// Uses both 16 byte uint64 values
func (t TelemetryId) ToHex16() string {
	return fmt.Sprintf("%016x%016x", t.msb, t.lsb)
}

// Some adapters may need a raw representation
func (t TelemetryId) ToUint64() uint64 {
	return t.lsb
}

func (t TelemetryId) ToRawTraceIdBytes() []byte {
	traceId := t.ToHex16()
	traceIdB, err := hex.DecodeString(traceId)
	if err != nil {
		log.Println(traceId, "convert traceid to raw bytes:", err)
		return make([]byte, 0)
	}

	return traceIdB
}

func (t TelemetryId) ToRawSpanIdBytes() []byte {
	spanId := t.ToHex8()
	spanIdB, err := hex.DecodeString(spanId)
	if err != nil {
		log.Println(spanId, "convert spanid to raw bytes:", err)
		return make([]byte, 0)
	}

	return spanIdB
}
