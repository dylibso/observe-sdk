package observe

import (
	"fmt"
	"testing"
)

func TestTraceId(t *testing.T) {
	a := NewTraceId()

	b := TelemetryId{}
	if err := b.FromString(a.ToHex16()); err != nil {
		t.Error(err)
	}

	fmt.Println(a.ToHex16(), b.ToHex16())
	fmt.Println(a, b)

	if a != b {
		t.Fail()
	}

	// err := b.FromString(a.ToHex16() + "a")
	// if err == nil {
	// 	t.Fail()
	// }
}
