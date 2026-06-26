# Noesis Architecture Evolution — Complete Specification

## Context

Noesis has grown from 0 to ~12,400 lines across 104 Rust source files, 11 processors, 6 fields, 17 signal types, a 3-tier LLM engine, REST API, TUI, Hermes plugin, and MCP server. The architecture is event-driven and decentralized, but it has reached a point where the flat organization limits growth. All processors live in a single directory, fields have no sub-domain structure, signals are in a flat catalog, and there is no abstraction for grouping related processors.

The goal of this evolution is to preserve everything that works while introducing the structural abstractions needed to scale to hundreds of processors without becoming difficult to understand.

---

## 1. High-Level Architecture Diagram

```
┌────────────────────────────────────────────────────────────┐
│                        KERNEL                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ EventBus │  │ Registry │  │ Runtime  │  │ Lifecycle │  │
│  └────┬─────┘  └──────────┘  └──────────┘  └───────────┘  │
└───────┼────────────────────────────────────────────────────┘
        │
        │ Signal Mesh (broadcast + mpsc fan-in)
        │
┌───────┼────────────────────────────────────────────────────┐
│       ▼                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Memory  │  │ Identity │  │Executive │  │Awareness │   │
│  │  Field   │  │  Field   │  │  Field   │  │  Field   │   │
│  ├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤   │
│  │ Capture  │  │ SelfModel│  │ Goals    │  │ Observer │   │
│  │ Extract  │  │ Beliefs  │  │ Planning │  │ Attention│   │ FIELDS
│  │ Organize │  │ Values   │  │ Strategy │  │Curiosity │   │
│  │ Index    │  │ Traits   │  │Eval/Risk │  │ Analytics│   │
│  │ Retrieve │  │Timeline  │  │          │  │ Mood     │   │
│  │ Maintain │  │          │  │          │  │ Health   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Cognition │  │Simulation│  │Knowledge │  │  Future  │   │
│  │  Field   │  │  Field   │  │  Field   │  │  Fields  │   │
│  ├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤   │
│  │Reasoning │  │WorldMdl │  │ Graph    │  │ Learning │   │
│  │MetaCog   │  │Forecast  │  │Embedding │  │ Creative │   │
│  │Decisions │  │Scenarios │  │Resolutn  │  │ Social   │   │
│  │MentalMdl │  │ Risk     │  │          │  │ Health   │   │
│  │Epistemics│  │ Counter  │  │          │  │ Vision   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                             │
│  DOMAINS (each logical grouping within a field)             │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │             PROCESSORS (one per signal transform)     │   │
│  │  Each subscribes → transforms → emits                │   │
│  │  No direct invocation. No knowledge of pipeline.     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              SIGNAL MESH                              │   │
│  │  All signals flow through the EventBus.               │   │
│  │  Broadcast → mpsc fan-in → recursive cascade         │   │
│  │  Equilibrium detection at each cascade step          │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
        │
        │ Interfaces
        ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│ REST API │ │   TUI    │ │   CLI   │ │MCP Server│ │  Hermes  │
└──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘

┌────────────────────────────────────────────────────────────┐
│                        PLUGIN SYSTEM                        │
│  Auto-discovery of processors, signals, jobs, config,       │
│  storage backends, and capabilities at startup              │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│                      CAPABILITY LAYER                       │
│  Every processor advertises capabilities (PatternDetection, │
│  Reasoning, Extraction, etc.) — enables dynamic discovery   │
└────────────────────────────────────────────────────────────┘
```

## 2. Revised Directory Structure

