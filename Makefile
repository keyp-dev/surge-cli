.PHONY: help build release run check test clean install

help:
	@echo "Surge TUI - Makefile 命令"
	@echo ""
	@echo "make build    - 调试构建"
	@echo "make release  - 发布构建"
	@echo "make run      - 运行程序"
	@echo "make check    - 检查代码"
	@echo "make test     - 运行测试"
	@echo "make clean    - 清理构建文件"
	@echo "make install  - 安装到系统"

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

check:
	cargo check
	cargo clippy

test:
	cargo test

clean:
	cargo clean

install:
	cargo install --path .
