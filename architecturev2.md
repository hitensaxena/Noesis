# Noesis Architecture v1.0 — Frozen

## Core Philosophy

Noesis is a Cognitive Operating System.

It is not an AI assistant.
It is not an agent framework.
It is not a memory system.
It is not a workflow engine.

It is a persistent cognitive organism whose intelligence emerges from decentralized recursive computation over persistent cognitive state.

Every architectural decision preserves this philosophy.

---

## Immutable Principles

### Fields remember.
Fields own persistent cognitive state. Fields never perform computation. Fields never invoke one another. Each field answers exactly one fundamental cognitive question. Fields are cognitive spaces, not software modules.

### Processors transform.
Processors are temporary computations. They subscribe to signals, read field state, compute, update field state, and emit new signals. They never directly invoke another processor. One processor performs one cognitive transformation. Persistent cognition always belongs to fields.

### Signals propagate.
Signals are cognitive activations, not commands and not API calls. They recursively propagate until activation naturally dissipates.

### Kernel sustains.
The Kernel provides runtime, scheduling, routing, plugin loading, lifecycle, and metrics. The Kernel never performs cognition.

---

## Final Cognitive Questions

| Field | Question |
|-------|----------|
| **Memory** | What do I remember? |
| **Identity** | Who am I? |
| **Agency** | What do I want? |
| **Action** | What am I doing? |
| **Awareness** | What am I noticing? |
| **Reasoning** | What do I conclude? |
| **Simulation** | What could happen? |

Every field answers exactly one question. No field answers two. Every cognitive function maps to exactly one field.

---

## Final Field Hierarchy

```
Kernel
  │
  Signal Mesh (kernel signals)
  │
Field Runtime
  │
  ├── Memory        (What do I remember?)
  ├── Identity      (Who am I?)
  ├── Agency        (What do I want?)       ← renamed from Intention
  ├── Action        (What am I doing?)
  ├── Awareness     (What am I noticing?)   ← READ-ONLY
  ├── Reasoning     (What do I conclude?)
  └── Simulation    (What could happen?)
  │
  Signal Mesh (cognitive signals)
```

**7 fields. Each answers one question. No overlaps.**

---

## 1. Repository Structure

### Principle: Separate runtime structure from cognitive structure.

Every field has exactly three layers:

```
field_name/
  state.rs          All persistent state types for this field
  domains/          Organizational grouping of state (one file per domain)
  processors/       One file per processor (one transformation per file)
```

Domains are organizational only. They are not runtime objects. Processors are the only active runtime entities within a field.

### Final directory structure:

