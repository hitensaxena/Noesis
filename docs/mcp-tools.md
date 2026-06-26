# MCP Server — Tool Documentation

> **Endpoint:** `POST /mcp` on port :8645 (start with `cargo run -- start --mcp`)  
> **Protocol:** JSON-RPC 2.0  
> **Purpose:** External AI agents (Claude, GPT, etc.) interact with the Noesis cognitive architecture

---

## Overview

The MCP (Model Context Protocol) server exposes Noesis's cognitive capabilities as callable tools. Each tool maps to a cognitive operation: memory recall, experience injection, field state inspection, capability discovery, and signal injection.

### Calling convention

All requests are `POST /mcp` with `Content-Type: application/json`:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "recall",
    "arguments": { "query": "running in the park", "k": 5 }
  }
}
```

---

## Tools

### 1. `recall`

Search episodic memory for experiences matching a query.

**Input schema:**
| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `query` | string | ✅ | — | Search query (case-insensitive content match) |
| `k` | integer | ❌ | 10 | Maximum number of results |

**Example request:**
```json
{
  "jsonrpc": "2.0", "id": 1, "method": "tools/call",
  "params": { "name": "recall", "arguments": { "query": "running", "k": 3 } }
}
```

**Example response:**
```json
{
  "jsonrpc": "2.0", "id": 1,
  "result": {
    "matches": 2,
    "results": [
      { "content": "I went for a run in the park...", "source": "demo", "tags": [] },
      { "content": "Morning run was great today", "source": "rest", "tags": [] }
    ]
  }
}
```

**Error cases:**
- No episodes stored → `matches: 0, results: []`
- Empty query → matches everything (up to `k`)

---

### 2. `inject`

Inject a raw experience into the cognitive system. Triggers the full signal cascade (memory consolidation → belief formation → goal creation → narrative generation).

**Input schema:**
| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `text` | string | ✅ | — | Experience text to inject |
| `source` | string | ❌ | `"mcp"` | Source label for attribution |

**Example request:**
```json
{
  "jsonrpc": "2.0", "id": 2, "method": "tools/call",
  "params": { "name": "inject", "arguments": { "text": "Completed the quarterly review presentation", "source": "claude" } }
}
```

**Example response:**
```json
{
  "jsonrpc": "2.0", "id": 2,
  "result": { "status": "injected", "text": "Completed the quarterly review presentation" }
}
```

**Notes:**
- The injection is asynchronous — the signal cascade propagates in the background
- Track results by observing signals via SSE endpoint (`/api/events/stream`) or querying `/api/memory/detail`

---

### 3. `field_state`

Read the current state snapshot of any field.

**Input schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field` | string | ✅ | One of: `memory`, `identity`, `agency`, `action`, `awareness`, `reasoning`, `simulation`, `knowledge_graph` |

**Example request:**
```json
{
  "jsonrpc": "2.0", "id": 3, "method": "tools/call",
  "params": { "name": "field_state", "arguments": { "field": "identity" } }
}
```

**Example response:**
```json
{
  "jsonrpc": "2.0", "id": 3,
  "result": {
    "field": "identity",
    "state": {
      "identity_version": 3,
      "beliefs": [{ "text": "I am a runner", "confidence": 0.85 }],
      "traits": ["active", "curious"]
    }
  }
}
```

**Error cases:**
- Unknown field name → `error: { code: -32000, message: "Field 'xyz' not found in cache" }`
- Empty field name → same error

---

### 4. `capabilities`

List all registered capabilities in the system. Each capability has an ID, one or more providers (processors that implement it), and a confidence score.

**Input schema:** `{}` (no arguments)

**Example request:**
```json
{
  "jsonrpc": "2.0", "id": 4, "method": "tools/call",
  "params": { "name": "capabilities", "arguments": {} }
}
```

**Example response:**
```json
{
  "jsonrpc": "2.0", "id": 4,
  "result": {
    "capabilities": [
      {
        "id": "memory.recall",
        "providers": [{ "name": "retrieval", "processor": "RetrievalProcessor", "confidence": 0.8 }]
      }
    ]
  }
}
```

---

### 5. `inject_signal`

Inject an arbitrary cognitive signal by type. Allows direct access to the signal bus for testing and custom integrations.

**Input schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `signal_type` | string | ✅ | Signal type string (e.g. `agency.goals.created`, `awareness.curiosity.detected`) |

**Example request:**
```json
{
  "jsonrpc": "2.0", "id": 5, "method": "tools/call",
  "params": { "name": "inject_signal", "arguments": { "signal_type": "awareness.curiosity.detected" } }
}
```

**Example response:**
```json
{
  "jsonrpc": "2.0", "id": 5,
  "result": { "status": "injected", "signal_type": "awareness.curiosity.detected" }
}
```

**Error cases:**
- Unknown signal type → `error: { code: -32601, message: "Unknown tool: ..." }`

---

## Protocol-Level Methods

### `tools/list`

Returns the full list of tools with their input schemas. Used by AI agents to discover available capabilities.

**Example request:**
```json
{
  "jsonrpc": "2.0", "id": 0, "method": "tools/list"
}
```

**Example response:**
```json
{
  "jsonrpc": "2.0", "id": 0,
  "result": {
    "tools": [
      { "name": "recall", "description": "Search episodic memory...", "input_schema": { ... } },
      { "name": "inject", "description": "Inject a raw experience...", "input_schema": { ... } },
      { "name": "field_state", "description": "Read the current state snapshot...", "input_schema": { ... } },
      { "name": "capabilities", "description": "List all registered capabilities...", "input_schema": { ... } },
      { "name": "inject_signal", "description": "Inject an arbitrary cognitive signal...", "input_schema": { ... } }
    ]
  }
}
```

---

## Error Codes

| Code | Meaning |
|------|---------|
| `-32601` | Method not found (unknown tool or RPC method) |
| `-32000` | Field not found in cache (field_state tool) |

---

## Quick Start

```bash
# Start Noesis with MCP server on port 8645
cargo run -- start --mcp

# In another terminal, list available tools
curl -s -X POST http://127.0.0.1:8645/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":0,"method":"tools/list"}' | jq .

# Inject an experience
curl -s -X POST http://127.0.0.1:8645/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"inject","arguments":{"text":"Had a great coffee this morning"}}}' | jq .
```
