# ADR-001: OpenAPI Specification Generation Approach

## Status

Accepted (2026-06-26)

## Context

Phase 4 of Sprint 2 requires a comprehensive OpenAPI 3.0 specification for the Noesis REST API (28+ endpoints). The options were:

1. **utoipa** — Annotate handlers with proc macros for automatic spec generation
2. **Static file** — Hand-author an `openapi.json` file alongside the code
3. **Programmatic generation** — Build the spec at runtime using `serde_json::json!()`

## Decision

We chose **programmatic generation** (`src/docs/mod.rs`):

The spec is built from `serde_json::json!()` invocations inside a `generate_openapi_spec()` function that returns a `serde_json::Value`. This function is called once per server start and served at `/api/docs/openapi.json`.

## Rationale

- **No new dependencies** — utoipa requires proc macros and extra crates; programmatic generation uses serde_json which is already present
- **Maintainable** — the spec lives in one file with clear structure; updating an endpoint's description is a local edit
- **Comprehensive** — we document all response shapes, query parameters, request bodies, and error conditions
- **Self-describing** — the spec is itself an endpoint; Swagger UI loads it client-side via CDN

## Consequences

- The spec must be manually kept in sync with handler changes
- The `json!()` macro hit the default recursion limit (128) — raised to 256 via `#![recursion_limit = "256"]` in `lib.rs`
- CDN dependency for Swagger UI means the docs page needs internet access
