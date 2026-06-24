# Noesis

**A decentralized cognitive architecture — emergent intelligence through recursive signal propagation.**

Most AI systems are built as collections of modules: Memory, Reflection, Identity, Goals, Knowledge Graph. These modules call each other through APIs in a mostly top-down architecture.

Noesis rejects that model.

Instead, it models cognition as an emergent decentralized network, inspired by biological neural systems, predictive processing, Global Workspace Theory, the Actor Model, and Advaita Vedanta.

There is no central "brain" controlling the system. No master cognition service. No module responsible for "thinking." Intelligence emerges from thousands of tiny computations interacting through signals — exactly as biological neural systems do.

## Architecture

Noesis is organized around three concepts:

**Fields** — persistent cognitive spaces that own state. Fields never call each other directly.

**Processors** — tiny autonomous workers, each performing exactly one cognitive transformation. Processors never invoke other processors directly.

**Signals** — the language of the organism. Everything communicates through signals. Signals propagate recursively until the network reaches equilibrium.

No processor knows the complete processing pipeline. No field knows what other fields exist. The final cognitive state is emergent.

## Quick Start

```bash
cargo build
cargo run -- start
cargo run -- inject "I went for a run in the park"
```

## Guiding Question

Every architectural decision answers: *Does this make the organism more capable of evolving through decentralized recursive cognition, or does it reintroduce centralized software architecture?*
