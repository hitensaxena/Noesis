# Noesis

**A decentralized cognitive architecture — emergent intelligence through recursive signal propagation.**

Most AI systems are built as collections of modules: Memory, Reflection, Identity, Goals, Knowledge Graph. These modules call each other through APIs in a mostly top-down architecture.

Noesis rejects that model.

Instead, it models cognition as an emergent decentralized network, inspired by biological neural systems, predictive processing, Global Workspace Theory, the Actor Model, and Advaita Vedanta.

There is no central "brain" controlling the system. No master cognition service. No module responsible for "thinking." Intelligence emerges from thousands of tiny computations interacting through signals — exactly as biological neural systems do.

---

## Architecture

Noesis is organized around three concepts:

**Fields** — persistent cognitive spaces that own state. Fields never call each other directly.

**Processors** — tiny autonomous workers (49 total), each performing exactly one cognitive transformation. Processors never invoke other processors directly.

**Signals** — the language of the organism. Everything communicates through signals. Signals propagate recursively until the network reaches equilibrium.

No processor knows the complete processing pipeline. No field knows what other fields exist. The final cognitive state is emergent.

### 8 Cognitive Fields

| Field | Role | Example Processors |
|-------|------|-------------------|
| **Memory** | Episodic & semantic storage | episode, extraction, consolidation, decay, dedup, indexing, retrieval |
| **Identity** | Self-model & beliefs | belief, identity, values, principles |
| **Agency** | Goals & priorities | goal, priority, strategy, opportunity |
| **Action** | Planning & execution | plan, project, task, execution, evaluation, risk, recovery |
| **Awareness** | Attention & reflection | attention, curiosity, narrative, reflection, observer, mood, health, pattern, open_loops |
| **Reasoning** | Metacognition & logic | metacognition, epistemic, reasoning, mental_model, decision, hypothesis, analogy, synthesis, concept |
| **Simulation** | What-if forecasting | world_model, assumption, scenario, counterfactual, forecast, risk |
| **Knowledge Graph** | Entity-relation store | entity, edge, triple extraction |

### Cascade Convergence

Signals propagate recursively: each processor subscribes to signal types, transforms them, and emits new signals. Activation decays per hop until no processor activates — the cascade terminates naturally. This guarantees convergence without infinite loops.

---

## Quick Start

### Prerequisites

- Rust 1.96+

### Run the daemon

```bash
cargo run --bin noesis -- start --rest --port 8647
```

Open the management dashboard at **http://127.0.0.1:8647/api/dashboard/**

### Inject a signal

```bash
curl -X POST http://127.0.0.1:8647/api/signals/inject \
  -H 'Content-Type: application/json' \
  -d '{"signal_type":"memory.capture.ingested","payload":{"text":"Hello from Noesis"}}'
```

### CLI commands

```bash
noesis start                           # Start the daemon
noesis start --rest --port 8647        # With REST API
noesis inject "I went for a run"       # Inject an experience
noesis list fields                     # List registered fields
noesis list signals                    # List all signal types
```

---

## Configuration

| Flag | Default | Description |
|------|---------|-------------|
| `--rest` | `false` | Enable REST API server |
| `--port` | `8647` | REST API port |
| `--mcp` | `false` | Enable MCP server (AI agent protocol) |
| `--mcp-port` | `8645` | MCP server port |
| `--storage` | `memory` | Storage backend (`memory` or `postgres`) |
| `--database-url` | auto-detect | Postgres connection URL |
| `--redis-url` | auto-detect | Redis connection URL |

Environment: `NOESIS_API_KEY` (enables auth), `RUST_LOG` (tracing level).

---

## Web Dashboard

Served at `/api/dashboard/` — 8 views:

- **Overview** — System health, field status cards, signal rate
- **Fields** — Per-field state inspector with raw JSON
- **Signals** — Real-time SSE signal stream with type filtering
- **Inject** — Signal injection console with quick-inject buttons
- **Processors** — Searchable processor registry (49 processors)
- **Plugins** — Plugin manager with hot-reload
- **Events** — Browsable event history with pagination
- **Metrics** — Signal distribution, processor latency, field sizes

---

## REST API

32 endpoints at `/api/*`. Full OpenAPI spec at `GET /api/docs/openapi.json`.

| Area | Key Endpoints |
|------|--------------|
| Health | `GET /api/health` |
| Ingest | `POST /api/ingest` |
| Signals | `GET /api/signals`, `POST /api/signals/inject`, `GET /api/signals/history?field=memory` |
| Observability | `GET /api/observability/overview`, `/signals`, `/processors`, `/cascade` |
| Plugins | `GET /api/plugins`, `GET /api/plugins/{name}`, `POST /api/plugins/reload` |
| Config | `GET /api/config` |
| Docs | `GET /api/docs/` (Swagger UI), `GET /api/docs/openapi.json` |
| Dashboard | `GET /api/dashboard/` |
| Events | `GET /api/events/stream` (SSE real-time) |

---

## Production

### Docker

```bash
# Build and run (in-memory storage)
docker compose up

# With Postgres persistence
docker compose --profile with-pg up

# Full stack: Noesis + Postgres + Redis
docker compose --profile all up
```

### Systemd

```bash
sudo cp noesis.service /etc/systemd/system/
sudo systemctl enable noesis
sudo systemctl start noesis
```

---

## Testing

```bash
cargo test                    # 342 tests
cargo test --test e2e_test    # E2E cascade stress test
```

Coverage: 49 processor unit tests, 5 field integration test files, E2E cascade, Postgres/Redis storage backends, plugin manifest system, REST API + auth middleware, SSE event stream.

---

## Guiding Question

Every architectural decision answers: *Does this make the organism more capable of evolving through decentralized recursive cognition, or does it reintroduce centralized software architecture?*

---

## Architecture Documents

| Document | Content |
|----------|---------|
| `NOESISV3.md` | Sprint 3 plan (architecture cleanup, UI, testing, production) |
| `PROGRESS.md` | Implementation status and metrics |

## License

MIT