```
src/
├── main.rs
├── lib.rs
├── bin/noesis_tui.rs
│
├── kernel/                          # Sustains — never performs cognition
│   ├── mod.rs
│   ├── event_bus.rs                 # Signal routing (kernel + cognitive)
│   ├── signal.rs                    # Signal with activation envelope
│   ├── registry.rs                  # Fields, Processors, Signals, Capabilities
│   ├── scheduler.rs                 # BeatCoordinator
│   ├── runtime.rs                   # Task management
│   ├── lifecycle.rs                 # Startup/shutdown
│   ├── plugin.rs                    # Plugin trait + PluginLoader
│   ├── metrics.rs                   # Metrics
│   └── state.rs                     # SystemState for interfaces
│
├── field_runtime/                   # Bridge between Kernel and Fields
│   ├── mod.rs
│   ├── context.rs                   # FieldContext — the processor's environment
│   ├── dispatcher.rs                # Routes signals to field processors
│   ├── snapshot.rs                  # Field state snapshot + cache
│   └── transactions.rs              # State transactions + rollback
│
├── fields/
│   ├── mod.rs
│   │
│   ├── memory/                      # WHAT DO I REMEMBER?
│   │   ├── mod.rs                   # MemoryField
│   │   ├── state.rs                 # All memory state types
│   │   ├── domains/
│   │   │   ├── working.rs           # Working memory structures
│   │   │   ├── episodic.rs          # Episode structures
│   │   │   ├── semantic.rs          # Semantic memory structures
│   │   │   ├── procedural.rs        # Procedural memory structures
│   │   │   └── knowledge.rs         # Knowledge graph entities, relations, embeddings
│   │   └── processors/
│   │       ├── episode_processor.rs # Ingest → EpisodeRecorded
│   │       ├── extraction_processor.rs # Episode → Triples
│   │       ├── resolution_processor.rs  # Triples → Entities/Edges
│   │       ├── consolidation_processor.rs   # Episodes → Semantic memories
│   │       ├── retrieval_processor.rs   # Query → Results
│   │       ├── context_processor.rs   # Retrieval → Context window
│   │       ├── indexing_processor.rs  # Entity → Embedding index
│   │       ├── decay_processor.rs     # Time-based memory decay
│   │       └── dedup_processor.rs     # Duplicate detection + resolution
│   │
│   ├── identity/                    # WHO AM I?
│   │   ├── mod.rs                   # IdentityField
│   │   ├── state.rs                 # Identity state types
│   │   ├── domains/
│   │   │   ├── self_model.rs        # Core identity record
│   │   │   ├── beliefs.rs           # Belief structures
│   │   │   ├── values.rs            # Value structures
│   │   │   ├── traits.rs            # Trait structures
│   │   │   ├── roles.rs             # Role structures
│   │   │   ├── personality.rs       # Personality profile
│   │   │   ├── principles.rs        # Principle structures
│   │   │   ├── timeline.rs          # Identity timeline
│   │   │   ├── evolution.rs         # Identity projection
│   │   │   └── narrative_self.rs    # Life narrative structures
│   │   └── processors/
│   │       ├── belief_processor.rs  # Memory → Beliefs
│   │       ├── value_processor.rs   # Decisions → Values
│   │       ├── trait_processor.rs   # Beliefs → Traits
│   │       ├── principle_processor.rs   # Decisions → Principles
│   │       └── self_integrator.rs   # All signals → Coherent self
│   │
│   ├── agency/                      # WHAT DO I WANT? (renamed from Intention)
│   │   ├── mod.rs                   # AgencyField
│   │   ├── state.rs                 # Agency state types
│   │   ├── domains/
│   │   │   ├── goals.rs             # Goal structures
│   │   │   ├── priorities.rs        # Priority queue structures
│   │   │   ├── strategy.rs          # Strategic plan structures
│   │   │   ├── opportunities.rs     # Opportunity structures
│   │   │   └── purpose.rs           # Mission/purpose statements
│   │   └── processors/
│   │       ├── goal_processor.rs    # Identity → Goals
│   │       ├── priority_processor.rs    # Goal changes → Priorities
│   │       ├── strategy_processor.rs    # Slow beat → Strategy
│   │       └── opportunity_processor.rs # Curiosity → Opportunities
│   │
│   ├── action/                      # WHAT AM I DOING?
│   │   ├── mod.rs                   # ActionField
│   │   ├── state.rs                 # Action state types
│   │   ├── domains/
│   │   │   ├── projects.rs          # Project structures
│   │   │   ├── plans.rs             # Plan structures
│   │   │   ├── tasks.rs             # Task structures
│   │   │   ├── executions.rs        # Execution state
│   │   │   ├── evaluations.rs       # Outcome history
│   │   │   └── risk.rs              # Risk register
│   │   └── processors/
│   │       ├── project_processor.rs # Goals → Projects
│   │       ├── plan_processor.rs    # Goals/Projects → Plans
│   │       ├── task_processor.rs    # Plans → Tasks
│   │       ├── execution_processor.rs   # Tasks → Execution
│   │       ├── evaluation_processor.rs  # Execution → Evaluation
│   │       └── risk_processor.rs    # Plans → Risk assessment
│   │
│   ├── awareness/                   # WHAT AM I NOTICING? (READ-ONLY)
│   │   ├── mod.rs                   # AwarenessField
│   │   ├── state.rs                 # Awareness state types
│   │   ├── domains/
│   │   │   ├── observer.rs          # State transition log
│   │   │   ├── attention.rs         # Focus stack, salience
│   │   │   ├── curiosity.rs         # Knowledge gap map
│   │   │   ├── open_loops.rs        # Open loop tracker
│   │   │   ├── mood.rs              # Mood history
│   │   │   ├── health.rs            # Health metrics
│   │   │   └── analytics.rs         # Signal rates, patterns
│   │   └── processors/
│   │       ├── observer_processor.rs    # All signals → State transitions
│   │       ├── attention_processor.rs   # Signals → Focus shifts
│   │       ├── curiosity_processor.rs   # Episodes → Knowledge gaps
│   │       ├── narrative_processor.rs   # Episode clusters → Narrative
│   │       ├── reflection_processor.rs  # Episode batches → Reflection
│   │       ├── mood_processor.rs        # Episode content → Mood estimate
│   │       ├── health_processor.rs      # Metrics → Health status
│   │       └── pattern_processor.rs     # Analytics → Pattern detection
│   │
│   ├── reasoning/                    # WHAT DO I CONCLUDE?
│   │   ├── mod.rs                    # ReasoningField
│   │   ├── state.rs                  # Reasoning state types
│   │   ├── domains/
│   │   │   ├── reasoning.rs          # Reasoning chain structures
│   │   │   ├── mental_models.rs      # Mental model graph
│   │   │   ├── metacognition.rs      # Confidence/insight structures
│   │   │   ├── decisions.rs          # Decision register
│   │   │   ├── hypotheses.rs         # Hypothesis space
│   │   │   ├── analogies.rs          # Analogy map
│   │   │   ├── epistemics.rs         # Epistemic classifications
│   │   │   ├── synthesis.rs          # Synthesized knowledge
│   │   │   └── concepts.rs           # Concept hierarchy
│   │   └── processors/
│   │       ├── reasoning_processor.rs    # Evidence → Conclusions
│   │       ├── mental_model_processor.rs # Patterns → Models
│   │       ├── metacognition_processor.rs    # All → Confidence/insight
│   │       ├── decision_processor.rs    # Choices → Decisions
│   │       ├── hypothesis_processor.rs  # Patterns → Hypotheses
│   │       ├── analogy_processor.rs     # New concepts → Analogies
│   │       ├── epistemic_processor.rs   # Facts → Epistemic class
│   │       ├── synthesis_processor.rs   # Related facts → Knowledge
│   │       └── concept_processor.rs     # Recurring patterns → Concepts
│   │
│   └── simulation/                   # WHAT COULD HAPPEN?
│       ├── mod.rs                    # SimulationField
│       ├── state.rs                  # Simulation state types
│       ├── domains/
│       │   ├── world_models.rs       # Causal model structures
│       │   ├── assumptions.rs        # Assumption register
│       │   ├── scenarios.rs          # Scenario space
│       │   ├── counterfactuals.rs    # What-if cache
│       │   ├── forecasting.rs        # Prediction records
│       │   └── risk.rs               # Risk simulation structures
│       └── processors/
│           ├── world_model_processor.rs   # Episodes → Causal models
│           ├── assumption_processor.rs    # Strategy → Assumptions
│           ├── scenario_processor.rs      # Decision points → Scenarios
│           ├── counterfactual_processor.rs # Outcomes → What-ifs
│           ├── forecasting_processor.rs   # Goals → Forecasts
│           └── risk_processor.rs          # Plans → Risk simulations
│
├── engines/                           # External intelligence
│   ├── llm/                           # Tiered LLM (Fast/Agentic/Deep)
│   └── graph/                         # Graph extraction utilities
│
├── interfaces/                        # External access — emit signals into the organism
│   ├── rest/                          # REST API (emit cognitive signals)
│   ├── tui/                           # Terminal UI (observe + emit)
│   ├── cli.rs                         # CLI (emit signals)
│   └── mcp_types.rs                   # MCP protocol types
│
└── storage/                           # Persistence backends
    ├── store.rs
    ├── memory_store.rs
    ├── event_store.rs
    └── backends/
```

