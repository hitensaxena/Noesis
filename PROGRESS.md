# Noesis — Progress

> v0.1.0 | Decentralized Cognitive Architecture

## Phase 0: Activation Signal Model ✅
- [x] Activation, salience, novelty, confidence, decay on `Signal` struct
- [x] Default values (activation=1.0, decay=0.7)
- [x] Activation threshold on `Processor` trait (default: 0.1)
- [x] Cascade convergence via activation decay (not max depth)

## Phase 1: Structural Restructure ✅
- [x] `kernel/` directory with EventBus, Scheduler, Lifecycle, Metrics, Plugin
- [x] `field_runtime/` with context, dispatcher, snapshot, transactions
- [x] `fields/` with memory, identity, agency, action, awareness, reasoning, simulation
- [x] Field domain files for state organization (all 7 fields)
- [x] Processors organized per-field (49+ processors across all fields)

## Phase 2: Field Runtime ✅
- [x] SignalDispatcher — routes signals by signal_type prefix
- [x] FieldContext — state access, signal emission, metrics, storage
- [x] Processor registration through FieldRuntime
- [x] Signal cascade loop with breadth-first queue and activation decay

## Phase 3: Cognitive Beats ✅
- [x] BeatCoordinator — BEAT_IMMEDIATE, FAST, MEDIUM, SLOW
- [x] Beat signal types in kernel signal namespace
- [x] Processors can subscribe to beat signals

## Phase 4: Plugin System ✅
- [x] Plugin trait with registry
- [x] All processors registered as built-in Noesis plugin
- [x] Manifest discovery (`~/.noesis/plugins/`)
- [x] Capability query via REST API + TUI
- [x] Hot-reload via `POST /api/plugins/reload`

## Phase 5: New Processors ✅
- [x] ObserverProcessor (awareness — state transitions)
- [x] Metacognition processor (reasoning — metacognitive insight)
- [x] MoodProcessor (awareness — valence/arousal estimation)
- [x] ContextConstructor (memory — retrieval context assembly)
- [x] EpistemicClassifier (reasoning — statement classification)
- [x] ConfidenceEstimator (reasoning — confidence scoring)
- [x] PlanDecomposer (action — goal → plan)
- [x] HealthChecker (awareness — subsystem health)
- [x] ValueExtractor (identity — decisions → values)
- [x] PrincipleDistiller (identity — decisions → principles)
- [x] TraitProcessor (identity — beliefs → traits)

## Phase 6: Production Readiness ✅
- [x] Docker multi-stage build
- [x] docker-compose.yml with Postgres/Redis profiles
- [x] systemd service unit
- [x] REST API (32 endpoints)
- [x] 8-view web dashboard (Overview, Fields, Signals, Inject, Processors, Plugins, Events, Metrics)
- [x] TUI (ratatui) with dashboard, fields, processors, signals, settings, observability, log
- [x] PostgresEventStore for persistent signal history
- [x] Redis + Postgres storage backends
- [x] LLM 3-tier routing (fast/agentic/deep)
- [x] Auth middleware with bearer token / X-API-Key support

## Current Metrics
- **Tests:** 263+ (0 failures)
- **Source files:** ~180 .rs files
- **Fields:** 8 (memory, identity, agency, action, awareness, reasoning, simulation, knowledge_graph)
- **Processors:** 50+ with `fn process()` implemented
- **Signal types:** 57 defined, all registered
- **Build warnings:** 0
