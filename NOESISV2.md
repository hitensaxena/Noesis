# Noesis Sprint 2 — Cognitive Substance Over Skeleton

> **Sprint 2:** 2026-06-26 → (ongoing)  
> **Theme:** Real cognitive behavior, comprehensive tests, full hardening  
> **Build:** ✅ V1 clean — 0 warnings, 59 tests, 49 processors  

---

## Why Sprint 2

V1 completed the **architecture skeleton** — signal cascade, field runtime, beat coordinator, plugin system, REST/MCP interfaces. What it left behind:

1. **22 stub processors** — registered but trivial (16-line shells with canned responses)
2. **Thin test coverage** — only 3 processor files have unit tests, 7 integration tests cover only 9 of 49 processors
3. **No dynamic plugins** — `PluginRegistry` exists but can't load external manifests
4. **No transaction safety** — `TransactionManager` is a no-op stub
5. **No event persistence** — all signal history lost on restart
6. **No web dashboard** — only a terminal TUI
7. **No API docs** — consumers have to read source

Each of these is a direct gap in the architecture's own requirements. Sprint 2 closes them.

---

## Sprint 2 Phases

```
Phase 1: Implement 22 stub processors   ← biggest gap, highest impact
Phase 2: Comprehensive test coverage    ← the architecture requires it
Phase 3: Hardening                      ← plugin loading, transactions, persistence
Phase 4: Documentation & tooling        ← API docs, dashboard, examples
```

---

## Phase 1 — Implement 22 Stub Processors

Each stub is a registered processor that declares signal subscriptions and emits signal structs, but its `process()` method returns a hardcoded dummy. Real implementation requires:

- **State tracking** — buffers, counters, ring buffers for recent input
- **Signal emission logic** — condition-based, not hardcoded
- **Deep termination** — signals carry actual computed values, not zeroed defaults

### 1A — Memory field (4 processors)

| Processor | File | Logic needed |
|-----------|------|-------------|
| `DecayProcessor` | `memory/decay_processor.rs` | Track episode timestamps; on BEAT_SLOW compute decay factor from recency; emit MemoryDecayed with count of episodes below threshold |
| `DedupProcessor` | `memory/dedup_processor.rs` | Maintain content-hash set of recent episode texts; on ingest, detect duplicates; emit DedupSkipped with matched id |
| `IndexingProcessor` | `memory/indexing_processor.rs` | On episode.recorded, extract key terms via simple TF scoring; emit IndexUpdated with term→episode mapping |
| `RetrievalProcessor` | `memory/retrieval_processor.rs` | On recall queries, score stored episodes by keyword overlap; return ranked results via EpisodesRetrieved signal |

### 1B — Agency field (3 processors)

| Processor | Logic needed |
|-----------|-------------|
| `PriorityProcessor` | Maintain priority queue of goals; on BEAT_FAST re-score by urgency/importance; emit PrioritiesReordered if order changed |
| `StrategyProcessor` | Accept goal.completed events; adjust strategy parameters; on BEAT_MEDIUM emit StrategyUpdated with current approach |
| `OpportunityProcessor` | Scan incoming signals for patterns matching opportunity profiles; emit OpportunityDetected with score and rationale |

### 1C — Reasoning field (7 processors)

| Processor | Logic needed |
|-----------|-------------|
| `AnalogyProcessor` | Apply simple structural mapping between source/target concepts; emit AnalogyDetected with mapping confidence |
| `ConceptProcessor` | Cluster entity-relation triples into concept groups; on BEAT_SLOW emit ConceptFormed with member entities |
| `DecisionProcessor` | Evaluate decision candidates against goal priorities; emit DecisionEvaluated with recommendation |
| `HypothesisProcessor` | On belief.changed or curiosity.detected, generate testable hypotheses; emit HypothesisFormed |
| `MentalModelProcessor` | Maintain simplified model of the world from entity relationships; on entity changes, emit MentalModelUpdated |
| `ReasoningProcessor` | Coordinating processor — chain sub-results from other reasoning processors; emit ConclusionReady |
| `SynthesisProcessor` | Merge related memories/narratives into higher-level synthesis; on BEAT_MEDIUM emit SynthesisReady |

### 1D — Awareness field (2 processors)

| Processor | Logic needed |
|-----------|-------------|
| `PatternProcessor` | Detect recurring topics across episodes (simple n-gram frequency); emit PatternDetected with signal strength |
| `OpenLoopsProcessor` | Track unresolved goals, pending decisions, unanswered curiosities; emit OpenLoopsReport on BEAT_MEDIUM |

