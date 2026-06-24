# Noesis — CLAUDE.md

## Project identity
- This is a **research project** exploring decentralized cognitive architecture.
- NO code from `~/curlyos-core/` should be copied or adapted here — this is a from-first-principles redesign.
- The architecture philosophy is documented in `DESIGN.md` — read it before making architectural changes.

## Key rules
- Fields never call each other directly — communicate through signals only.
- Processors never invoke other processors — subscribe to signals, emit new signals.
- Signals are the ONLY data flow between cognitive components.
- All state lives in fields; processors are stateless transformations.
- The kernel owns lifecycle and wiring — no god objects.
- Tests live next to implementation files, not in a separate directory.

## Signal-first thinking
- When adding a new capability: define the signals first, then the processor that handles them, then the field that stores the resulting state.
- Signals propagate recursively until the network reaches equilibrium (no processor emits a new signal in response to the cascade).
- Sonnet 4.6 for routine implementation, Haiku for lookups, Opus for architecture decisions.