```
src/
├── bin/noesis_tui.rs
├── lib.rs
├── main.rs
│
├── kernel/                          # Core infrastructure (rename from core/)
│   ├── mod.rs
│   ├── event_bus.rs                 # Signals, broadcasting, dispatch (from eventbus/)
│   ├── registry.rs                  # Field/Processor/Signal registration
│   ├── runtime.rs                   # Task management, CancellationToken
│   ├── lifecycle.rs                 # Ordered startup/shutdown
│   ├── state.rs                     # SystemState, FieldStateCache
│   └── signal.rs                    # SignalType, SignalMeta, Signal trait
│
├── fields/                          # Each field is a module directory
│   ├── mod.rs
│   │
│   ├── memory/                      # +++ EXPANDED +++
│   │   ├── mod.rs                   # MemoryField — orchestrates all memory domains
│   │   ├── state.rs                 # MemoryFieldState (episodic, semantic, procedural, working)
│   │   ├── capture.rs               # Domain: Capture (Ingest → EpisodeRecorded)
│   │   ├── extraction.rs            # Domain: Extraction (Episode → Triples/Facts) ← existing
│   │   ├── resolution.rs            # Domain: Entity Resolution ← existing (from processors/)
│   │   ├── consolidation.rs         # Domain: Consolidation ← existing
│   │   ├── retrieval.rs             # Domain: Retrieval (hybrid search, recall)
│   │   ├── indexing.rs              # Domain: Indexing (embedding, graph index)
│   │   └── maintenance.rs           # Domain: Decay, dedup, conflict resolution
│   │
│   ├── identity/                    # +++ REDESIGNED +++
│   │   ├── mod.rs                   # IdentityField
│   │   ├── state.rs                 # IdentityFieldState
│   │   ├── self_model.rs            # Domain: Self Model (core identity representation)
│   │   ├── beliefs.rs               # Domain: Beliefs (from processors/belief.rs)
│   │   ├── values.rs                # Domain: Values (pending)
│   │   ├── traits.rs                # Domain: Traits
│   │   ├── roles.rs                 # Domain: Roles (pending)
│   │   ├── principles.rs            # Domain: Principles (pending)
│   │   └── timeline.rs              # Domain: Identity Timeline (pending)
│   │
│   ├── executive/                   # +++ EXPANDED +++
│   │   ├── mod.rs                   # ExecutiveField
│   │   ├── state.rs                 # ExecutiveFieldState
│   │   ├── goals.rs                 # Domain: Goals (from processors/goal.rs)
│   │   ├── planning.rs              # Domain: Planning (pending — decomposes goals)
│   │   ├── execution.rs             # Domain: Execution (pending — task dispatch)
│   │   └── evaluation.rs            # Domain: Evaluation & Risk (pending)
│   │
│   ├── awareness/                   # +++ REDESIGNED +++
│   │   ├── mod.rs                   # AwarenessField
│   │   ├── state.rs                 # AwarenessFieldState
│   │   ├── observer.rs              # Domain: Observer (meta-observation)
│   │   ├── attention.rs             # Domain: Attention ← existing
│   │   ├── curiosity.rs             # Domain: Curiosity ← existing
│   │   ├── analytics.rs             # Domain: Analytics (pending)
│   │   ├── mood.rs                  # Domain: Mood (pending)
│   │   ├── health.rs                # Domain: Health (pending)
│   │   └── reflection.rs            # Domain: Reflection & Narrative ← existing
│   │
│   ├── cognition/                   # +++ NEW FIELD +++
│   │   ├── mod.rs                   # CognitionField
│   │   ├── state.rs                 # CognitionFieldState
│   │   ├── reasoning.rs             # Domain: Reasoning (pending)
│   │   ├── metacognition.rs         # Domain: Meta-Cognition (pending)
│   │   ├── decision.rs              # Domain: Decision Making (pending)
│   │   ├── mental_models.rs         # Domain: Mental Models (pending)
│   │   └── epistemics.rs            # Domain: Epistemics (pending)
│   │
│   ├── simulation/                  # +++ EXPANDED +++
│   │   ├── mod.rs                   # SimulationField
│   │   ├── state.rs                 # SimulationFieldState
│   │   ├── world_models.rs          # Domain: World Models (pending)
│   │   ├── forecasting.rs           # Domain: Forecasting (pending)
│   │   ├── scenarios.rs             # Domain: Scenarios (existing minimal)
│   │   └── risk.rs                  # Domain: Risk (pending)
│   │
│   └── knowledge/                   # +++ SPLIT FROM MEMORY +++
│       ├── mod.rs                   # KnowledgeField (was GraphField)
│       ├── state.rs                 # KnowledgeFieldState (entities, relations, embeddings)
│       ├── graph.rs                 # Domain: Graph store + queries (from fields/graph.rs)
│       ├── resolution.rs            # Domain: Entity resolution (from processors/resolution.rs)
│       └── embedding.rs             # Domain: Embedding & vector search (pending)
│
├── interfaces/                      # External interfaces
│   ├── mod.rs
│   ├── rest/                        # Axum REST API (keep current structure)
│   ├── tui/                         # ratatui TUI (keep current structure)
│   ├── cli.rs                       # CLI subcommands (keep current)
│   ├── mcp/                         # MCP server types (keep Python server standalone)
│   └── hermes/                      # Hermes plugin types (keep Python plugin standalone)
│
├── storage/                         # Persistence backends (keep current)
│   ├── mod.rs
│   ├── store.rs                     # Storage trait
│   ├── memory_store.rs
│   ├── event_store.rs
│   └── backends/
│       ├── mod.rs                   # CompositeStorage
│       ├── postgres_backend.rs
│       └── redis_backend.rs
│
├── engines/                         # External intelligence engines (keep current)
│   ├── mod.rs
│   ├── llm/                         # LLM engine with tiered routing
│   └── graph/                       # Graph extraction utilities
│
├── plugin/                          # +++ REDESIGNED +++
│   ├── mod.rs
│   ├── loader.rs                    # Auto-discovery at startup
│   ├── manifest.rs                  # Plugin manifest format
│   ├── registry.rs                  # Plugin registration
│   └── capability.rs                # Capability model
│
├── scheduler/                       # Scheduler (keep current)
│
└── metrics/                         # Metrics collector (keep current)
```

### Rationale for Structure

- **`kernel/` replaces `core/` + `eventbus/`** — these were always conceptually one layer. The EventBus is the kernel's primary communication mechanism. Merging avoids unnecessary module boundary.
- **Fields become directories** — each field gets a `mod.rs`, `state.rs`, and one file per domain. This scales to dozens of processors per field without file explosion.
- **Domains as files** — not subdirectories. Each domain file contains 2-5 related processors. A single file per domain is manageable up to ~300 lines.
- **`knowledge/` split from `memory/`** — Knowledge graph has fundamentally different state (graph topology), different access patterns (graph traversal vs memory retrieval), and different storage requirements. Splitting is cleaner.
- **`cognition/` as new field** — Reasoning, decision-making, and meta-cognition are fundamentally different from Awareness (which observes) and Executive (which acts). Cognition *thinks*.
- **`plugin/` expanded** — Plugin system gets a registry, capability model, and auto-discovery. The current stub is replaced with a real system.