---

## 2. Signal Model

### Two categories: Kernel Signals + Cognitive Signals

Both use the same signal struct. They are separated only by namespace and conceptual ownership.

### Signal struct:

```rust
struct Signal {
    // Cognitive propagation
    activation: f32,       // 0.0-1.0 (decreases per hop, guarantees convergence)
    salience: f32,         // 0.0-1.0 (importance)
    novelty: f32,          // 0.0-1.0 (surprise)
    confidence: f32,       // 0.0-1.0 (certainty of emitter)
    decay: f32,            // per-hop multiplier (typical: 0.7)
    
    // Metadata
    id: Uuid,
    correlation_id: Option<Uuid>,
    signal_type: SignalType,
    timestamp: DateTime<Utc>,
    source: String,
    depth: u32,
    
    // Payload
    payload: Arc<dyn Any + Send + Sync>,
}
```

### Kernel Signals (the operating system):

```
kernel.runtime.started
kernel.runtime.stopping
kernel.plugin.loaded
kernel.plugin.failed
kernel.scheduler.beat.immediate
kernel.scheduler.beat.fast
kernel.scheduler.beat.medium
kernel.scheduler.beat.slow
kernel.scheduler.beat.sleep
kernel.scheduler.beat.offline
kernel.processor.registered
kernel.processor.failed
kernel.storage.ready
kernel.metrics.threshold_exceeded
```

