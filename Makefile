.PHONY: all build-wasm test-rs bench-rs test-js test-all clean help

# Default target
all: help

# Build WASM for Node.js
build-wasm:
	wasm-pack build --target nodejs --release
	cp README.md pkg/README.md
	@echo "Build complete. You can now publish from the pkg directory:"
	@echo "cd pkg && npm publish"

# Run Rust tests
test-rs:
	@if [ -z "$(CZDB_SECRET)" ]; then echo "Error: CZDB_SECRET is not set"; exit 1; fi
	cargo test

# Run Rust benchmarks
bench-rs:
	@if [ -z "$(CZDB_SECRET)" ]; then echo "Error: CZDB_SECRET is not set"; exit 1; fi
	cargo test --test bench_rust -- --nocapture

# Run JS/WASM benchmarks
test-js: build-wasm
	@if [ -z "$(CZDB_SECRET)" ]; then echo "Error: CZDB_SECRET is not set"; exit 1; fi
	cd tests/npm-test && bun install
	bun run tests/npm-test/bench.ts

# Run all tests
test-all: test-rs bench-rs test-js

# Clean build artifacts
clean:
	cargo clean
	rm -rf pkg
	rm -rf tests/output/*

# Show help
help:
	@echo "Available targets:"
	@echo "  build-wasm : Build WASM package for Node.js"
	@echo "  test-rs    : Run Rust tests (requires CZDB_SECRET)"
	@echo "  bench-rs   : Run Rust benchmarks (requires CZDB_SECRET)"
	@echo "  test-js    : Run JS/WASM benchmarks (requires CZDB_SECRET)"
	@echo "  test-all   : Run all tests"
	@echo "  clean      : Clean build artifacts"