---

## 3. Redesigned Field Hierarchy

### Memory Field

**Responsibility:** Owns all memory traces — what the organism remembers, how memories are stored, organized, retrieved, and maintained over time.

**Owns State:** Episodes (raw), Semantic memories (consolidated), Working memory (active context), Procedural knowledge, Indexes (by-source, by-tag, embedding vectors).

**Consumes Signals:** `INGEST_REQUEST`, `MEMORY_CONSOLIDATED`, `PATTERN_DETECTED`

**Emits Signals:** via its domain processors (listed below)

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **Capture** | Episodes buffer | `INGEST_REQUEST` | `EPISODE_RECORDED` | EpisodeProcessor |
| **Extraction** | Extraction rules | `EPISODE_RECORDED` | `FACT_EXTRACTED`, `TRIPLES_EXTRACTED` | ExtractionProcessor |
| **Resolution** | Entity cache, dedup map | `TRIPLES_EXTRACTED` | `ENTITY_CREATED`, `EDGE_CREATED` | ResolutionProcessor |
| **Consolidation** | Consolidation state, recent content | `EPISODE_RECORDED` | `MEMORY_CONSOLIDATED`, `PATTERN_DETECTED` | FastConsolidation, DeepConsolidation |
| **Retrieval** | Search indexes | recall/search signals | Retrieved results | ContentMatcher, HybridRetriever, ContextConstructor |
| **Indexing** | Embeddings, graph index | `ENTITY_CREATED`, `MEMORY_CONSOLIDATED` | `INDEX_UPDATED` | EmbeddingIndexer, GraphIndexer |
| **Maintenance** | Decay schedules, dedup rules | `MEMORY_CONSOLIDATED` | `MEMORY_DECAYED`, `CONFLICT_RESOLVED` | DecayProcessor, DedupProcessor, ConflictResolver |

**Future Processors:** WorkingMemoryManager, ProceduralMemoryExtractor, SemanticRouter, EpisodicReplay

---

### Identity Field

**Responsibility:** Owns the organism's self-model — who it is, what it believes, what it values, how it sees itself evolving.

**Owns State:** Beliefs, values, traits, roles, preferences, principles, identity version, timeline of identity changes.

**Consumes Signals:** `BELIEF_CHANGED`, `TRAIT_DETECTED`, `PRINCIPLE_DERIVED`

**Emits Signals:** `IDENTITY_UPDATED`, `VALUE_REFINED`, `ROLE_DETECTED`

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **SelfModel** | Core identity record, coherence score | `IDENTITY_UPDATED` | — (reads only) | SelfModelIntegrator |
| **Beliefs** | Belief vec, confidence scores | `MEMORY_CONSOLIDATED` | `BELIEF_CHANGED` | BeliefExtractor (LLM Fast) |
| **Values** | Value vec, importance scores | `BELIEF_CHANGED` | `VALUE_REFINED` | ValueExtractor (pending) |
| **Traits** | Trait vec, strength scores | `TRAIT_DETECTED` | — (reads only) | TraitDetector |
| **Roles** | Role vec, active flag | pattern signals | `ROLE_DETECTED` | RoleDetector (pending) |
| **Principles** | Principle vec | `DECISION_EVALUATED` | `PRINCIPLE_DERIVED` | PrincipleDistiller (pending) |
| **Timeline** | Identity change history | `IDENTITY_UPDATED` | — (reads only) | TimelineRecorder |

**Future Processors:** IdentityConsistencyChecker, IdentityConfidenceScorer, NarrativeSelfIntegrator, ValueConflictDetector

**Critical Rationale:** Identity should NOT own goals or executive functions. Goals belong to Executive. Identity answers "who am I?" not "what should I do?"

---

### Executive Field

**Responsibility:** Owns what the organism intends to do — goals, plans, tasks, strategy, and evaluation of outcomes. The biological analog is the prefrontal cortex.

**Owns State:** Goals (active/completed/abandoned), plans, tasks, strategy, risk register, decision history, evaluation scores.

**Consumes Signals:** `IDENTITY_UPDATED`, `OPPORTUNITY_DETECTED`, `EVALUATION_REQUESTED`

**Emits Signals:** `GOAL_CREATED`, `GOAL_COMPLETED`, `GOAL_ABANDONED`, `TASK_CREATED`, `PLAN_CREATED`, `DECISION_EVALUATED`

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **Goals** | Goal vec, priority queue | `IDENTITY_UPDATED` | `GOAL_CREATED`, `GOAL_COMPLETED` | GoalCreator (LLM Fast), GoalTracker |
| **Planning** | Plans, milestones | `GOAL_CREATED` | `PLAN_CREATED`, `TASK_CREATED` | PlanDecomposer (pending), TaskScheduler |
| **Execution** | Active tasks, status | `TASK_CREATED` | `TASK_COMPLETED` | TaskDispatcher (pending), ExecutionMonitor |
| **Evaluation** | Evaluation history, risk register | `TASK_COMPLETED`, `DECISION_EVALUATED` | `DECISION_EVALUATED` | OutcomeEvaluator, RiskAssessor (pending) |