### 1E — Simulation field (6 processors)

| Processor | Logic needed |
|-----------|-------------|
| `WorldModelProcessor` | Maintain entity→state map from knowledge graph signals; emit WorldModelUpdated on entity.edge changes |
| `AssumptionProcessor` | Extract implicit assumptions from decisions/goals; on BEAT_SLOW emit AssumptionTested with validity check |
| `ScenarioProcessor` | Branch on current state × possible actions; emit ScenarioReady with projected outcomes |
| `CounterfactualProcessor` | On decision.evaluated, compute "what if X had been different"; emit CounterfactualReady |
| `ForecastingProcessor` | Extrapolate trends from entity timelines; emit ForecastReady with prediction intervals |
| `SimulationRiskProcessor` | Cross-reference scenario outputs with known risk profiles; emit RiskAssessed (simulation variant) |

### 1F — Action field (7 processors — already have skeleton logic)

The Action field processors (`plan_processor.rs`, `project_processor.rs`, `task_processor.rs`, `execution_processor.rs`, `evaluation_processor.rs`, `risk_processor.rs`, `recovery_processor.rs`) have more code than stubs (31–61 lines each) but still simple. Review and deepen:

- `PlanProcessor` — maintain actual plan structure (steps, dependencies, status)
- `TaskProcessor` — decompose goals into actionable tasks with priority/estimate
- `ExecutionProcessor` — track running task state, detect stalls
- `RiskProcessor` — accumulate risk factors from all sources, compute aggregate
- `RecoveryProcessor` — on failure signals, generate recovery proposals
- Each gets real state tracking + unit tests

---

## Phase 2 — Comprehensive Test Coverage

### 2A — Unit tests per processor (22 new test modules)

Every processor file gets a `#[cfg(test)] mod tests` block with:

- **Construction test** — `Processor::new()` succeeds, names match
- **Subscription test** — `subscribed_signals()` and `emitted_signals()` return expected lists
- **Process test** — feed a relevant signal, verify correct emission
- **Edge cases** — empty input, duplicate input, boundary conditions
- **State test** — verify processor maintains correct internal state across multiple calls

### 2B — Integration tests by field (8 new test files)

```
tests/field_memory_test.rs
tests/field_identity_test.rs
tests/field_agency_test.rs
tests/field_action_test.rs
tests/field_awareness_test.rs
tests/field_reasoning_test.rs
tests/field_simulation_test.rs
tests/field_knowledge_test.rs
```

Each tests the full cascade for one field:
- Inject field-relevant signals → verify processor chain → verify field state updated
- Verify correct signals emitted, no unintended cross-field interference

### 2C — LLM engine tests

- **Tier routing** — verify Fast/Agentic/Deep dispatch picks correct provider
- **API key discovery** — test env var → config → fallback chain
- **Rate limiting** — verify backpressure when limits hit
- **Model fallback** — verify 503 → next-tier fallback behavior
- **Mock provider** — test against a local mock HTTP server

### 2D — Interface tests

- **MCP server** — test JSON-RPC request/response cycle (tools/list, tools/call for recall, inject, field_state)
- **REST endpoints** — test health, ingest, memories, recall, capabilities
- **Auth middleware** — test valid/missing/expired API keys
- **BEAT injection via API** — test `/api/signals/inject` for each beat type

### 2E — Storage tests

- **MemoryStore** — test CRUD for episodes, memories, entities, edges
- **EventStore** — test event recording, replay, pagination
- **Postgres backend** — test connection pool, query execution (integration, requires PG)
- **Redis backend** — test cache operations (integration, requires Redis)

---

## Phase 3 — Hardening

### 3A — Dynamic plugin loading

- **Manifest format** — `~/.noesis/plugins/<name>/plugin.yaml` with name, version, processors, capabilities
- **Scanner** — watch directory on startup, register found plugins
- **`fn fields()` on Plugin trait** — allow plugins to register entire new fields
- **Hot-reload** — optional SIGHUP-driven re-scan
- **Sandbox** — plugin isolation (panic boundary, resource limits)
- **Tests** — manifest parsing, field registration, duplicate detection

### 3B — Transaction rollback

- **`TransactionManager` implementation** — begin/commit/rollback via stored checkpoint
- **Processor failure isolation** — if one processor in a cascade panics, roll back its field mutations
- **Partial success handling** — cascade continues even if some processors fail (best-effort)
- **Tests** — inject failure, verify field state restored