### Cognitive Signals (the organism):

```
memory.capture.recorded
memory.capture.ingested
memory.extraction.triples_extracted
memory.extraction.fact_extracted
memory.consolidation.consolidated
memory.consolidation.pattern_detected
memory.knowledge.entity_created
memory.knowledge.edge_created
memory.retrieval.requested
memory.retrieval.completed
memory.maintenance.decayed
memory.maintenance.conflict_resolved
memory.context.assembled
memory.indexing.updated

identity.beliefs.changed
identity.values.refined
identity.traits.detected
identity.roles.detected
identity.principles.derived
identity.self.updated
identity.timeline.recorded
identity.narrative.updated

agency.goals.created
agency.goals.completed
agency.goals.abandoned
agency.priorities.reordered
agency.strategy.updated
agency.opportunity.detected

action.projects.created
action.projects.completed
action.planning.plan_ready
action.tasks.created
action.tasks.completed
action.execution.started
action.execution.completed
action.execution.failed
action.evaluation.evaluated
action.risk.assessed
action.recovery.started

awareness.attention.shifted
awareness.curiosity.detected
awareness.reflection.insight
awareness.reflection.narrative
awareness.mood.estimated
awareness.health.status_changed
awareness.analytics.pattern_detected
awareness.drift.detected
awareness.open_loops.updated
awareness.themes.extracted
awareness.observer.transition_detected

reasoning.conclusion.ready
reasoning.mental_models.updated
reasoning.metacognition.insight
reasoning.decision.made
reasoning.hypothesis.generated
reasoning.analogy.detected
reasoning.epistemics.classified
reasoning.synthesis.ready
reasoning.concept.formed

simulation.world_model.updated
simulation.assumption.tested
simulation.scenario.ready
simulation.counterfactual.ready
simulation.forecast.ready
simulation.risk.assessed
```

---

## 3. Processor Model

```rust
#[async_trait]
trait Processor: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "" }
    
    // What this processor does (for capability discovery)
    fn capabilities(&self) -> Vec<Capability>;
    
    // Signal routing
    fn subscribed_signals(&self) -> &[SignalType];
    fn emitted_signals(&self) -> &[SignalType];
    
    // Attention economy
    fn activation_threshold(&self) -> f32 { 0.1 }
    fn priority(&self) -> u8 { 100 }
    
    // Lifecycle — runs inside FieldContext
    async fn init(&mut self, ctx: &FieldContext) -> Result<()> { Ok(()) }
    async fn process(&mut self, ctx: &FieldContext, signal: Signal) -> Result<Vec<Signal>>;
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
```

**Rules enforced by convention and review:**
- One processor = one cognitive transformation
- Reads field state through FieldContext
- Updates field state through FieldContext
- Emits new cognitive signals
- Never invokes another processor
- May keep ephemeral runtime state (counters, caches)
- Persistent state always belongs to the field (written to state.rs)

---

## 4. FieldContext Design

FieldContext is the processor's local cognitive environment. Processors operate inside this context — they should never touch Tokio, the EventBus directly, or any kernel infrastructure.

