package observe

import (
	"errors"

	"github.com/tetratelabs/wabin/binary"
	"github.com/tetratelabs/wabin/wasm"
)

// Parse the names of the functions out of the
// names custom section in the wasm binary.
func parseNames(data []byte) (map[uint32]string, error, bool) {
	isOldNamespace := false
	features := wasm.CoreFeaturesV2
	m, err := binary.DecodeModule(data, features)
	if err != nil {
		return nil, err, isOldNamespace
	}

	if m.NameSection == nil {
		return nil, errors.New("Name section not found"), isOldNamespace
	}

	names := make(map[uint32]string, len(m.NameSection.FunctionNames))

	for _, v := range m.NameSection.FunctionNames {
		names[v.Index] = v.Name
	}

	for _, item := range m.ImportSection {
		if item.Module == "dylibso_observe" {
			isOldNamespace = true
			break
		}
	}

	return names, nil, isOldNamespace
}
