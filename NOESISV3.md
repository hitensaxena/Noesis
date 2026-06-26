# Noesis Sprint 3 — Finalize, Integrate, Operate

> **Sprint 3:** 2026-06-26 → (target: 1-2 weeks)  
> **Theme:** System completeness, production readiness, operational UI  
> **Build:** ✅ 286 tests, 0 warnings, 49/49 processors tested  

---

## Why Sprint 3

Sprint 1 built the architecture skeleton (signal cascade, field runtime, beats, plugins, REST/MCP). Sprint 2 filled in the cognitive substance (49 real processors, comprehensive tests, hardening, documentation/tooling).

**What Sprint 2 left behind — gaps against the architecture spec:**

1. **Inline cascade loop** — `main.rs` has its own cascade loop that duplicates `FieldRuntime`'s built-in dispatch. The architecture says the kernel should delegate entirely to `field_runtime.process_signal()`.
2. **3 integration test files instead of 8** — The architecture plan (NOESISV2 §2B) called for one test file per field. Only 3 exist (cascade, field_integration, interface).
3. **Minimal web dashboard** — 295 lines of inline Rust HTML/JS. Functional for inspection, but the user needs "a good web UI for managing everything" — field inspection, signal injection, processor management, plugin management, event browsing, configuration.
4. **No production packaging** — No Dockerfile, no docker-compose, no systemd service. Running in production requires manual `cargo run`.
5. **Static plugin loading only** — Plugin manifests are discoverable at startup but no hot-reload mechanism exists.
6. **Storage backends untested** — Postgres and Redis backends exist in `src/storage/backends/` but are not wired into the main data path or tested.
7. **No graceful shutdown integration** — `CancellationToken` is created but the shutdown sequence needs verification and hardening.
8. **No field filtering** — List endpoints (`/api/episodes`, `/api/memories`) don't support `?field=memory` style filtering.

Sprint 3 closes these gaps and delivers a **production-viable system** with a **comprehensive web UI**.

---

## Sprint 3 Phases

```
Phase 1: Architecture cleanup     ← main.rs rewrite, field filtering, graceful shutdown
Phase 2: Web UI overhaul          ← proper dashboard for full system management
Phase 3: Testing across processes ← field integration tests, end-to-end cascade, storage tests
Phase 4: Production readiness     ← Docker packaging, plugin hot-reload, backends
```

---

## Phase 1 — Architecture Cleanup

### 1A — Rewrite main.rs cascade topology

**Current state:** `main.rs` has a 200-line recursive cascade loop (`while let Some(signal) = cascade_queue.pop_front()`) that reimplements signal dispatch. This duplicates `FieldRuntime`'s built-in `process_signal()` method.

**Target:** `main.rs` delegates signal dispatch to `FieldRuntime::process_signal()`. The main loop only:
1. Receives external signals (REST, MCP, beats)
2. Pushes them into FieldRuntime
3. Collects emitted signals from FieldRuntime
4. Loops until equilibrium

**Files:** `src/main.rs`, `src/field_runtime/runtime.rs`, `src/field_runtime/dispatcher.rs`

**Tests:** Verify no behavioral regression — all 286 tests pass with rewritten cascade.

**Effort:** 2-3h

---

### 1B — Field filtering on list endpoints

**Current state:** `GET /api/episodes`, `GET /api/memories`, `GET /api/signals/history` return all records with no field-level filter.

**Target:** Add `?field=memory` query parameter to filter results by field namespace. E.g.:
- `GET /api/episodes?field=memory` → only memory field episodes
- `GET /api/signals/history?field=identity` → only identity signals

**Files:** `src/interfaces/rest/handlers/` (signals, memories, episodes)

**Effort:** 1h

---

### 1C — Gradual shutdown hardening

**Current state:** `CancellationToken` exists, `tokio::signal::ctrl_c()` is handled, but the shutdown sequence doesn't drain in-flight requests before stopping processors.

**Target:** Implement proper 3-phase shutdown:
1. Stop accepting new requests (grace period)
2. Drain in-flight cascade (wait for current signals to settle)
3. Shutdown processors and persist state

**Files:** `src/main.rs`, `src/kernel/runtime.rs`

**Effort:** 1-2h

---

## Phase 2 — Web UI Overhaul

### 2A — Full dashboard application

**Current state:** ~295 lines of inline HTML/JS in `src/interfaces/rest/dashboard.rs`. Single page with basic field status table and signal injection input.

**Target:** A proper management dashboard as a Rust-served single-page application, with:

| View | Purpose | Data Source |
|------|---------|------------|
| **System Overview** | Live health, uptime, signal rate, processor count | `/api/observability/overview` |
| **Field Inspector** | Per-field state with expandable detail panels | `/api/<field>/detail` endpoints |
| **Signal Explorer** | Real-time signal stream with filtering/severity | `/api/events/stream` (SSE) |
| **Signal Injector** | Type signal type + optional payload → send | `POST /api/signals/inject` |
| **Processor Registry** | See all 49 processors, subscriptions, capabilities | `/api/capabilities`, `/api/observability/processors` |
| **Plugin Manager** | View loaded plugins, trigger reload, view manifests | Plugin registry APIs |
| **Event Browser** | Browse event history with time range, type filter, pagination | `/api/signals/history` + EventStore |
| **Metrics Dashboard** | Charts for signal throughput, processor latency, field state sizes | `/api/observability/signals`, `/api/stats` |

**Implementation approach:** Serve a complete HTML/CSS/JS application (not a framework — keep it dependency-free). The JS fetches all data via REST endpoints and renders via DOM manipulation. Uses SSE for live updates.

**Files:**
- `src/interfaces/rest/dashboard.rs` — complete rewrite
- Static assets served via axum (or inline in Rust)

**Effort:** 4-6h

---

### 2B — Dashboard API extensions

Some dashboard views need backend data that doesn't exist yet:

| API | Purpose | Status |
|-----|---------|--------|
| `GET /api/plugins` | List loaded plugins with version/status | ⬜ |
| `GET /api/plugins/<name>` | Plugin detail (processors, signals, config) | ⬜ |
| `POST /api/plugins/reload` | Trigger plugin manifest re-scan | ⬜ |
| `GET /api/config` | Current runtime configuration | ⬜ |

**Files:** `src/interfaces/rest/handlers/`, `src/interfaces/rest/mod.rs`

**Effort:** 1-2h

---

## Phase 3 — Testing Across Processes

### 3A — Field integration tests (complete the planned 8)

**Current state:** 3 integration test files with 22 tests total. The architecture plan (NOESISV2 §2B) specified 8 field-level test files.

**Target:** Add dedicated integration test files for each remaining field:

| Test File | What It Covers | Status |
|-----------|---------------|--------|
| `tests/cascade_test.rs` | Multi-field cascade propagation | ✅ exists (7 tests) |
| `tests/field_integration_test.rs` | Cross-field integration | ✅ exists (6 tests) |
| `tests/interface_test.rs` | REST endpoints + auth | ✅ exists (9 tests) |
| `tests/field_memory_test.rs` | Full memory cascade | ⬜ |
| `tests/field_identity_test.rs` | Identity cascade (MemoryConsolidated → Belief → Identity → Goal) | ⬜ |
| `tests/field_agency_test.rs` | Agency cascade (Goal → Priority → Strategy → Opportunity) | ⬜ |
| `tests/field_awareness_test.rs` | Awareness cascade (Observer → Attention → Curiosity → Mood → etc.) | ⬜ |
| `tests/field_action_test.rs` | Action cascade (Plan → Task → Execution → Evaluation → Risk) | ⬜ |

Each test file should:
- Inject a field-relevant root signal
- Verify the full processor chain fires
- Verify field state is updated
- Verify no unintended cross-field signal leakage

**Files:** `tests/field_*.rs` (5 new files)

**Effort:** 3-5h

---

### 3B — End-to-end cascade stress test

**Target:** A `tests/e2e_test.rs` that:
1. Starts a full runtime (EventBus + FieldRuntime + all 49 processors)
2. Injects a realistic `IngestRequest` signal
3. Waits for the cascade to converge
4. Verifies signals were emitted across all 7 fields
5. Verifies field state changed appropriately
6. Measures cascade depth and signal count

This validates the entire cognitive pipeline from raw ingest → episode → extraction → consolidation → belief → identity → goal → plan → task → awareness → reasoning → simulation.

**Files:** `tests/e2e_test.rs`

**Effort:** 2-3h

---

### 3C — Storage backend integration tests

**Current state:** Postgres and Redis backends exist in `src/storage/backends/` but are only unit-tested with mocks.

**Target:** Integration tests that:
- Start a Postgres container (or connect to one via env var)
- Run full CRUD against the Postgres backend
- Same for Redis
- Verify data survives restart (file store already tested)

**Files:** `tests/storage_postgres_test.rs`, `tests/storage_redis_test.rs`

**Effort:** 2h (lighter if using existing infra)

---

### 3D — Plugin loading integration test

