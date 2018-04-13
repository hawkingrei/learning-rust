ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

build:
	cd gif/rgif && cargo build --release
	cp gif/rgif/target/release/librgif.dylib lib
	go build -ldflags="-r $(ROOT_DIR)lib" main.go

run: build
	./main