**Future Processors:** StrategyFormulator, OpportunityDetector, PriorityResolver, DeadlineTracker, ResourceAllocator

---

### Awareness Field

**Responsibility:** Observes the organism's internal state without modifying it. Awareness is the introspective layer — it tracks what the organism is focused on, curious about, feeling, and how healthy the system is.

**Owns State:** Focus stack, salience map, curiosity items, mood history, health metrics, analytics, open loops.

**Consumes Signals:** All signals (via observer). Specifically: `EPISODE_RECORDED`, `CURIOSITY_DETECTED`, `MEMORY_CONSOLIDATED`, any signal with high salience.

**Emits Signals:** `ATTENTION_SHIFTED`, `CURIOSITY_DETECTED`, `NARRATIVE_GENERATED`, `MOOD_UPDATED`, `PATTERN_DETECTED`, `HEALTH_CHECK`

**Critical Constraint:** Awareness reads signal state, updates its own state, and emits observation signals. It NEVER modifies field state in other fields.

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **Observer** | Signal frequency, anomaly log | All signals | — (reads only) | SignalObserver, AnomalyDetector |
| **Attention** | Focus stack, salience map | `EPISODE_RECORDED`, `CURIOSITY_DETECTED` | `ATTENTION_SHIFTED` | SalienceComputer (LLM Fast), FocusManager |
| **Curiosity** | Curiosity items, gap map | `EPISODE_RECORDED`, `MEMORY_CONSOLIDATED` | `CURIOSITY_DETECTED` | GapDetector (LLM Agentic) |
| **Analytics** | Signal rates, processor latencies | Cascade completion events | — | MetricsObserver, PatternDetector |
| **Reflection** | Reflection reports, narratives | `EPISODE_RECORDED` (every 5) | `NARRATIVE_GENERATED` | NarrativeComposer, ReflectionEngine (LLM Deep) |
| **Mood** | Mood history, valence/energy | `EPISODE_RECORDED`, signficance signals | `MOOD_UPDATED` | MoodEstimator (pending) |
| **Health** | Health status, subsystem checks | Scheduler tick | `HEALTH_STATUS` | HealthChecker, LoadMonitor |

**Future Processors:** ThemeExtractor, BehaviorDriftDetector, LifeReplayEngine, OpenLoopTracker, MetaAwarenessIntegrator

---

### Cognition Field (NEW)

**Responsibility:** The organism's thinking layer — reasoning, problem-solving, decision-making, mental models, and epistemic analysis. If Awareness observes, Cognition thinks.

**Owns State:** Active reasoning chains, mental models, decision register, hypothesis space, epistemic classifications.

**Consumes Signals:** `DECISION_REQUESTED`, `PROBLEM_DETECTED`, `ANALOGY_TRIGGERED`

**Emits Signals:** `DECISION_EVALUATED`, `HYPOTHESIS_GENERATED`, `ANALOGY_DETECTED`, `MENTAL_MODEL_UPDATED`

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **Reasoning** | Reasoning chains | `PROBLEM_DETECTED` | `DECISION_EVALUATED` | ReasoningEngine (LLM Deep) |
| **MetaCognition** | Meta-cognitive monitoring | Signal patterns | — | MetaProcessor, ConfidenceEstimator (pending) |
| **MentalModels** | Mental model vec | `EPISODE_RECORDED`, pattern signals | `MENTAL_MODEL_UPDATED` | ModelBuilder (pending), ModelTester |
| **Decision** | Decision register | `DECISION_REQUESTED` | `DECISION_EVALUATED` | DecisionMaker (pending) |
| **Epistemics** | Epistemic classifications | `FACT_EXTRACTED` | `STATEMENT_CLASSIFIED` | EpistemicClassifier (pending) |

**Future Processors:** AnalogyEngine, HypothesisGenerator, KnowledgeSynthesizer, ConceptFormer, WorldUnderstandingIntegrator

**Rationale for New Field:** 
- Reasoning and decision-making are structurally different from observation (Awareness) and action (Executive)
- Awareness observes internal state; it doesn't think about it
- Executive acts on intentions; it doesn't reason about them
- Cognition sits between Awareness and Executive: it sees what Awareness observes and generates reasoned inputs that Executive can act on
- This follows the biological analog: sensory cortex (Awareness) → association cortex (Cognition) → prefrontal cortex (Executive)

---

### Simulation Field

**Responsibility:** Runs what-if scenarios, maintains world models, forecast outcomes, and assesses risk. The organism's imagination layer.

**Owns State:** World models (causal structures), scenarios, forecasts, risk assessments, counterfactuals, assumptions.

**Consumes Signals:** `GOAL_CREATED`, `DECISION_REQUESTED`, `OPPORTUNITY_DETECTED`

