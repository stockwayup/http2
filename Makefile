VERSION := $(shell cat VERSION)

.PHONY: help fmt lint test test-verbose check build push increment-version get-version

help:
	@echo "Available commands:"
	@echo "  fmt              - Format code with cargo fmt"
	@echo "  lint             - Run clippy with auto-fixes"
	@echo "  test             - Run all tests"
	@echo "  test-verbose     - Run tests with output visible"
	@echo "  check            - Type check without building"
	@echo "  docker_build     - Build Docker image with current version"
	@echo "  build            - Auto-increment version, build and push Docker image"
	@echo "  push             - Push Docker image to registry"
	@echo "  increment-version - Increment minor version in VERSION file"
	@echo "  get-version      - Display current version"

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

docker_build:
	docker build . -t soulgarden/swup:http2-$(VERSION) -t soulgarden/swup:http2-latest --platform linux/amd64

build: increment-version
	@echo "Building with version: $$(cat VERSION)"
	docker build . -t soulgarden/swup:http2-$$(cat VERSION) -t soulgarden/swup:http2-latest --platform linux/amd64
	docker push soulgarden/swup:http2-$$(cat VERSION)
	docker push soulgarden/swup:http2-latest

push:
	docker push soulgarden/swup:http2-$(VERSION)
	docker push soulgarden/swup:http2-latest

increment-version:
	@current_version=$$(cat VERSION); \
	major=$$(echo $$current_version | cut -d. -f1); \
	minor=$$(echo $$current_version | cut -d. -f2); \
	patch=$$(echo $$current_version | cut -d. -f3); \
	new_minor=$$((minor + 1)); \
	new_version="$$major.$$new_minor.$$patch"; \
	echo "Incrementing version from $$current_version to $$new_version"; \
	echo $$new_version > VERSION

get-version:
	@echo $(VERSION)