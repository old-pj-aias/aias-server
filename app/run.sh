#!/bin/sh

cd ./aias-core/go && cargo build && cd /go/app
cp ./aias-core/go/target/debug/libaias_go.so ./aias-core/go/

# We must generate executable manually to link Rust lib. (Not `go run`.)
go build -ldflags="-r ./aias-core/go" src/main.go
./main