**Emits Signals:** `FORECAST_READY`, `RISK_ASSESSED`, `SCENARIO_SIMULATED`, `ASSUMPTION_VALIDATED`

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **WorldModels** | Causal model graph | `EPISODE_RECORDED` | `MODEL_UPDATED` | CausalModelBuilder (pending) |
| **Forecasting** | Forecast records | `GOAL_CREATED` | `FORECAST_READY` | OutcomePredictor (pending) |
| **Scenarios** | Scenario space | `DECISION_REQUESTED` | `SCENARIO_SIMULATED` | ScenarioRunner (pending) |
| **Risk** | Risk register | `FORECAST_READY` | `RISK_ASSESSED` | RiskAssessor (pending) |
| **Counterfactuals** | Counterfactual cache | `DECISION_EVALUATED` | `COUNTERFACTUAL_READY` | CounterfactualEngine (pending) |

**Future Processors:** AssumptionTracker, PlanningSandbox, DecisionSimulator, FutureProjector, ModelValidator

---

### Knowledge Field (SPLIT FROM MEMORY)

**Responsibility:** Owns structured knowledge — entities, relationships, graph topology, and semantic embeddings. Where Memory stores episodes, Knowledge stores structured facts.

**Owns State:** Entities (with categories and properties), Relations (typed edges with confidence), Entity resolution cache, Embedding vectors.

**Consumes Signals:** `TRIPLES_EXTRACTED`, `ENTITY_CREATED`, `EDGE_CREATED`

**Emits Signals:** `KNOWLEDGE_QUERIED`, `ENTITY_RESOLVED`, `EMBEDDING_UPDATED`

**Domains:**

| Domain | State Owned | Signals Consumed | Signals Emitted | Processors |
|--------|-------------|------------------|-----------------|------------|
| **Graph** | Entity + Relation stores | `ENTITY_CREATED`, `EDGE_CREATED` | — (query API) | GraphStore, GraphQuery (moved from fields/graph.rs) |
| **Resolution** | Entity name→ID cache | `TRIPLES_EXTRACTED` | `ENTITY_CREATED`, `EDGE_CREATED` | EntityResolutionProcessor (moved from processors/) |
| **Embedding** | Vector index | `ENTITY_CREATED` | `EMBEDDING_UPDATED` | EmbedingIndexer (pending) |

**Rationale for Splitting:** Knowledge graph is structurally different from episodic/semantic memory:
- Memory stores linear sequences (episodes) with temporal ordering
- Knowledge stores graph structures (entities + relations) with topological ordering
- Memory retrieval is about recency and relevance 
- Knowledge retrieval is about connectivity and traversal
- Memory uses embedding similarity; Knowledge uses graph traversal
- IMemory has decay (old episodes become less relevant); Knowledge accumulates (entities persist)
- A single field managing both would need two fundamentally different storage engines, two index types, and two retrieval pipelines. Splitting keeps each field simpler.

---

## 4. Processor Catalog

### Current Processors (11) — where they move

| Current Location | Processor | Moves To | Signal Sub | Signal Emit |
|-----------------|-----------|----------|------------|-------------|
| `processors/episode.rs` | EpisodeProcessor | `fields/memory/capture.rs` | `INGEST_REQUEST` | `EPISODE_RECORDED` |
| `processors/extraction.rs` | ExtractionProcessor | `fields/memory/extraction.rs` | `EPISODE_RECORDED` | `TRIPLES_EXTRACTED` |
| `processors/resolution.rs` | ResolutionProcessor | `fields/knowledge/resolution.rs` | `TRIPLES_EXTRACTED` | `ENTITY_CREATED`, `EDGE_CREATED` |
| `processors/consolidation.rs` | ConsolidationProcessor | `fields/memory/consolidation.rs` | `EPISODE_RECORDED` | `MEMORY_CONSOLIDATED`, `PATTERN_DETECTED` |
| `processors/belief.rs` | BeliefProcessor | `fields/identity/beliefs.rs` | `MEMORY_CONSOLIDATED` | `BELIEF_CHANGED` |
| `processors/identity.rs` | IdentityProcessor | `fields/identity/self_model.rs` | `BELIEF_CHANGED` | `IDENTITY_UPDATED` |
| `processors/goal.rs` | GoalProcessor | `fields/executive/goals.rs` | `IDENTITY_UPDATED` | `GOAL_CREATED` |
| `processors/narrative.rs` | NarrativeProcessor | `fields/awareness/reflection.rs` | `EPISODE_RECORDED` | `NARRATIVE_GENERATED` |
| `processors/attention.rs` | AttentionProcessor | `fields/awareness/attention.rs` | `EPISODE_RECORDED`, `CURIOSITY_DETECTED` | `ATTENTION_SHIFTED` |
| `processors/curiosity.rs` | CuriosityProcessor | `fields/awareness/curiosity.rs` | `EPISODE_RECORDED`, `MEMORY_CONSOLIDATED` | `CURIOSITY_DETECTED` |
| `processors/reflection.rs` | ReflectionProcessor | `fields/awareness/reflection.rs` | `EPISODE_RECORDED`, `MEMORY_CONSOLIDATED` | `IDENTITY_UPDATED`, `BELIEF_CHANGED` |

### Pending Processors (from current gap list)

