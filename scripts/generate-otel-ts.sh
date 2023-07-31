#!/usr/bin/env bash

# npm i -g ts-proto
# requires protoc installed as well (brew install protobuf, apt install protobuf-compiler)

# from project root:
protoc --plugin=ts_proto proto/opentelemetry/proto/trace/v1/trace.proto -I proto --ts_proto_out=./js/proto
