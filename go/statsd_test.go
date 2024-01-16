package observe

import (
	"testing"
)

func TestParseStatsdDataGram_Simple(t *testing.T) {
	message := "my-counter:1.98|c"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-counter" {
		t.Errorf("expected name to be 'my-counter', got %s", datagram.Name)
	} else if datagram.Type != StatsdCounter {
		t.Errorf("expected type to be StatsdCounter, got %d", datagram.Type)
	} else if datagram.Value != 1.98 {
		t.Errorf("expected value to be 1.98, got %f", datagram.Value)
	} else if datagram.HasSampleRate == true {
		t.Errorf("expected HasSampleRate to be false, got %v", datagram.HasSampleRate)
	} else if len(datagram.Tags) != 0 {
		t.Errorf("expected 0 tags, got %d", len(datagram.Tags))
	}
}

func TestParseStatsdDataGram_Counter_NoTags(t *testing.T) {
	message := "my-counter:1.98|c|@0.1"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-counter" {
		t.Errorf("expected name to be 'my-counter', got %s", datagram.Name)
	} else if datagram.Type != StatsdCounter {
		t.Errorf("expected type to be StatsdCounter, got %d", datagram.Type)
	} else if datagram.Value != 1.98 {
		t.Errorf("expected value to be 1.98, got %f", datagram.Value)
	} else if datagram.HasSampleRate == false {
		t.Errorf("expected HasSampleRate to be true, got %v", datagram.HasSampleRate)
	} else if datagram.SampleRate != 0.1 {
		t.Errorf("expected sample rate to be 0.1, got %f", datagram.SampleRate)
	} else if len(datagram.Tags) != 0 {
		t.Errorf("expected 0 tags, got %d", len(datagram.Tags))
	}
}
func TestParseStatsdDataGram_Counter_NoSampleRate(t *testing.T) {
	message := "my-counter:1.98|c|#tag1:value1,tag2:value2"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-counter" {
		t.Errorf("expected name to be 'my-counter', got %s", datagram.Name)
	} else if datagram.Type != StatsdCounter {
		t.Errorf("expected type to be StatsdCounter, got %d", datagram.Type)
	} else if datagram.Value != 1.98 {
		t.Errorf("expected value to be 1.98, got %f", datagram.Value)
	} else if datagram.HasSampleRate == true {
		t.Errorf("expected HasSampleRate to be false, got %v", datagram.HasSampleRate)
	} else if len(datagram.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(datagram.Tags))
	} else if datagram.Tags["tag1"] != "value1" {
		t.Errorf("expected tag1 to be 'value1', got %s", datagram.Tags["tag1"])
	} else if datagram.Tags["tag2"] != "value2" {
		t.Errorf("expected tag2 to be 'value2', got %s", datagram.Tags["tag2"])
	}
}

func TestParseStatsdDataGram_Counter(t *testing.T) {
	message := "my-counter:1.98|c|@0.1|#tag1:value1,tag2:value2"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-counter" {
		t.Errorf("expected name to be 'my-counter', got %s", datagram.Name)
	} else if datagram.Type != StatsdCounter {
		t.Errorf("expected type to be StatsdCounter, got %d", datagram.Type)
	} else if datagram.Value != 1.98 {
		t.Errorf("expected value to be 1.98, got %f", datagram.Value)
	} else if datagram.SampleRate != 0.1 {
		t.Errorf("expected sample rate to be 0.1, got %f", datagram.SampleRate)
	} else if len(datagram.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(datagram.Tags))
	} else if datagram.Tags["tag1"] != "value1" {
		t.Errorf("expected tag1 to be 'value1', got %s", datagram.Tags["tag1"])
	} else if datagram.Tags["tag2"] != "value2" {
		t.Errorf("expected tag2 to be 'value2', got %s", datagram.Tags["tag2"])
	}
}

