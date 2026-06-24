# Noesis — Design Philosophy

## Why Not Modules?

Most AI cognitive architectures organize code the way we organize software: modules, services, APIs, dependency injection, layered architectures. A "Memory Module" calls a "Reflection Module" which calls a "Goal Module" — all wired through a central orchestrator.

This is comfortable because it mirrors how we build web services. But it is not how cognition works.

The brain has no central orchestrator. It has no "memory module" that returns results to a "thinking module." It has billions of neurons, each performing a trivial computation, communicating through electrochemical signals. Intelligence is an **emergent property** of these local interactions — not a feature of any individual component.

Noesis takes this seriously as an architectural constraint: **if you can point at any single component and say "that's where the thinking happens," the architecture has failed.**

## The Three Primitives

### 1. Fields

Fields are persistent cognitive spaces. Each field owns a slice of the organism's state.

- The **Memory Field** owns episodic and semantic memories.
- The **Identity Field** owns beliefs, traits, and the self-model.
- The **Executive Field** owns goals and active intentions.
- The **Awareness Field** owns the current focus and salience map.
- The **Simulation Field** owns what-if scenarios.

Fields are **isolated**. They never call each other. They never know other fields exist. They have one way to learn about the world: signals delivered to them. They have one way to affect the world: signals they emit back.

New fields can be added without changing existing architecture. Examples of future fields: Emotion, Learning, Social, Finance, Health, Creative, Vision, Motor, Robot.

### 2. Processors

Processors are tiny autonomous workers. Each performs exactly one cognitive transformation.

- The **Episode Processor** converts raw experience into structured episodes.
- The **Belief Processor** extracts beliefs from patterns in memories.
- The **Identity Processor** integrates beliefs into the self-model.
- The **Narrative Processor** weaves episodes into coherent stories.
- The **Goal Processor** manages goal lifecycle from creation to completion.
- The **Attention Processor** computes salience and shifts focus.
- The **Curiosity Processor** detects knowledge gaps.

A processor:
1. Subscribes to specific signal types.
2. When a signal arrives, reads local field state (from the fields it's attached to).
3. Performs its transformation.
4. Emits zero or more new signals.
5. Returns to rest.

Processors never invoke other processors. They never call field methods directly. Their sole interaction mechanism is the signal bus.

### 3. Signals

Signals are the language of the organism. Everything communicates through signals.

```
Experience
    ↓
EpisodeRecorded ──────────→ [Memory Field updates]
    ↓
Signal broadcast ─────────→ [Interested processors activate]
    ↓
BeliefProcessor fires ────→ BeliefChanged signal
    ↓
Signal broadcast ─────────→ [More processors activate]
    ↓
IdentityProcessor fires ───→ IdentityUpdated signal
    ↓
...propagation continues...
    ↓
Eventually no processor fires → Network reaches equilibrium
```

The final cognitive state — what the organism "thinks" about an experience — is the emergent result of this recursive propagation, not the output of any single component.

## Design Principles

**No central controller.** The kernel only starts and stops components. It never routes, decides, or transforms data.

**No god objects.** No single struct knows about all components. The kernel holds references but never acts on them intelligently.

**No module dependencies.** A processor does not import another processor. A field does not import another field. The signal bus is the only dependency.

**Event-first architecture.** Define the signals before the processors. Define the processors before the fields. The flow of signals defines the system's behavior.

**Signals over function calls.** If component A needs component B to do something, A emits a signal. B may or may not be subscribed. It may or may not act. The system is resilient to any component being absent.

**Local computation only.** A processor only reads its attached field's state. It never queries other fields. It never makes remote calls. All knowledge is local; global coherence is emergent.

**Recursive information propagation.** Signals ripple through the network in cascades. Each ripple may change state and produce new signals. The cascade ends when no processor emits a new signal — the network has reached equilibrium.

**Eventually consistent cognitive state.** There is no transaction, no atomic update across fields. Each field converges in its own time. The system as a whole is always moving toward coherence but never perfectly synchronized.

**Plugin-first design.** Everything new is a plugin. A plugin only registers: processors, signals, configuration schema, storage requirements, and scheduled jobs. No core changes required.

**Replaceable engines.** The LLM engine, embedding engine, retrieval engine, ranking engine, graph engine — all pluggable. The architecture does not depend on any specific AI provider.

## What This Means for Code

The code should feel strange if you approach it with traditional software instincts.

- **No service layer.** There is no `MemoryService` that coordinates memory operations. Memory operations happen because signals arrive at the Memory Field.
- **No orchestrator.** There is no `CognitionOrchestrator` that decides when to reflect, when to consolidate, when to update identity. Decisions emerge from signal cascades.
- **No `if` chains routing by type.** A processor does not have a `match` statement checking signal types. It subscribes to the signals it cares about; the event bus handles routing.
- **No `async` request-response between components.** Components never wait for each other. A processor fires and forgets. The cascade handles the rest.

## The Long-Term Vision

Noesis should become a **cognitive operating system** rather than a traditional software application.

CurlyOS — the production system — will eventually become one application running on top of the Noesis cognitive engine.

The architecture should be capable of growing from today's memory, identity, and reflection capabilities into:
- Curiosity and attention as autonomous drives
- Creativity and dreaming as recombination engines
- Emotions as valenced state evaluations
- Social reasoning as multi-agent simulation
- Robotics as sensorimotor coupling
- Autonomous scientific discovery as hypothesis generation and testing

All without requiring architectural redesign. Each new capability is a plugin registering new signals, processors, and fields.

## Inspirations

- Biological neural networks — local computation, distributed representation, plasticity
- Cortical feedback loops — reciprocal signaling between processing layers
- Predictive processing — top-down predictions meet bottom-up sensory signals
- Global Workspace Theory — conscious content is what's in the global broadcast
- Active Inference — organisms minimize free energy by updating beliefs and acting
- Event Sourcing — state is the sum of past events, never overwritten
- Actor Model — everything is an actor, communicating through messages
- Cellular Automata — complex global behavior from simple local rules
- Swarm Intelligence — decentralized agents producing emergent order
- Complex Adaptive Systems — systems that adapt through local interactions
- Advaita Vedanta — the observer is not the observed; identification is a process, not a position
- Systems Thinking — behavior emerges from structure and feedback loops

## The Guiding Question

Every architectural decision answers:

**"Does this make the organism more capable of evolving through decentralized recursive cognition, or does it reintroduce centralized software architecture?"**

If the answer is the latter, the design is wrong.
