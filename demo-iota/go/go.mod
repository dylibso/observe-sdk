module github.com/dylibso/observe-sdk/demo-iota/go

go 1.19

require (
	github.com/dylibso/observe-sdk/go v0.0.0-20230817180337-d820d9baa3bb
	github.com/tetratelabs/wazero v1.4.0
)

// TODO: get this working in Docker
// replace github.com/dylibso/observe-sdk/go => ../../go

require (
	github.com/ianlancetaylor/demangle v0.0.0-20230524184225-eabc099b10ab // indirect
	github.com/tetratelabs/wabin v0.0.0-20230304001439-f6f874872834 // indirect
)