| Processor | Field Domain | Priority | What It Does |
|-----------|-------------|----------|-------------|
| MetaProcessor | `cognition/metacognition.rs` | High | Extracts mental models, assumptions, principles from decisions |
| MoodProcessor | `awareness/mood.rs` | Medium | Estimates valence/energy from episode content |
| EpistemicClassifier | `cognition/epistemics.rs` | Medium | Classifies statements as canonical/belief/hypothesis |
| ThemeExtractor | `awareness/reflection.rs` | Medium | Extracts themes across narratives |
| ValueExtractor | `identity/values.rs` | Medium | Extracts core values from belief patterns |
| PrincipleDistiller | `identity/principles.rs` | Medium | Extracts principles from decisions |
| RoleDetector | `identity/roles.rs` | Low | Detects roles from behavioral patterns |
| PlanDecomposer | `executive/planning.rs` | Low | Decomposes goals into plans with tasks |
| TaskDispatcher | `executive/execution.rs` | Low | Dispatches tasks for execution |
| OutcomeEvaluator | `executive/evaluation.rs` | Low | Evaluates outcomes of completed tasks |
| HealthChecker | `awareness/health.rs` | Low | Monitors subsystem health |
| AnalogyEngine | `cognition/reasoning.rs` | Low | Finds analogies across domains |

### Recommended New Processors (not in current gap list)

| Processor | Field Domain | Priority | What It Does | Why It Matters |
|-----------|-------------|----------|-------------|----------------|
| ContextConstructor | `memory/retrieval.rs` | High | Assembles relevant context for LLM calls | Every processor that calls LLM needs context; this prevents each from building its own |
| SignalObserver | `awareness/observer.rs` | High | Monitors all signal traffic for anomalies | Foundation for Awareness — needed before mood, health, or analytics work |
| LoadMonitor | `awareness/health.rs` | Medium | Tracks processor latency, signal volume | Prevents cascade overload; fed by MetricsCollector |
| SelfModelIntegrator | `identity/self_model.rs` | Medium | Maintains coherence score across identity domains | Ensures identity remains consistent as beliefs/values/traits evolve |
| ConfidenceEstimator | `cognition/metacognition.rs` | Medium | Assigns confidence to all cognitive outputs | Critical for downstream decision-making |
| GoalConflictDetector | `executive/goals.rs` | Medium | Detects conflicting or duplicative goals | Prevents resource waste |
| NarrativeThreader | `awareness/reflection.rs` | Low | Connects narratives across time into life story | Long-term coherence |
| BehavioralDriftDetector | `awareness/analytics.rs` | Low | Detects changes in processor behavior over time | Monitors system health |
| KnowledgeSynthesizer | `cognition/epistemics.rs` | Low | Synthesizes new knowledge from fact patterns | Drives discovery |
| ConceptFormer | `cognition/reasoning.rs` | Low | Forms abstract concepts from concrete examples | Core cognitive ability |

---

## 5. Signal Taxonomy

### Current Signals (17) — Reorganized

All signals follow the convention: `{field}.{domain}.{verb}` using past tense for events.

**Memory Domain:**

| Signal | Current | Notes |
|--------|---------|-------|
| `memory.capture.recorded` | `episode.recorded` | Renamed for domain clarity |
| `memory.capture.ingested` | `ingest.request` | Renamed: input events use past tense |
| `memory.extraction.extracted` | `fact.extracted` | Kept |
| `memory.extraction.triples_extracted` | `triples.extracted` | Kept |
| `memory.consolidation.consolidated` | `memory.consolidated` | Renamed for consistency |
| `memory.consolidation.pattern_detected` | `pattern.detected` | Renamed for domain clarity |
| `memory.maintenance.decayed` | — | New: triggered by decay processor |
| `memory.maintenance.conflict_resolved` | — | New: triggered by conflict resolver |
| `memory.retrieval.queried` | — | New: triggered by retrieval |
| `memory.indexing.updated` | — | New: triggered by indexer |

**Identity Domain:**

| Signal | Current | Notes |
|--------|---------|-------|
| `identity.beliefs.changed` | `belief.changed` | Renamed for domain clarity |
| `identity.traits.detected` | `trait.detected` | Renamed for domain clarity |
| `identity.self.updated` | `identity.updated` | Renamed for domain clarity |
| `identity.values.refined` | — | New |
| `identity.principles.derived` | — | New |
| `identity.roles.detected` | — | New |

**Executive Domain:**

| Signal | Current | Notes |
|--------|---------|-------|
| `executive.goals.created` | `goal.created` | Renamed for domain clarity |
| `executive.goals.completed` | `goal.completed` | Renamed for domain clarity |
| `executive.goals.abandoned` | — | New |
| `executive.planning.created` | — | New |
| `executive.execution.task_created` | — | New |
| `executive.execution.task_completed` | — | New |
| `executive.evaluation.evaluated` | `decision.evaluated` | Re-scoped |

**Awareness Domain:**

| Signal | Current | Notes |
|--------|---------|-------|
| `awareness.attention.shifted` | `attention.shifted` | Kept |
| `awareness.curiosity.detected` | `curiosity.detected` | Kept |
| `awareness.reflection.narrative` | `narrative.generated` | Renamed |
| `awareness.mood.updated` | — | New |
| `awareness.health.status_changed` | — | New |
| `awareness.analytics.pattern_detected` | — | New (moved from consolidation) |