```rust
/// The local cognitive environment for all processors within a field.
/// Processors operate inside this context. They never touch the kernel directly.
#[derive(Clone)]
struct FieldContext {
    /// Field identification
    field_name: &'static str,
    
    /// Field state access
    state: Arc<RwLock<FieldState>>,
    
    /// Signal emission (the ONLY way to communicate)
    emitter: SignalEmitter,
    
    /// Capability lookup (find other processors by what they do)
    capabilities: Arc<CapabilityRegistry>,
    
    /// Scheduler beat subscription
    beats: BeatSubscriber,
    
    /// Metrics recording
    metrics: MetricsCollector,
    
    /// Storage access
    storage: Arc<dyn Storage>,
    
    /// Logger with field context pre-populated
    log: FieldLogger,
}
```

**Design principles:**
- FieldContext is the ONLY dependency injection a processor receives
- It provides everything a processor needs without exposing kernel internals
- Processors are independently testable by mocking FieldContext
- The `emitter` field is how processors emit signals — they never call `event_bus.publish()` directly

---

## 5. Field Runtime Design

Inserted between Kernel and Fields. The Kernel knows nothing about individual fields. Fields know nothing about the Kernel. Field Runtime bridges them.

```
Kernel                    (sustains — pure infrastructure)
  │
Field Runtime             (bridges — knows both kernel and fields)
  │
Fields                    (cognition — fields remember, processors transform)
```

### Field Runtime responsibilities:

```rust
struct FieldRuntime {
    /// 1. State loading: Hydrate field state from storage on startup
    state_loader: StateLoader,
    
    /// 2. Signal dispatch: Route incoming signals to the correct field's processors
    dispatcher: SignalDispatcher,
    
    /// 3. Snapshots: Periodically serialize field state for REST API + persistence
    snapshot: SnapshotManager,
    
    /// 4. Transactions: Wrap processor execution in atomic state changes
    transactions: TransactionManager,
    
    /// 5. Rollback: Revert field state if a processor fails mid-transaction
    rollback: RollbackManager,
    
    /// 6. Local scheduling: Beat subscriptions for field-local processors
    scheduler: LocalScheduler,
}
```

### Why this layer exists:

Without Field Runtime, the Kernel would need to know:
- Which fields exist
- What state each field needs
- How to serialize each field's state
- How to handle processor failures per field

This is knowledge the Kernel should not have. Field Runtime encapsulates it.

### Implementation note:

Field Runtime should NOT be a complex framework. Start as a thin dispatcher that routes signals to fields by signal_type prefix:
- `memory.*` → MemoryField dispatcher
- `identity.*` → IdentityField dispatcher
- etc.

Add snapshot management and transactions only when processor failure scenarios emerge through real usage. Do not overengineer upfront.

---

## 6. Public API Philosophy

### Internal (within the organism):

Fields own state. Processors modify state through signals. No other mechanism exists for internal cognitive change.

### External (interfaces):

Interfaces (REST, CLI, TUI, mobile, desktop) emit signals into the organism. They never invoke fields directly and never modify field state directly.

```
User → REST API → emit(memory.capture.ingested) → Signal Mesh → processors → field update
```

The organism never receives imperative commands. It receives input signals that propagate through the same cognitive cascade as every other signal. This ensures:

1. External inputs follow the same propagation rules as internal signals
2. External inputs are subject to the same activation/salience/novelty filtering
3. There is no privileged external access path that bypasses cognition

### Public read APIs:

Fields expose read-only APIs for external interfaces:

```rust
impl MemoryField {
    fn recall(&self, query: &str, k: usize) -> Result<Vec<MemoryItem>>;
    fn context(&self, query: &str) -> Result<ContextWindow>;
}
impl IdentityField {
    fn self_model(&self) -> SelfModel;
    fn beliefs(&self) -> Vec<Belief>;
}
```

These are the ONLY external APIs. Write operations always go through signals.

---

## 7. Rename Intention → Agency

**Decision: Rename to Agency.**

