#!/bin/sh

# We must generate executable manually to link Rust lib. (Not `go run`.)
go build -ldflags="-r ./core" src/main.go
./main