**Cognition Domain:**

| Signal | Current | Notes |
|--------|---------|-------|
| `cognition.reasoning.evaluated` | — | New: output of reasoning |
| `cognition.metacognition.insight` | — | New: meta-cognitive insight |
| `cognition.decision.evaluated` | `decision.evaluated` | Moved here from executive |
| `cognition.mental_models.updated` | — | New |
| `cognition.epistemics.classified` | — | New |

**Knowledge Domain:**

| Signal | Current | Notes |
|--------|---------|-------|
| `knowledge.graph.entity_created` | `entity.created` | Renamed for domain |
| `knowledge.graph.edge_created` | `edge.created` | Renamed for domain |
| `knowledge.resolution.triples_extracted` | `triples.extracted` | Shared with memory.extraction |
| `knowledge.embedding.updated` | — | New |

## 6. Plugin Architecture

### Design

The plugin system uses a registration-based approach, not dynamic loading (for now). Each plugin is a Rust module that implements the `Plugin` trait:

```rust
/// A plugin registers additional processors, signals, jobs, and capabilities.
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str { "0.1.0" }
    
    /// Register processors this plugin provides.
    fn register_processors(&self) -> Vec<Box<dyn Processor>> { vec![] }
    
    /// Register signal types this plugin introduces.
    fn register_signals(&self) -> Vec<(SignalType, &str)> { vec![] }
    
    /// Register scheduled jobs.
    fn register_jobs(&self) -> Vec<ScheduledJob> { vec![] }
    
    /// Register configuration defaults.
    fn register_config(&self) -> Vec<(&str, serde_json::Value)> { vec![] }
    
    /// Register storage backends.
    fn register_storage(&self) -> Vec<Box<dyn StorageExtension>> { vec![] }
    
    /// Register this plugin's capabilities.
    fn register_capabilities(&self) -> Vec<Capability> { vec![] }
}
```

### Auto-Discovery

At startup, the plugin loader scans these paths in order:
1. Built-in plugins (registered in `main.rs`)
2. `/Users/hitensaxena/.noesis/plugins/` (user-installed)
3. `./plugins/` (project-local)
4. Environment variable `NOESIS_PLUGIN_PATH`

Each directory is scanned for `.so`/`.dylib` files (dynamic loading) and `.yaml`/`.json` manifest files (configuration-only plugins).

### Manifest Format

```yaml
# plugin.yaml
name: noesis-knowledge-graph
version: "0.1.0"
description: Knowledge graph plugin
capabilities:
  - entity_resolution
  - graph_query
  - embedding_index
processors:
  - resolution
  - graph_query
storage:
  - postgres_extension
```

## 7. Capability Model

### Design

Every processor advertises what it can do, independent of its location:

```rust
/// A capability is something a processor can do.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Extraction & Analysis
    PatternDetection,
    EntityExtraction,
    RelationExtraction,
    FactExtraction,
    Classification,
    Summarization,
    
    // Reasoning
    Reasoning,
    DecisionMaking,
    Planning,
    ProblemSolving,
    Analogy,
    
    // Memory
    EpisodicMemory,
    SemanticMemory,
    ProceduralMemory,
    WorkingMemory,
    MemoryRetrieval,
    MemoryConsolidation,
    
    // Identity
    BeliefFormation,
    ValueExtraction,
    IdentityIntegration,
    TraitDetection,
    
    // Awareness
    Attention,
    Curiosity,
    MoodEstimation,
    Reflection,
    Narrative,
    HealthMonitoring,
    
    // Knowledge
    GraphQuery,
    EntityResolution,
    Embedding,
    
    // Meta
    Observability,
    Metrics,
    Scheduling,
}
```

### Usage

- The Registry maintains a `HashMap<Capability, Vec<ProcessorId>>` for reverse lookup
- A processor can be discovered by capability: `registry.find_by_capability(Capability::EntityExtraction)`
- The TUI shows capabilities per processor
- Future: the orchestrator uses capabilities to dynamically construct processing pipelines

## 8. Migration Roadmap

### Phase 1: Architecture Cleanup (Preserve everything)

**Goal:** No behavioral changes. Only structural reorganization.

1. Rename `src/core/` → `src/kernel/`
2. Merge `src/eventbus/` content into `src/kernel/`
3. Update all imports across the codebase
4. Create empty field directories with the new structure
5. Processors stay in `src/processors/` until Phase 3
6. Tests must pass without changes

**Files modified:** `src/main.rs`, `src/lib.rs`, all files that import from `core::` or `eventbus::`
**Files created:** Empty field module directories and `mod.rs` files
**Risk:** Low. Pure rename/restructure.
**Verification:** `cargo build` + `cargo test`

### Phase 2: Field Redesign

**Goal:** Fields own their domain structure. State definitions are enhanced.

1. Redesign each field's `state.rs` with the full domain state
2. Add domain dispatch in each field's `mod.rs`:
   - Field receives signal
   - Routes to correct domain file based on signal type
   - Domain file contains its processors
3. Add the new Cognition and Knowledge fields
4. Expand Awareness state (observer, mood, health, analytics)
5. Expand Identity state (values, roles, principles, timeline)

