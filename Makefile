.PHONY: help build release run check test clean install

help:
	@echo "Surge TUI - Makefile commands"
	@echo ""
	@echo "make build    - debug build (en-us, default)"
	@echo "make release  - release build (en-us, default)"
	@echo "make run      - run the program (en-us, default)"
	@echo "make check    - check code (both languages)"
	@echo "make test     - run tests"
	@echo "make clean    - clean build artifacts"
	@echo "make install  - install to system (en-us, default)"
	@echo ""
	@echo "Chinese build: cargo build --features zh-cn"

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

check:
	cargo check
	cargo check --features zh-cn
	cargo clippy

test:
	cargo test

clean:
	cargo clean

install:
	cargo install --path .
