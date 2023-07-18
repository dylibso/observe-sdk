package observe

import (
	"fmt"
	"math/rand"
	"time"
)

type TelemetryId uint64

var rng rand.Source

func init() {
	rng = rand.NewSource(time.Now().UnixNano())
}

func NewTraceId() TelemetryId {
	return TelemetryId(rng.Int63())
}

func NewSpanId() TelemetryId {
	return TelemetryId(rng.Int63())
}

func (t TelemetryId) ToHex8() string {
	return fmt.Sprintf("%016x", t)
}

func (t TelemetryId) ToHex16() string {
	return fmt.Sprintf("%032x", t)
}
