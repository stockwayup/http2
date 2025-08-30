# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Rust sources — key modules: `main.rs` (binary), `lib.rs` (exports), `routes.rs` (HTTP routing), `handlers.rs` (request/response), `conf.rs` (JSON config), `metrics.rs` and `observability.rs` (Prometheus + tracing).
- `tests/`: Integration and API tests (`*_test.rs`).
- `config.json`: Default runtime config; override path via `CFG_PATH` env.
- `Makefile`, `Dockerfile`, `Cargo.toml`: tooling and build metadata.

## Build, Test, and Development Commands
- `cargo run` — start locally using `config.json` (or `CFG_PATH=./path/config.json cargo run`).
- `make fmt` — format with rustfmt.
- `make lint` — run Clippy with autofix.
- `make test` / `make test-verbose` — run unit/integration tests.
- `make check` — type-check (fast feedback).
- `make build` — Docker build and push (updates tag in Makefile).

## Coding Style & Naming Conventions
- Rust 2021; 4-space indentation; no unsafe (`#![forbid(unsafe_code)]`) and no warnings (`#![deny(warnings)]`).
- Use idiomatic Rust naming: modules/paths `snake_case`, types `PascalCase`, functions `snake_case`.
- Responses must set `content-type: application/vnd.api+json` (see `handlers.rs`).
- Keep route templates under `routes.rs` (`/api/v1/...`); prefer constants for shared prefixes.

## Testing Guidelines
- Frameworks: `tokio` async tests, Axum + `tower::ServiceExt` for routing.
- Locations: unit tests inline (e.g., `src/conf.rs`), integration tests in `tests/`.
- Names: use `*_test.rs` and descriptive `#[tokio::test]` functions.
- Run: `cargo test` (some end-to-end paths expect a NATS server; tests are written to skip gracefully when unavailable).

## Commit & Pull Request Guidelines
- Commits: concise, imperative subject (e.g., "Add request timing logging"); include scope when helpful.
- PRs must include: summary of changes, rationale, testing notes (commands used), and any config or API impacts. Link related issues.
- CI expectations: pass `make fmt lint test check` before requesting review.

## Security & Configuration Tips
- Configure via JSON: example `config.json` keys: `listen_port`, `nats.host`, `allowed_origins`, `is_debug`. Use `CFG_PATH` to point to env-specific files.
- Limit CORS origins to required hosts only (see `routes.rs`).
- Body size is limited to 250KB by middleware; avoid increasing without justification.
- Docker image runs as non-root (`www-data`). Avoid committing secrets; use secure secret stores.