### 3C — Event persistence to disk

- **File-backed EventStore** — append-only JSONL format, auto-rotate at size threshold
- **Read/seek** — replay from checkpoint, query by time range
- **Replace signal_history stub** — wire to real EventStore
- **Tests** — write then restart, verify events survive

### 3D — REST API refinements

- **Structured error responses** — consistent `{error, code, detail}` envelope
- **Rate limiting** — per-IP token bucket via tower middleware
- **Pagination** — list endpoints accept `?limit=&offset=`
- **Field filtering** — `GET /api/episodes?field=memory` filter
- **Graceful shutdown** — drain in-flight requests on SIGTERM

---

## Phase 4 — Documentation & Tooling

### 4A — API documentation (OpenAPI 3.0)

- Generate `openapi.yaml` covering all 21+ REST endpoints
- Each endpoint: path, method, description, request/response schema, auth requirement
- Server URL, security scheme, tags for field grouping
- Serve at `GET /api/docs` via Swagger UI

### 4B — MCP tool documentation

- Document each MCP tool: name, description, input schema, output schema
- Usage examples for recall, inject, field_state, capabilities
- Error codes and troubleshooting

### 4C — Basic web dashboard

- Simple static HTML/JS served by Axum at `/dashboard/`
- Field state inspection (read each field's current state via REST)
- Signal injection console (type a signal type + payload, see it propagate)
- Live cascade log (SSE stream of signals as they fire)
- Processor registry browser (see all 49 processors, their subscriptions, current status)

### 4D — Code documentation

- Processor-level doc comments: "what this does, what signals it consumes/produces, state it maintains"
- Field-level docs: "what question this field answers, what state it owns"
- Signal docs: "when this signal is emitted, what data it carries"
- Architecture Decision Records (ADRs) in `docs/adr/` for key decisions

---

## Estimation

| Phase | Items | Rough effort |
|-------|-------|-------------|
| 1A Memory processors | 4 processors + tests | 2–3h |
| 1B Agency processors | 3 processors + tests | 1–2h |
| 1C Reasoning processors | 7 processors + tests | 3–4h |
| 1D Awareness processors | 2 processors + tests | 1h |
| 1E Simulation processors | 6 processors + tests | 2–3h |
| 1F Action processor review | 7 processors + deeper tests | 2h |
| 2B Integration tests | 8 test files | 2–3h |
| 2C LLM engine tests | Mock provider + tier tests | 1–2h |
| 2D Interface tests | MCP + REST + auth | 1–2h |
| 3A Dynamic plugin loading | Manifest + scanner + tests | 2h |
| 3B Transaction rollback | Checkpoint + isolation | 2h |
| 3C Event persistence | File-backed EventStore | 1h |
| 3D REST refinements | Error handling + rate limiting | 1–2h |
| 4A OpenAPI spec | Schema generation | 1h |
| 4C Web dashboard | Static HTML served by Axum | 3–4h |
| **Total** | | **27–34h** |

---

## Ordering

Work in the numbered order above — each phase builds on the previous:

1. **Phase 1 first** — without real processors the system is a skeleton. Every processor gets real behavior + its own unit test.
2. **Phase 2 second** — the integration and interface tests verify everything works together.
3. **Phase 3 third** — hardening is wasted on untested code; do it after coverage exists.
4. **Phase 4 last** — documentation and dashboard are polish, not substance.

Within each phase, group by field (memory → identity → agency → action → awareness → reasoning → simulation) to minimize context switching.

---

## Success criteria

At end of Sprint 2:

- [ ] All 49 processors have real (non-trivial) `process()` implementations
- [ ] Every processor file has its own `#[cfg(test)] mod tests`
- [ ] At least 40 additional unit tests across processor files
- [ ] At least 8 integration test files covering all fields
- [ ] LLM tier routing has tests
- [ ] REST + MCP interfaces have tests
- [ ] Dynamic plugin loading works from `~/.noesis/plugins/`
- [ ] Transaction rollback protects field state on processor failure
- [ ] Event persistence survives restart (file-backed store)
- [ ] REST API has consistent error responses and pagination
- [ ] OpenAPI spec generated
- [ ] Web dashboard serves at `/dashboard/`
- [ ] `cargo test` clean (0 warnings, all new tests pass)
- [ ] `cargo build` clean (0 warnings)