Rationale:
- "Intention" describes a single concept (goals). It undersells the field's scope.
- "Agency" encompasses goals, priorities, motivation, purpose, and autonomy — all the things that answer "What do I want?"
- In cognitive science, agency is the capacity to act with intention and purpose.
- The word "Agency" creates a clean cognitive pairing with "Action": Agency decides what to pursue, Action pursues it.

All prior references to Intention are replaced with Agency in this document.

---

## 8. Plugin Architecture

### Design:

```rust
#[async_trait]
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "" }
    
    // Registration
    fn processors(&self) -> Vec<Box<dyn Processor>> { vec![] }
    fn signals(&self) -> Vec<(SignalType, &str)> { vec![] }
    fn capabilities(&self) -> Vec<Capability> { vec![] }
    fn config_defaults(&self) -> Vec<(&str, Value)> { vec![] }
    fn scheduled_jobs(&self) -> Vec<ScheduledJob> { vec![] }
    fn storage(&self) -> Vec<Box<dyn StorageExtension>> { vec![] }
}
```

### Auto-discovery (future):

Scan these paths at startup:
1. Built-in plugins (registered in main.rs)
2. `~/.noesis/plugins/*/plugin.yaml`
3. `./plugins/*/plugin.yaml`
4. `$NOESIS_PLUGIN_PATH`

### Future evolution:

The Plugin trait is designed so `fn fields() -> Vec<Box<dyn Field>>` can be added in a future version without breaking existing plugins. This enables plugins that register entire cognitive fields. Do not implement this yet — design for compatibility.

---

## 9. Capability Model

String-based, dynamically extensible (not an enum):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Capability {
    id: String,           // e.g., "entity_extraction"
    name: String,         // e.g., "Entity Extraction"
    description: String,
    confidence: f32,      // 0.0-1.0
    processor: String,    // processor name that provides this
}
```

### Canonical capabilities (by convention):

| ID | Description |
|----|-------------|
| `reasoning` | Logical reasoning and inference |
| `observation` | Observing state transitions |
| `pattern_detection` | Detecting recurring patterns |
| `prediction` | Predicting outcomes |
| `retrieval` | Retrieving stored information |
| `extraction` | Extracting structured data from text |
| `planning` | Decomposing goals into plans |
| `simulation` | Running what-if scenarios |
| `classification` | Classifying inputs into categories |
| `summarization` | Summarizing content |
| `decision_making` | Making decisions from evidence |
| `analogy` | Finding analogies |
| `synthesis` | Synthesizing new knowledge |
| `translation` | Translating between representations |

Capabilities enable dynamic processor discovery. Registry maintains a reverse index: `HashMap<String, Vec<ProcessorId>>`.

---

## 10. Scheduler Design

### Cognitive Beats (not timers):

| Beat | Period | Signals Emitted | Example Consumers |
|------|--------|-----------------|-------------------|
| **Immediate** | Every signal | `kernel.scheduler.beat.immediate` | Working memory, attention |
| **Fast** | ~1s | `kernel.scheduler.beat.fast` | Extraction, identity updates |
| **Medium** | ~60s | `kernel.scheduler.beat.medium` | Curiosity, reflection, planning |
| **Slow** | ~15min | `kernel.scheduler.beat.slow` | Narrative, identity evolution |
| **Sleep** | Session idle | `kernel.scheduler.beat.sleep` | Consolidation, model stabilization |
| **Offline** | On demand | `kernel.scheduler.beat.offline` | Re-indexing, optimization |

### Implementation:

```rust
struct BeatCoordinator {
    last_fast: Instant,
    last_medium: Instant,
    last_slow: Instant,
    last_sleep: Option<Instant>,
}

