package observe

import (
	"log"
	"sync"
	"time"
)

type Flusher interface {
	Flush(events []TraceEvent) error
}

type EventBucket struct {
	mu          sync.Mutex
	wg          sync.WaitGroup
	bucket      []TraceEvent
	flushPeriod time.Duration
	batchSize   int
}

func NewEventBucket(batchSize int, flushPeriod time.Duration) *EventBucket {
	return &EventBucket{
		flushPeriod: flushPeriod,
		batchSize:   batchSize,
	}
}

func (b *EventBucket) addEvent(e TraceEvent, f Flusher) {
	b.mu.Lock()
	wasEmpty := len(b.bucket) == 0
	b.bucket = append(b.bucket, e)
	b.mu.Unlock()
	// if this is the first event in the bucket,
	// we schedule a flush
	if wasEmpty {
		b.scheduleFlush(f)
	}
}

// Wait will block until all pending flushes are done
func (b *EventBucket) Wait() {
	b.wg.Wait()
}

// we start this routine and immediately wait, we are effectively
// scheduling the flush to run flushPeriod sections later. In the meantime,
// events may still be coming into the eventBucket
func (b *EventBucket) scheduleFlush(f Flusher) {
	go func() {
		defer b.wg.Done()
		b.wg.Add(1)

		time.Sleep(b.flushPeriod)

		// move the events out of the EventBucket to a slice
		// and add 1 to the waitgroup
		b.mu.Lock()
		bucket := b.bucket
		b.bucket = nil
		b.mu.Unlock()

		for i := 0; i < len(bucket); i += b.batchSize {
			j := i + b.batchSize
			if j > len(bucket) {
				j = len(bucket)
			}
			// TODO retry logic?
			err := f.Flush(bucket[i:j])
			if err != nil {
				log.Fatal(err)
			}
		}

	}()
}