**Target:** A `tests/plugin_integration_test.rs` that:
1. Creates a temporary `~/.noesis/plugins/test-plugin/plugin.json`
2. Starts Noesis with plugin discovery
3. Verifies the plugin's processors are registered
4. Verifies field registration from plugins
5. Cleans up temp files

**Files:** `tests/plugin_integration_test.rs`

**Effort:** 1h

---

## Phase 4 — Production Readiness

### 4A — Self-hosting / Docker packaging

**Target:** Noesis runs in production with a single command:

```
├── Dockerfile              # Multi-stage: compile + distroless runtime
├── docker-compose.yml       # Noesis + optional Postgres + Redis
├── noesis.service           # systemd unit file
└── .noesis/config.toml      # Runtime configuration
```

**Dockerfile structure:**
```dockerfile
# Stage 1: Build
FROM rust:1.x AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/noesis /usr/local/bin/noesis
COPY --from=builder /app/.noesis /root/.noesis
EXPOSE 8080 8645
ENTRYPOINT ["noesis", "start"]
```

**Files:** `Dockerfile`, `docker-compose.yml`, `noesis.service` (at project root)

**Effort:** 1-2h

---

### 4B — Plugin hot-reload

**Current state:** Plugin manifests are scanned once at startup. To add a plugin, you restart Noesis.

**Target:**
- Add `POST /api/plugins/reload` endpoint
- Re-scan `~/.noesis/plugins/*/plugin.json`
- Register new plugins, remove stale ones
- Emit `kernel.plugin.loaded` / `kernel.plugin.failed` signals
- Wire into the dashboard Plugin Manager view

**Files:** `src/kernel/plugin.rs`, `src/interfaces/rest/handlers/`, dashboard

**Effort:** 2-3h

---

### 4C — Storage backend wiring

**Current state:** Postgres and Redis backends exist as module files but the main data path uses `MemoryStore` by default.

**Target:**
- Wire Postgres backend as the production storage
- Auto-detect `DATABASE_URL` env var at startup
- Redis for cache/metrics (optional, graceful fallback)
- Config flag: `--storage memory|postgres`

**Files:** `src/storage/store.rs`, `src/main.rs`, `src/kernel/state.rs`

**Effort:** 2-3h

---

## Estimation

| Phase | Items | Rough Effort |
|-------|-------|-------------|
| 1A | Rewrite main.rs cascade | 2-3h |
| 1B | Field filtering on endpoints | 1h |
| 1C | Gradual shutdown hardening | 1-2h |
| 2A | Full dashboard application | 4-6h |
| 2B | Dashboard API extensions | 1-2h |
| 3A | Field integration tests (5 new files) | 3-5h |
| 3B | E2E cascade stress test | 2-3h |
| 3C | Storage backend integration tests | 2h |
| 3D | Plugin integration test | 1h |
| 4A | Docker packaging | 1-2h |
| 4B | Plugin hot-reload | 2-3h |
| 4C | Storage backend wiring | 2-3h |
| **Total** | | **22-33h** |

## Ordering

Work in phase order — each builds on the previous:

1. **Phase 1 first** — architecture cleanup makes everything else cleaner. The main.rs rewrite is the most invasive change; do it early so tests validate the new topology.
2. **Phase 2 second** — the dashboard is the user-facing deliverable. It also reveals missing APIs (Phase 2B) that need to be built.
3. **Phase 3 third** — integration tests validate Phases 1+2 didn't break anything. The E2E stress test is the milestone for "system works end-to-end."
4. **Phase 4 last** — production packaging is polish after everything works.

Within each phase, work items in listed order.

## Success Criteria

At end of Sprint 3:

- [ ] `main.rs` cascade loop delegates to `FieldRuntime::process_signal()` — no inline dispatch
- [ ] All list endpoints support `?field=<name>` filtering
- [ ] Graceful shutdown drains in-flight work before exit
- [ ] Web dashboard serves 8 views: Overview, Fields, Signals, Inject, Processors, Plugins, Events, Metrics
- [ ] Dashboard uses SSE for real-time updates
- [ ] 8 integration test files (3 existing + 5 new) covering all 7 fields
- [ ] E2E stress test validates ingest→all-fields cascade
- [ ] Plugin hot-reload works via `POST /api/plugins/reload`
- [ ] Docker image builds and runs (`docker build . && docker run noesis`)
- [ ] `cargo test` clean (all tests pass, 0 warnings)
- [ ] `cargo build` clean (0 warnings)

## Version Target

Sprint 3 delivers **Noesis v1.0.0-beta** — a functionally complete, production-packaged cognitive operating system with a full management UI.
