ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

build:
	cd gif/rgif && cargo build 
	cp gif/rgif/target/release/librgif.so lib
	go build -ldflags="-w -r $(ROOT_DIR)lib" main.go

run: build
	./main
