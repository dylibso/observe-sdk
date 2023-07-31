package observe

import (
	"testing"
	"time"
)

type TestFlusher struct {
	NumFlushes    int
	FlushedEvents [][]TraceEvent
}

func (f *TestFlusher) Flush(events []TraceEvent) error {
	f.NumFlushes += 1
	f.FlushedEvents = append(f.FlushedEvents, events)
	return nil
}

func mockEvent() Event {
	return CallEvent{
		Time:     time.Now(),
		Duration: time.Duration(1 * time.Millisecond),
	}
}

func mockTraceEvent() TraceEvent {
	var evts []Event
	evts = append(evts, mockEvent())
	evts = append(evts, mockEvent())
	evts = append(evts, mockEvent())

	return TraceEvent{
		TelemetryId: NewTraceId(),
		AdapterMeta: nil,
		Events:      evts,
	}
}

func TestBucket(t *testing.T) {
	flusher := &TestFlusher{}
	bucket := NewEventBucket(2, time.Duration(500*time.Millisecond))

	bucket.addEvent(mockTraceEvent(), flusher)
	bucket.addEvent(mockTraceEvent(), flusher)
	bucket.addEvent(mockTraceEvent(), flusher)

	time.Sleep(time.Duration(250 * time.Millisecond))

	if flusher.NumFlushes > 0 {
		t.Fatalf("After 0.25 seconds, no events should be flushed but there were %d flushes", flusher.NumFlushes)
	}

	time.Sleep(time.Duration(500 * time.Millisecond))

	if flusher.NumFlushes != 2 {
		t.Fatalf("After .75 seconds, there should have been exactly 2 flushes but there were %d flushes", flusher.NumFlushes)
	}

	evts1 := flusher.FlushedEvents[0]
	if len(evts1) != 2 {
		t.Fatalf("Expected 2 TraceEvents but got %d", len(evts1))
	}
	evts2 := flusher.FlushedEvents[1]
	if len(evts2) != 1 {
		t.Fatalf("Expected 1 but got %d", len(evts2))
	}

	if len(bucket.bucket) != 0 {
		t.Fatalf("Expected the event bucket to be empty but there were %d traceevents", len(bucket.bucket))
	}
}
