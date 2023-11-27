package observe

import (
	"errors"
	"log"

	"github.com/tetratelabs/wabin/binary"
	"github.com/tetratelabs/wabin/wasm"
)

// Parse the names of the functions out of the
// names custom section in the wasm binary.
func parseNames(data []byte) (map[uint32]string, error) {
	features := wasm.CoreFeaturesV2
	m, err := binary.DecodeModule(data, features)
	if err != nil {
		return nil, err
	}

	if m.NameSection == nil {
		return nil, errors.New("Name section not found")
	}

	names := make(map[uint32]string, len(m.NameSection.FunctionNames))

	for _, v := range m.NameSection.FunctionNames {
		names[v.Index] = v.Name
	}

	warnOnDylibsoObserve := true
	for _, item := range m.ImportSection {
		if item.Module == "dylibso_observe" {
			if warnOnDylibsoObserve {
				warnOnDylibsoObserve = false
				log.Println("Module uses deprecated namespace \"dylibso_observe\"!\n" +
					"Please consider reinstrumenting with newer wasm-instr!")
			}
			switch item.Name {
			case "span_enter", "span_tags", "metric", "log", "span_exit":
				return nil, errors.New("go sdk does not yet support Observe API")
			}
		} else if item.Module == "dylibso:observe/api" {
			return nil, errors.New("go sdk does not yet support Observe API")
		}
	}

	return names, nil
}
