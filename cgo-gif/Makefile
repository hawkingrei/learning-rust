ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

build:
	cd gif/rgif && cargo build 
	cp gif/rgif/target/debug/librgif.dylib lib
	go build  -gcflags=all="-N -l" -ldflags="-r $(ROOT_DIR)lib" main.go

run: build
	./main
