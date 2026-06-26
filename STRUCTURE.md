Current Structure

# Layer 1 - Kernel & Infra - /core
* EventBus Lifecycle
* Runtime (CancellationToken)
* Registry (DashMap fields/processors/signals)
* Ordered Lifecycle Hookd
* FieldStateCache + SystemState

# Layer 2 - Event Bus - /eventbus
* Tokio Broadcast Channels per Signal Type
* MPSC Fan-In
* CloudEvents v1.0 envelope
* 56-event closed catalog
* Arc-based Signal Dispatch

# Layer 3 - Signals - /signals
* Awareness
* Executive
* Graph
* Identity
* Memory
* Mod

# Layer 4 - Fields - /fields
* MemoryField
* IdentityField
* ExecutiveField
* AwarenessField
* SimulationField
* GraphField

# Layer 5 - Processors - /processors
* Episode
* Extraction
* Resolution
* Consolidation
* Belief
* Identity
* Goal
* Narrative
* Attention
* Curiosity
* Reflection

# Layer 6 - LLM Engine - /engines/llm
* LLMClient
* ModelChain
* TieredRouter (Fast, Agentic, Deep)

# Layer 7 - Knowledge Graph - /engines/graph
* Entity/Relation types
* EntityCategory + RelationType enums
* LLM triple extraction system prompt

# Layer 8 - Storage - /storage/
* Storage trait
* MemoryStore
* EventStore + MemoryEventStore
* CompositeStorage
* Auto-connect to curlyos-core containers

# Layer 9 - REST API - /interfaces/rest/
* Health
* Ingest
* Stats
* Signals
* Memories
* Episodes
* Graph
* Identity
* Cognition
* 6 Deep Detail endpoints : identity, executive, memory, awareness, simulation, core
* Observability - overview, signal metrics, processor metrics, cascade trace.

# Layer 10 - TUI - /tui/
* Dashboard - Live field summaries + stats
* Signals - types + bus info
* Fields - with per-field detail data
* Processors - pipeline diagram + dispatch metrics
* Observability - runtime + config + processor metrics
* Log - color-coded entries + info panel
* Detail - 6-field deep view with sub-nav
* Settings - connection + refresh + system info
* Auto-refresh toggle, interval control, detail cycling, screen navigation

# Layer 11 - CLI - /interfaces/cli
* 5 subcommands - start, inject, inspect, list, plugins

# Layer 12 - Hermes Plugin - /hermes_integration
* Noesis_client - all noesis api endpoints
* Noesis_plugin - memoryprovider with 3 tools (recall, add, search)

# Layer 13 - MCP Server - /mcp_server
* FastMCP SSE server on :8645
* HTTP client to Noesis API, 5 tools

# Layer 14 - Metrics - /metrics
* MetricsCollector with per-signal-type counters and per-processor latency histogram


## Pending Work
* MetaProcessor
* MoodProcessor
* ReflectionField
* EpistemicProcessor
* ThemesProcessor
* Evaluation/Studio Fields
* Orchestrator
* Plugin System auto-scanning at startup
* API auth
* Hybrid Retrieval (Semantic memory retrival)
* Event Persistence.
