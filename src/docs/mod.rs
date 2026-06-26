//! OpenAPI 3.0 documentation for the Noesis REST API.
//!
//! Provides:
//! - `generate_openapi_spec()` — full OpenAPI 3.0 document as JSON
//! - `swagger_ui_html()` — Swagger UI HTML page served at `/api/docs/`
//!
//! The spec covers all 28+ REST endpoints across the cognitive architecture.
//! Generated programmatically via `serde_json::json!` for maintainability
//! without external OpenAPI dependencies.

use serde_json::{json, Value};

/// Generate the complete OpenAPI 3.0.3 specification document.
///
/// Covers:
/// - Health & system status
/// - Ingest & memory CRUD
/// - Signal types, injection, and history
/// - Knowledge graph queries
/// - Identity & cognition meta
/// - Deep field observability (6 detail views)
/// - Analytics / observability metrics
/// - Session cascade log (SSE)
/// - Capabilities registry
/// - Web dashboard
/// - Plugin management
/// - Documentation endpoints (self-describing)
pub fn generate_openapi_spec() -> Value {
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Noesis Cognitive API",
            "version": "0.1.0",
            "description": "REST API for the Noesis decentralized cognitive architecture.\n\nNoesis models cognition as an emergent decentralized network. Fields own state, Processors transform signals, and Signals are the only communication mechanism. No central controller. No god objects.\n\n## Concepts\n\n- **Fields** — Cognitive domains that own state (memory, identity, agency, action, awareness, reasoning, simulation, graph)\n- **Processors** — Stateless signal transformers subscribed to specific signal types\n- **Signals** — The sole data flow between components, with activation decay for cascading convergence\n",
            "contact": {
                "name": "Noesis Architecture",
                "url": "https://github.com/crazymage21/noesis"
            },
            "license": {
                "name": "MIT"
            }
        },
        "externalDocs": {
            "description": "Architecture Design Records",
            "url": "/api/docs/adrs"
        },
        "servers": [
            {
                "url": "http://localhost:8647",
                "description": "Local development"
            }
        ],
        "paths": {
            // ---- Health ----
            "/api/health": {
                "get": {
                    "summary": "System health check",
                    "operationId": "healthCheck",
                    "tags": ["System"],
                    "responses": {
                        "200": {
                            "description": "System is healthy",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string", "example": "ok" },
                                            "service": { "type": "string", "example": "noesis" },
                                            "version": { "type": "string", "example": "0.1.0" },
                                            "architecture": { "type": "string", "example": "decentralized-signal-cascade" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Ingest ----
            "/api/ingest": {
                "post": {
                    "summary": "Inject raw text into the cognition pipeline",
                    "operationId": "ingestText",
                    "tags": ["Cognition"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/IngestBody"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Text accepted for processing",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string", "example": "accepted" },
                                            "source": { "type": "string", "example": "rest" },
                                            "text_length": { "type": "integer", "example": 42 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Stats ----
            "/api/stats": {
                "get": {
                    "summary": "Full system statistics",
                    "operationId": "getStats",
                    "tags": ["System"],
                    "responses": {
                        "200": {
                            "description": "System statistics including fields, processors, signal types",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "fields": { "type": "integer" },
                                            "processors": { "type": "integer" },
                                            "signal_types": { "type": "integer" },
                                            "field_names": { "type": "array", "items": { "type": "string" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/stats/signals": {
                "get": {
                    "summary": "Per-signal-type count metrics",
                    "operationId": "signalStats",
                    "tags": ["System"],
                    "responses": {
                        "200": {
                            "description": "Signal type frequency map",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "additionalProperties": { "type": "integer" }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Memories ----
            "/api/memories": {
                "get": {
                    "summary": "List all stored memories",
                    "operationId": "listMemories",
                    "tags": ["Memory"],
                    "responses": {
                        "200": {
                            "description": "Memory field state or empty placeholder",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "field": { "type": "string", "example": "memory" },
                                            "state": { "type": "object" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create a memory directly",
                    "operationId": "createMemory",
                    "tags": ["Memory"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/CreateMemoryBody"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Memory created",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string", "example": "created" },
                                            "source": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/memory/recall": {
                "get": {
                    "summary": "Search episodes by content",
                    "operationId": "recallMemories",
                    "tags": ["Memory"],
                    "parameters": [
                        {
                            "name": "q",
                            "in": "query",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "Search query"
                        },
                        {
                            "name": "k",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "integer", "default": 10, "maximum": 100 },
                            "description": "Maximum results (max 100)"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Matching episodes",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "query": { "type": "string" },
                                            "matches": { "type": "integer" },
                                            "results": { "type": "array" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/episodes": {
                "get": {
                    "summary": "List recorded episodes",
                    "operationId": "listEpisodes",
                    "tags": ["Memory"],
                    "responses": {
                        "200": {
                            "description": "Episodes from field state cache",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "episodes_from_cache": { "type": "object" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Graph ----
            "/api/graph": {
                "get": {
                    "summary": "Knowledge graph state overview",
                    "operationId": "getGraph",
                    "tags": ["Knowledge Graph"],
                    "responses": {
                        "200": {
                            "description": "Graph state with entities and relations",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/graph/sources": {
                "get": {
                    "summary": "Entity counts by source episode",
                    "operationId": "graphSources",
                    "tags": ["Knowledge Graph"],
                    "responses": {
                        "200": {
                            "description": "Source-to-count map",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/graph/expand": {
                "get": {
                    "summary": "Expand entity connections",
                    "operationId": "expandEntity",
                    "tags": ["Knowledge Graph"],
                    "parameters": [
                        {
                            "name": "entity",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "string" },
                            "description": "Entity name to expand"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Entity with connected relations",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },

            // ---- Identity ----
            "/api/identity": {
                "get": {
                    "summary": "Current identity state",
                    "operationId": "getIdentity",
                    "tags": ["Identity"],
                    "responses": {
                        "200": {
                            "description": "Beliefs, traits, self-model",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "identity": { "type": "object" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Cognition ----
            "/api/cognition/meta": {
                "get": {
                    "summary": "Principles, assumptions, mental models",
                    "operationId": "cognitionMeta",
                    "tags": ["Cognition"],
                    "responses": {
                        "200": {
                            "description": "Meta view from field cache",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/cognition/reflection": {
                "get": {
                    "summary": "Reflection reports",
                    "operationId": "cognitionReflection",
                    "tags": ["Cognition"],
                    "responses": {
                        "200": {
                            "description": "Reflection output",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/cognition/narrative": {
                "get": {
                    "summary": "Generated narratives",
                    "operationId": "cognitionNarrative",
                    "tags": ["Cognition"],
                    "responses": {
                        "200": {
                            "description": "Narrative state",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },

            // ---- Signals ----
            "/api/signals": {
                "get": {
                    "summary": "List all registered signal types",
                    "operationId": "listSignalTypes",
                    "tags": ["Signals"],
                    "responses": {
                        "200": {
                            "description": "Signal types with descriptions",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "signal_types": {
                                                "type": "array",
                                                "items": {
                                                    "type": "object",
                                                    "properties": {
                                                        "type": { "type": "string" },
                                                        "description": { "type": "string" }
                                                    }
                                                }
                                            },
                                            "count": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/signals/inject": {
                "post": {
                    "summary": "Inject an arbitrary signal into the bus",
                    "operationId": "injectSignal",
                    "tags": ["Signals"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/InjectSignalBody"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Signal injected",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string", "example": "injected" },
                                            "signal_type": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Unknown signal type",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "error": { "type": "string" },
                                            "known_types": { "type": "array", "items": { "type": "string" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/signals/history": {
                "get": {
                    "summary": "Recent signal history from event store",
                    "operationId": "signalHistory",
                    "tags": ["Signals"],
                    "parameters": [
                        {
                            "name": "from_seq",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "integer", "default": 1 }
                        },
                        {
                            "name": "limit",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "integer", "default": 50, "maximum": 500 }
                        },
                        {
                            "name": "event_type",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "string" },
                            "description": "Filter by event type prefix"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Paginated signal history",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "signals": { "type": "array" },
                                            "count": { "type": "integer" },
                                            "total": { "type": "integer" },
                                            "from_seq": { "type": "integer" },
                                            "limit": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Deep Detail Views ----
            "/api/identity/detail": {
                "get": {
                    "summary": "Deep identity observability",
                    "operationId": "identityDetail",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Full identity breakdown with beliefs, traits, values, roles, history",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/memory/detail": {
                "get": {
                    "summary": "Deep memory observability",
                    "operationId": "memoryDetail",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Full memory system breakdown (working, episodic, semantic, procedural, graph, retrieval, consolidation, indexing)",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/agency/detail": {
                "get": {
                    "summary": "Deep agency observability",
                    "operationId": "agencyDetail",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Full agency system breakdown (goals, projects, tasks, plans, pursuits, strategy)",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/awareness/detail": {
                "get": {
                    "summary": "Deep awareness observability",
                    "operationId": "awarenessDetail",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Full awareness system breakdown",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/simulation/detail": {
                "get": {
                    "summary": "Deep simulation observability",
                    "operationId": "simulationDetail",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Full simulation system breakdown",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/core/detail": {
                "get": {
                    "summary": "Deep core system observability",
                    "operationId": "coreDetail",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Full runtime infrastructure breakdown (event_bus, scheduler, registry, plugin_loader, kernel, runtime, config, metrics, permissions)",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },

            // ---- Observability ----
            "/api/observability/overview": {
                "get": {
                    "summary": "System-wide observability summary",
                    "operationId": "observabilityOverview",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Uptime, field/processor/signal counts, signal metrics",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/observability/signals": {
                "get": {
                    "summary": "Per-signal-type metrics",
                    "operationId": "observabilitySignals",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Signal type frequency map",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/observability/processors": {
                "get": {
                    "summary": "Per-processor latency metrics",
                    "operationId": "observabilityProcessors",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Processor latency statistics",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },
            "/api/observability/cascade": {
                "get": {
                    "summary": "Cascade trace (recent cascade events)",
                    "operationId": "observabilityCascade",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Recent cascade trace events",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        }
                    }
                }
            },

            // ---- Capabilities ----
            "/api/capabilities": {
                "get": {
                    "summary": "List all registered capabilities from plugins",
                    "operationId": "listCapabilities",
                    "tags": ["System"],
                    "responses": {
                        "200": {
                            "description": "Capabilities with providers",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "capabilities": {
                                                "type": "array",
                                                "items": {
                                                    "type": "object",
                                                    "properties": {
                                                        "id": { "type": "string" },
                                                        "available": { "type": "boolean" },
                                                        "providers": {
                                                            "type": "array",
                                                            "items": {
                                                                "type": "object",
                                                                "properties": {
                                                                    "name": { "type": "string" },
                                                                    "description": { "type": "string" },
                                                                    "confidence": { "type": "number" },
                                                                    "processor": { "type": "string" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            },
                                            "total": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Docs ----
            "/api/docs/openapi.json": {
                "get": {
                    "summary": "OpenAPI 3.0 specification",
                    "operationId": "getOpenApiSpec",
                    "tags": ["Documentation"],
                    "responses": {
                        "200": {
                            "description": "The complete OpenAPI specification",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/docs/": {
                "get": {
                    "summary": "Swagger UI documentation browser",
                    "operationId": "getSwaggerUi",
                    "tags": ["Documentation"],
                    "responses": {
                        "200": {
                            "description": "Swagger UI HTML page",
                            "content": {
                                "text/html": {
                                    "schema": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- SSE Events ----
            "/api/events/stream": {
                "get": {
                    "summary": "Real-time cascade log stream (SSE)",
                    "operationId": "eventStream",
                    "tags": ["Events"],
                    "responses": {
                        "200": {
                            "description": "Server-Sent Events stream of signal activity",
                            "content": {
                                "text/event-stream": {
                                    "schema": {
                                        "type": "string",
                                        "description": "SSE formatted data lines (event: signal, data: {JSON})"
                                    }
                                }
                            }
                        }
                    }
                }
            },

            // ---- Dashboard ----
            "/api/dashboard/": {
                "get": {
                    "summary": "Web dashboard for field inspection",
                    "operationId": "getDashboard",
                    "tags": ["Dashboard"],
                    "responses": {
                        "200": {
                            "description": "HTML dashboard page",
                            "content": {
                                "text/html": {
                                    "schema": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },

        "components": {
            "schemas": {
                "IngestBody": {
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Raw text to inject into the cognition pipeline",
                            "example": "I went for a run in the park today."
                        },
                        "source": {
                            "type": "string",
                            "description": "Source label for attribution",
                            "example": "rest",
                            "default": "rest"
                        }
                    }
                },
                "CreateMemoryBody": {
                    "type": "object",
                    "required": ["content"],
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Memory content",
                            "example": "Remembered that I need to buy groceries"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source label"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional tags for categorization"
                        }
                    }
                },
                "InjectSignalBody": {
                    "type": "object",
                    "required": ["signal_type"],
                    "properties": {
                        "signal_type": {
                            "type": "string",
                            "description": "Signal type to inject",
                            "example": "memory.capture.ingested"
                        },
                        "payload": {
                            "type": "object",
                            "description": "Optional signal payload fields",
                            "example": { "text": "Hello from API" }
                        }
                    }
                }
            },
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key",
                    "description": "API key authentication. Set NOESIS_API_KEY env var to enable."
                },
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "API Key",
                    "description": "Bearer token authentication using NOESIS_API_KEY"
                }
            }
        },

        "security": [
            { "ApiKeyAuth": [] },
            { "BearerAuth": [] }
        ],

        "tags": [
            { "name": "System", "description": "Health, stats, and system-level endpoints" },
            { "name": "Cognition", "description": "Text ingestion and cognitive pipeline" },
            { "name": "Memory", "description": "Episodic and semantic memory CRUD" },
            { "name": "Knowledge Graph", "description": "Entity and relation graph queries" },
            { "name": "Identity", "description": "Beliefs, traits, and self-model" },
            { "name": "Signals", "description": "Signal type listing, injection, and history" },
            { "name": "Observability", "description": "Deep field detail and system metrics" },
            { "name": "Events", "description": "Real-time event streaming (SSE)" },
            { "name": "Dashboard", "description": "Web UI panels" },
            { "name": "Documentation", "description": "API documentation endpoints" }
        ]
    })
}

/// Generate the Swagger UI HTML page.
///
/// Loads Swagger UI from CDN and points it at `/api/docs/openapi.json`.
pub fn swagger_ui_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Noesis API — Swagger UI</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
  <style>
    html { box-sizing: border-box; overflow-y: scroll; }
    *, *::before, *::after { box-sizing: inherit; }
    body { margin: 0; background: #fafafa; }
    .topbar { display: none; }
    .information-container .info .title { color: #1a1a2e; }
    .scheme-container { background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,0.1); }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/api/docs/openapi.json',
      dom_id: '#swagger-ui',
      presets: [
        SwaggerUIBundle.presets.apis,
        SwaggerUIBundle.SwaggerUIStandalonePreset
      ],
      layout: 'BaseLayout',
      deepLinking: true,
      showExtensions: true,
      showCommonExtensions: true,
      defaultModelExpandDepth: 3,
      docExpansion: 'list'
    });
  </script>
</body>
</html>"#.to_string()
}
