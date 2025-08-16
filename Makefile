.PHONY: help fmt lint test test-verbose check build

help:
	@echo "Available commands:"
	@echo "  fmt          - Format code with cargo fmt"
	@echo "  lint         - Run clippy with auto-fixes"
	@echo "  test         - Run all tests"
	@echo "  test-verbose - Run tests with output visible"
	@echo "  check        - Type check without building"
	@echo "  build        - Build and push Docker image"

fmt:
	cargo fmt --all

lint:
	cargo clippy --fix --allow-dirty --allow-staged

test:
	cargo test

test-verbose:
	cargo test -- --nocapture

check:
	cargo check

build:
	docker build . -t soulgarden/swup:http2-0.1.1 --platform linux/amd64
	docker push soulgarden/swup:http2-0.1.1
