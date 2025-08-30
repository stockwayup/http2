# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based HTTP/2 microservice that acts as an asynchronous request gateway for a financial platform. The service receives HTTP requests, forwards them to a backend via NATS messaging, and returns responses. It's the second version, rewritten from Go to Rust for improved performance.

## Core Architecture

**Request Flow:**
1. Axum web server receives HTTP requests on port 8000
2. Requests are serialized using MessagePack (rmp-serde) and sent to NATS subject "http"
3. Backend services process requests and send responses via NATS headers
4. Service returns JSON API-compliant responses to the client

**Key Components:**
- `main.rs` - Application entry point, sets up NATS client and Axum server
- `routes.rs` - Defines all API endpoints (financial platform routes like portfolios, securities, etc.)
- `handlers.rs` - Contains `proxy()` function that forwards all requests to NATS and `health_check()`
- `events.rs` - Defines `HttpReq` struct for request serialization
- `conf.rs` - Configuration management from `config.json` or `CFG_PATH`
- `responses/` - Error handling and status response structures

**Messaging:**
- Uses NATS (async-nats crate) instead of traditional HTTP proxying
- All requests except health checks go through the `proxy()` handler
- Request/response correlation using UUID in NATS headers
- 10-second request timeout with ping interval for connection health

## Development Commands

```bash
# Code quality
make fmt          # Format code with cargo fmt
make lint         # Run clippy with auto-fixes
cargo check       # Type check without building

# Testing
make test         # Run all tests (unit + integration)
make test-verbose # Run tests with output visible
cargo test        # Direct cargo test command

# Building
cargo build       # Development build
cargo build --release  # Production build

# Docker deployment
make build        # Build and push Docker image (soulgarden/swup:http2-0.1.3)

# Running locally
cargo run         # Requires NATS at nats.backend.orb.local:4222

# Help
make help         # Show all available make commands
```

## Configuration

The service requires:
- `config.json` in project root (or path from `CFG_PATH` env var)
- NATS server running at configured host
- Required config fields: `listen_port`, `nats.host`, `allowed_origins`, `is_debug`

## Testing

The project includes comprehensive test coverage:

**Unit Tests (17 tests):**
- Configuration parsing and validation (`conf.rs`)
- HTTP request serialization (`events.rs`)
- Response structure formatting (`responses/`)

**Integration Tests (17 tests):**
- HTTP handlers with mocked dependencies
- Router configuration and CORS setup
- End-to-end request/response cycles

Tests are designed to run without requiring external dependencies (NATS server).
Integration tests automatically skip when NATS is unavailable.

## Financial Platform API

The service proxies a comprehensive financial API with endpoints for:
- User management and authentication
- Portfolio management with securities and transactions
- Market data (prices, earnings, dividends, news)
- Financial statements (balance sheets, income statements)
- Reference data (countries, currencies, sectors, exchanges)

All API routes are prefixed with `/api/v1` and follow REST conventions with proper HTTP methods.
