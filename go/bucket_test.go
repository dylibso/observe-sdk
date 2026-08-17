package observe

import (
	"sync"
	"testing"
	"time"
)

// TestFlusher is read by the test goroutine and written by the bucket's own
// flush goroutine, so its fields need a lock rather than relying on
// time.Sleep to order the two - sleeping long enough happens to make the
// flush observable in practice, but it isn't a happens-before relationship,
// so the race detector correctly flags direct field access as racy.
type TestFlusher struct {
	mu            sync.Mutex
	NumFlushes    int
	FlushedEvents [][]TraceEvent
}

func (f *TestFlusher) Flush(events []TraceEvent) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.NumFlushes += 1
	f.FlushedEvents = append(f.FlushedEvents, events)
	return nil
}

func (f *TestFlusher) numFlushes() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.NumFlushes
}

func (f *TestFlusher) flushedEvents() [][]TraceEvent {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.FlushedEvents
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

	if n := flusher.numFlushes(); n > 0 {
		t.Fatalf("After 0.25 seconds, no events should be flushed but there were %d flushes", n)
	}

	time.Sleep(time.Duration(500 * time.Millisecond))

	if n := flusher.numFlushes(); n != 2 {
		t.Fatalf("After .75 seconds, there should have been exactly 2 flushes but there were %d flushes", n)
	}

	flushedEvents := flusher.flushedEvents()
	evts1 := flushedEvents[0]
	if len(evts1) != 2 {
		t.Fatalf("Expected 2 TraceEvents but got %d", len(evts1))
	}
	evts2 := flushedEvents[1]
	if len(evts2) != 1 {
		t.Fatalf("Expected 1 but got %d", len(evts2))
	}

	bucket.mu.Lock()
	remaining := len(bucket.bucket)
	bucket.mu.Unlock()
	if remaining != 0 {
		t.Fatalf("Expected the event bucket to be empty but there were %d traceevents", remaining)
	}
}

// TestBucketWaitBlocksUntilFlush proves that Wait() cannot return before a
// flush scheduled by a concurrent addEvent() has actually run: Add(1) must
// happen-before Wait() observes the counter, not from inside the goroutine
// it is meant to guard. With that ordering wrong, Wait() can return while
// the counter is still zero and the flush is still in flight - exactly the
// pattern that produced a real data race in downstream consumers reading a
// buffer the flush goroutine was still writing to (github.com/extism/go-sdk's
// TestObserve).
func TestBucketWaitBlocksUntilFlush(t *testing.T) {
	flusher := &TestFlusher{}
	bucket := NewEventBucket(10, 50*time.Millisecond)

	bucket.addEvent(mockTraceEvent(), flusher)
	bucket.Wait()

	if n := flusher.numFlushes(); n != 1 {
		t.Fatalf("expected the flush to have completed by the time Wait() returned, got %d flushes", n)
	}
}