func TestParseStatsdDataGram_Gauge(t *testing.T) {
	message := "my-guage:8723.042|g|@0.001|#tag1:value1,tag2:value2"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-guage" {
		t.Errorf("expected name to be 'my-guage', got %s", datagram.Name)
	} else if datagram.Type != StatsdGauge {
		t.Errorf("expected type to be StatsdGauge, got %d", datagram.Type)
	} else if datagram.Value != 8723.042 {
		t.Errorf("expected value to be 8723.042, got %f", datagram.Value)
	} else if datagram.SampleRate != 0.001 {
		t.Errorf("expected sample rate to be 0.001, got %f", datagram.SampleRate)
	} else if len(datagram.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(datagram.Tags))
	} else if datagram.Tags["tag1"] != "value1" {
		t.Errorf("expected tag1 to be 'value1', got %s", datagram.Tags["tag1"])
	} else if datagram.Tags["tag2"] != "value2" {
		t.Errorf("expected tag2 to be 'value2', got %s", datagram.Tags["tag2"])
	}
}

func TestParseStatsdDataGram_Timing(t *testing.T) {
	message := "my-timing:1|ms|@0.1|#tag1:value1,tag2:value2"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-timing" {
		t.Errorf("expected name to be 'my-timing', got %s", datagram.Name)
	} else if datagram.Type != StatsdTiming {
		t.Errorf("expected type to be StatsdTiming, got %d", datagram.Type)
	} else if datagram.Value != 1 {
		t.Errorf("expected value to be 1, got %f", datagram.Value)
	} else if datagram.SampleRate != 0.1 {
		t.Errorf("expected sample rate to be 0.1, got %f", datagram.SampleRate)
	} else if len(datagram.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(datagram.Tags))
	} else if datagram.Tags["tag1"] != "value1" {
		t.Errorf("expected tag1 to be 'value1', got %s", datagram.Tags["tag1"])
	} else if datagram.Tags["tag2"] != "value2" {
		t.Errorf("expected tag2 to be 'value2', got %s", datagram.Tags["tag2"])
	}
}

func TestParseStatsdDataGram_Histogram(t *testing.T) {
	message := "my-histogram:1|h|@0.1|#tag1:value1,tag2:value2"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-histogram" {
		t.Errorf("expected name to be 'my-histogram', got %s", datagram.Name)
	} else if datagram.Type != StatsdHistogram {
		t.Errorf("expected type to be StatsdHistogram, got %d", datagram.Type)
	} else if datagram.Value != 1 {
		t.Errorf("expected value to be 1, got %f", datagram.Value)
	} else if datagram.SampleRate != 0.1 {
		t.Errorf("expected sample rate to be 0.1, got %f", datagram.SampleRate)
	} else if len(datagram.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(datagram.Tags))
	} else if datagram.Tags["tag1"] != "value1" {
		t.Errorf("expected tag1 to be 'value1', got %s", datagram.Tags["tag1"])
	} else if datagram.Tags["tag2"] != "value2" {
		t.Errorf("expected tag2 to be 'value2', got %s", datagram.Tags["tag2"])
	}
}

func TestParseStatsdDataGram_Set(t *testing.T) {
	message := "my-set:1|s|@0.1|#tag1:value1,tag2:value2"
	datagram, err := parseStatsdDataGram(message)
	if err != nil {
		t.Errorf("parseStatsdDataGram() error = %v", err)
	}

	if datagram.Name != "my-set" {
		t.Errorf("expected name to be 'my-set', got %s", datagram.Name)
	} else if datagram.Type != StatsdSet {
		t.Errorf("expected type to be StatsdSet, got %d", datagram.Type)
	} else if datagram.Value != 1 {
		t.Errorf("expected value to be 1, got %f", datagram.Value)
	} else if datagram.SampleRate != 0.1 {
		t.Errorf("expected sample rate to be 0.1, got %f", datagram.SampleRate)
	} else if len(datagram.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(datagram.Tags))
	} else if datagram.Tags["tag1"] != "value1" {
		t.Errorf("expected tag1 to be 'value1', got %s", datagram.Tags["tag1"])
	} else if datagram.Tags["tag2"] != "value2" {
		t.Errorf("expected tag2 to be 'value2', got %s", datagram.Tags["tag2"])
	}
}

func TestParseStatsdDataGram_InvalidType(t *testing.T) {
	message := "invalid:1|x"
	_, err := parseStatsdDataGram(message)
	if err == nil {
		t.Errorf("parseStatsdDataGram() expected error, got nil")
	}
}

func TestParseStatsdDataGram_InvalidFormat(t *testing.T) {
	message := "invalid"
	_, err := parseStatsdDataGram(message)
	if err == nil {
		t.Errorf("parseStatsdDataGram() expected error, got nil")
	}
}