// Spawned on kernel startup
// Emits beat signals that processors subscribe to
// Processors replace "if count % N == 0" with beat subscriptions
```

---

## 11. Kernel Responsibilities (Final)

```
Kernel
├── EventBus           # Signal routing (kernel + cognitive)
│   ├── Kernel signals namespace
│   └── Cognitive signals namespace
├── Registry           # Field registration, Processor registration, Signal types, Capabilities
├── Scheduler          # BeatCoordinator (see above)
├── Runtime            # Tokio task management, CancellationToken
├── Lifecycle          # Ordered startup: Kernel → Scheduler → Plugins → Field Runtime → Fields
├── PluginLoader       # Plugin trait registration + auto-discovery
├── Metrics            # Per-signal counters, per-processor latency, field state sizes
└── SystemState        # FieldStateCache for REST API reads
```

**The Kernel never performs cognition.** It provides the environment. "Cognition" is the entire layered stack above it.

---

## 12. Migration Roadmap (6 Phases)

### Phase 0: Activation Signal Model (~2 days)
- Add activation, salience, novelty, confidence, decay to Signal struct
- Default values for backward compatibility (activation=1.0, decay=0.7)
- Add activation_threshold to Processor trait (default: 0.1)
- Cascade convergence uses activation decay, not max_depth cap
- **No behavioral changes. All tests pass.**

### Phase 1: Structural Restructure (~2 weeks)
- Create `kernel/` directory, merge `core/` + `eventbus/` into it
- Create `field_runtime/` directory with context.rs + dispatcher.rs
- Create `fields/` with new domain/processor sub-structure
- Split Executive → Agency + Action
- Move existing processor code to field directories
- **No behavioral changes. All tests pass.**

### Phase 2: Field Runtime (~1 week)
- Implement SignalDispatcher (routes by signal_type prefix to field)
- Implement FieldContext (state, emitter, capabilities, metrics, storage, log)
- Move processor registration from kernel to field_runtime
- Verify all 11 processors work through FieldRuntime
- **No behavioral changes. All tests pass.**

### Phase 3: Cognitive Beats (~1 week)
- Implement BeatCoordinator
- Add beat signal types to kernel signal namespace
- Migrate processors from hard-coded counters to beat subscriptions
- **No behavioral changes. All tests pass.**

### Phase 4: Plugin System (~1 week)
- Implement Plugin trait + PluginRegistry
- Register all processors as built-in plugins
- Implement manifest discovery for `~/.noesis/plugins/`
- Add capability query to REST API + TUI
- **Duration:** ~1 week

### Phase 5: New Processors (~4-6 weeks)
Build in priority order (each independently testable):
1. ObserverProcessor (awareness root — foundation for all awareness)
2. MetaProcessor (metacognitive insight)
3. MoodProcessor (mood from episode content)
4. ContextConstructor (reasoning context assembly)
5. EpistemicClassifier (statement classification)
6. ConfidenceEstimator (confidence scoring)
7. PlanDecomposer (goal → plan)
8. HealthChecker (subsystem monitoring)
9. ValueExtractor (decisions → values)
10. PrincipleDistiller (decisions → principles)

---

## 13. Future Fields (Design Compatibility)

These fields should be addable without architectural changes. Do NOT implement them yet. Ensure the architecture supports them:

| Future Field | Question | Natural Extension |
|-------------|----------|-------------------|
| Learning | How do I improve? | New field + processors that observe outcomes and adjust behavior |
| Social | Who am I with? | Plugin registering field + processors for multi-agent interaction |
| Communication | How do I express? | Plugin registering signal output processors |
| ScientificDiscovery | What do I discover? | Reasoning sub-field for hypothesis testing |
| CreativeReasoning | What do I create? | Simulation sub-field for novel combination |
| Finance | What do I own? | Agency sub-field for resource management |
| Vision | What do I see? | Separate input pipeline feeding Memory capture |
| HealthField | How do I feel? | Awareness sub-field for embodied monitoring (separate from system health) |

---

## 14. Architecture Freeze

From this point forward:

- **Stop redesigning.** The architecture is complete.
- **Stop introducing foundational abstractions.** No more layers. No more new concepts.
- **Begin restructuring the repository.** Phase 0 implementation starts.
- **Migrate existing code incrementally.** Each phase builds on the previous.
- **Implement pending processors.** Phase 5 processors are the primary feature work.
- **Write tests.** Every processor must be independently testable.
- **Measure performance.** Cascade depth, signal throughput, processor latency.
- **Evolve through implementation.** Future changes must be justified by real experience, not hypothetical needs.

This document is the architectural constitution of Noesis.
