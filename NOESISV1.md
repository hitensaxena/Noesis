# Noesis v1 — Cognitive Operating System

Noesis is a **persistent cognitive organism** whose intelligence emerges from decentralized recursive computation over persistent cognitive state. It is not an AI assistant, agent framework, memory system, or workflow engine — it is all of those things unified.

## Architecture

```
Kernel                    (sustains — pure infrastructure)
  │
Field Runtime             (bridges — dispatches signals, manages processors)
  │
┌────────┬───────┬───────┬───────┬──────────┬──────────┬──────────┐
│Memory  │Identity│Agency │Action │Awareness │Reasoning │Simulation│
│What do │Who am │What do│What am│What am I │What do I │What could│
│I rem.? │ I?    │I want?│doing? │noticing? │conclude? │ happen?  │
└────────┴───────┴───────┴───────┴──────────┴──────────┴──────────┘
```

### Immutable Principles

- **Fields remember.** Fields own persistent cognitive state. Each answers exactly one question.
- **Processors transform.** They subscribe to signals, transform them, and emit new signals. One processor = one transformation.
- **Signals propagate.** Recursive activation with energy decay guarantees convergence.
- **Kernel sustains.** The kernel never performs cognition — it provides runtime, scheduling, routing, and metrics.

## Quick Start

```bash
cargo run -- start                    # Start daemon + REST on :8647
cargo run -- start --rest --mcp       # Start with REST + MCP (:8645)
cargo run -- inject "text" "source"   # Inject experience → cascade
cargo run -- list fields              # List 8 registered fields
cargo run -- list processors          # List 49 registered processors
```

## API

### REST (:8647)
```
GET  /api/health                    — System health
POST /api/ingest                    — Inject raw text
POST /api/memories                  — Create memory
GET  /api/memories                  — List memories
GET  /api/memory/recall?q=...       — Search episodes
GET  /api/episodes                  — List episodes
GET  /api/graph                     — Knowledge graph
GET  /api/identity                  — Identity state
GET  /api/capabilities              — Registered capabilities
GET  /api/observability/overview    — System overview
...
```

### MCP (:8645, JSON-RPC 2.0)
```
POST /mcp  { method: "tools/list" }
POST /mcp  { method: "tools/call", params: { name: "recall", arguments: { query: "..." } } }
POST /mcp  { method: "tools/call", params: { name: "inject", arguments: { text: "..." } } }
POST /mcp  { method: "tools/call", params: { name: "field_state", arguments: { field: "memory" } } }
POST /mcp  { method: "tools/call", params: { name: "capabilities" } }
```

### Auth
Set `NOESIS_API_KEY` env var to require `Authorization: Bearer <key>` on all REST requests.

## System

- **49 processors** across 8 fields, 52 signal types
- **47 cognitive transformations** — each one independently testable
- **LLM tiered routing** — Fast (Haiku), Agentic (Sonnet), Deep (Opus)
- **Postgres/Redis** backends for persistence (fallback to in-memory)
- **Signal cascade** — recursive activation with automatic convergence via energy decay
- **Cognitive beats** — fast (1s), medium (60s), slow (15min) scheduling

## Stack

- **Language:** Rust (edition 2021)
- **Runtime:** Tokio (async), Axum (HTTP), Ratatui (TUI)
- **Storage:** In-memory with optional Postgres + Redis
- **LLM:** Tiered routing via OpenRouter-compatible APIs
- **Source:** ~15,000 lines, 178 files, zero warnings
