# ADR-002: SSE for Real-Time Cascade Monitoring

## Status

Accepted (2026-06-26)

## Context

The Noesis REST API needed a real-time endpoint that streams signal activity from the EventBus. The options were:

1. **Server-Sent Events (SSE)** — HTTP-based unidirectional event stream
2. **WebSockets** — Bidirectional persistent connection
3. **Polling** — Client periodically fetches `/api/signals/history`
4. **gRPC Server Streaming** — Requires gRPC infrastructure

## Decision

We chose **Server-Sent Events**:

`GET /api/events/stream` returns a `text/event-stream` response. Each SSE event is a JSON payload describing a signal published on the EventBus. The connection stays open indefinitely, with keep-alive pings every 15 seconds.

The SSE endpoint receives events through a dedicated broadcast channel (`event_stream_tx: Option<broadcast::Sender<String>>` in `ApiState`). A background task in `main.rs` subscribes to all signal types on the EventBus and forwards formatted JSON strings to this channel.

## Rationale

- **Simple protocol** — SSE uses standard HTTP with no upgrade handshake; works with `curl`, browser `EventSource`, and any HTTP client
- **axum-native** — implemented via `axum::body::Body::from_stream()` without external dependencies beyond `tokio-stream` and `async-stream`
- **Directional** — cascade monitoring is read-only; the web dashboard uses REST for signal injection, not the event stream
- **Automatic reconnection** — browsers auto-reconnect on SSE disconnect
- **No dependency** — axum's WebSocket support requires the `ws` feature and additional handling; SSE is purely HTTP

## Consequences

- SSE is unidirectional (server → client); signal injection goes through `POST /api/signals/inject`
- Browsers limit SSE connections (typically 6 per domain)
- Long-lived connections increase server resource usage; each SSE client gets one broadcast receiver
- 15-second keepalive prevents proxy/gateway timeouts
- The broadcast channel has 1024-slot capacity; slow clients that don't read will miss events (lagged → `event: lag` notification)