**Files modified:** All 6 field modules + 2 new field modules
**Risk:** Medium. State changes affect serialization.
**Verification:** `cargo build` + inject data + check API endpoints

### Phase 3: Processor Migration

**Goal:** Each processor lives in its owning field's domain file.

1. Move each processor from `src/processors/` to its field domain file
2. Update the `Processor` trait registration path
3. Remove `src/processors/episode.rs` → content goes to `src/fields/memory/capture.rs`
4. Remove `src/processors/extraction.rs` → content goes to `src/fields/memory/extraction.rs`
5. ... etc for all 11 processors
6. Delete empty `src/processors/` directory

**Files modified:** All 11 processor files move; `src/main.rs` registration paths update
**Risk:** Medium-high. Each move needs careful import verification.
**Verification:** `cargo build` + `cargo test` + end-to-end test

### Phase 4: Plugin Architecture

**Goal:** Plugin system works for built-in plugins.

1. Implement the `Plugin` trait
2. Implement `PluginRegistry` with capability indexing
3. Re-register all 11 processors as built-in plugins
4. Implement auto-discovery for `~/.noesis/plugins/`
5. Add capability display to TUI
6. Add capability query to REST API

**Files created:** `src/plugin/` modules
**Files modified:** `src/main.rs` (registration flow)
**Risk:** Medium. Plugin abstraction must not break existing registration.
**Verification:** `cargo build` + `cargo test` + startup with and without plugins

### Phase 5: Advanced Cognition

**Goal:** New processors that implement cognition, mood, epistemics, planning.

1. MetaProcessor (cognition/metacognition.rs) — LLM Deep
2. MoodProcessor (awareness/mood.rs) — LLM Fast
3. EpistemicClassifier (cognition/epistemics.rs) — LLM Agentic
4. PlanDecomposer (executive/planning.rs) — LLM Agentic
5. HealthChecker (awareness/health.rs) — rule-based
6. ContextConstructor (memory/retrieval.rs) — hybrid
7. ConfidentEstimator (cognition/metacognition.rs) — LLM Fast

**Files created:** 7 new processor files
**Risk:** Low-medium. Each is independently testable.
**Verification:** Unit tests per processor + integration test

---

## 9. Key Architectural Trade-offs

| Decision | Option A (Chosen) | Option B (Rejected) | Rationale |
|----------|-------------------|---------------------|-----------|
| **Knowledge as separate field** | Split from Memory | Keep inside Memory | Different state structure (graph vs sequence), different access patterns (traversal vs retrieval), different storage (adjacency vs vector) |
| **Domains as files, not directories** | One file per domain | One directory per domain with sub-files | Avoids file explosion. 2-5 processors per domain fits in one 200-400 line file. |
| **Cognition as new field** | New field between Awareness and Executive | Put reasoning in Executive or Awareness | Reasoning is neither observation (Awareness) nor action (Executive). It has its own state (reasoning chains, mental models) and responsibilities. |
| **Beliefs stay in Identity** | Keep in Identity field | Move to Cognition | Beliefs are self-model, not thinking. Identity answers "what do I believe?" Cognition answers "is this belief true?" Different roles. |
| **Plugin model: registration-first** | Rust trait + manifest files | Dynamic loading via libloading | Dynamic loading is platform-specific, error-prone, and limits what plugins can do. Registration-first keeps plugins as first-class Rust modules that compile together. Dynamic loading can be added later. |
| **Signal renaming: field.domain.verb** | Namespace signals by field.domain | Keep flat names | Prevents signal collision as the system scales to hundreds of signal types. Self-documenting: `memory.capture.recorded` immediately tells you where it comes from. |
| **No orchestrator module** | Cognition emerges from signal cascades | Central orchestrator that sequences processors | The entire architecture is built on the principle that no processor knows the pipeline. An orchestrator would reintroduce central control. The cascade IS the orchestrator. |
| **Awareness is read-only** | Awareness observes, never modifies | Awareness can update other fields | This is the most important architectural constraint. If Awareness could modify memory, there's no check on recursive self-modification. Awareness must be the observer that feeds Cognition, which alone can recommend changes to Executive. |

---

## 10. Cascade Convergence & Equilibrium

The recursive cascade converges through a dampening mechanism:

1. **Depth tracking**: Each signal carries a `depth` counter. Child signals get `parent.depth + 1`.
2. **Max depth**: Hard cap at 50 (current). Soft cap at 10 for most cascades.
3. **Salience gating**: Low-salience signals can be dropped at high depth to prevent runaway cascades.
4. **Signal dedup**: The EventBus can be configured to skip signals that are semantically identical to recently processed ones (by content hash for small signals, by type + entity ID for graph signals).
5. **Equilibrium detection**: When an entire cascade cycle produces zero new signals, the network is at equilibrium. The cascade loop sleeps 50ms before checking for new external signals.
6. **External signal injection**: New `INGEST_REQUEST` signals from REST/MCP/TUI reset the equilibrium counter and start a new cascade cycle.

In practice, a single ingest triggers ~15-30 signals across depth 0-3, and equilibrium is reached within 50-200ms for rule-based processing, or 2-15s with LLM processing (depending on model speed). The cascade reliably converges because every processor produces a finite, bounded number of output signals per input signal, and most output signals match no further processor subscriptions.
