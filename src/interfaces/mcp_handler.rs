//! MCP server — exposes Noesis cognitive capabilities to external AI agents.
//!
//! Serves JSON-RPC 2.0 requests on a dedicated endpoint (:8645 by default).
//! Supports the Model Context Protocol tool-calling surface:
//!
//!   POST /mcp  { jsonrpc: "2.0", method: "tools/list", ... }
//!   POST /mcp  { jsonrpc: "2.0", method: "tools/call", params: { name: "recall", arguments: {...} } }

use std::sync::Arc;
use axum::{Json, extract::State, routing::post, Router};
use serde_json::Value;

use crate::interfaces::rest::ApiState;
use crate::interfaces::mcp::{JSONRPCRequest, JSONRPCResponse, noesis_tools};
use crate::signals::IngestRequest;

/// Build the MCP router.
pub fn mcp_router() -> Router<ApiState> {
    Router::new()
        .route("/mcp", post(handle_mcp))
}

/// Single handler for all JSON-RPC methods.
async fn handle_mcp(
    State(state): State<ApiState>,
    Json(req): Json<JSONRPCRequest>,
) -> Json<JSONRPCResponse> {
    let id = req.id.clone();

    match req.method.as_str() {
        "tools/list" => {
            Json(JSONRPCResponse::success(id, serde_json::json!({
                "tools": noesis_tools(),
            })))
        }
        "tools/call" => {
            let tool_name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let default_args = serde_json::json!({});
            let args = req.params.get("arguments").unwrap_or(&default_args);

            match tool_name {
                "recall" => {
                    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                    let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                    let memory_state = state.field_cache.get("memory");
                    match memory_state {
                        Some(cache) => {
                            let episodes = cache.value().get("episodes")
                                .and_then(|e| e.as_array()).cloned().unwrap_or_default();
                            let q = query.to_lowercase();
                            let matches: Vec<_> = episodes.iter()
                                .filter(|ep| {
                                    ep.get("content").and_then(|c| c.as_str()).unwrap_or("").to_lowercase().contains(&q)
                                })
                                .take(k)
                                .cloned()
                                .collect();
                            Json(JSONRPCResponse::success(id, serde_json::json!({
                                "matches": matches.len(),
                                "results": matches,
                            })))
                        }
                        None => Json(JSONRPCResponse::success(id, serde_json::json!({
                            "matches": 0, "results": [], "note": "No memory state cached"
                        }))),
                    }
                }
                "inject" => {
                    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("mcp");
                    let episode = IngestRequest::new(text, source);
                    state.event_bus.publish(Arc::new(episode));
                    Json(JSONRPCResponse::success(id, serde_json::json!({
                        "status": "injected",
                        "text": text.chars().take(60).collect::<String>(),
                    })))
                }
                "field_state" => {
                    let field = args.get("field").and_then(|v| v.as_str()).unwrap_or("");
                    match state.field_cache.get(field) {
                        Some(cache) => Json(JSONRPCResponse::success(id, serde_json::json!({
                            "field": field,
                            "state": cache.value(),
                        }))),
                        None => Json(JSONRPCResponse::error(id, -32000, &format!("Field '{}' not found in cache", field))),
                    }
                }
                "capabilities" => {
                    let caps: Vec<Value> = state.capability_registry.list().iter().filter_map(|id| {
                        let providers = state.capability_registry.find_providers(id);
                        if providers.is_empty() { None }
                        else {
                            Some(serde_json::json!({
                                "id": id,
                                "providers": providers.iter().map(|c| serde_json::json!({
                                    "name": c.name, "processor": c.processor, "confidence": c.confidence,
                                })).collect::<Vec<_>>(),
                            }))
                        }
                    }).collect();
                    Json(JSONRPCResponse::success(id, serde_json::json!({ "capabilities": caps })))
                }
                _ => Json(JSONRPCResponse::error(id, -32601, &format!("Unknown tool: {}", tool_name))),
            }
        }
        _ => Json(JSONRPCResponse::error(id, -32601, &format!("Unknown method: {}", req.method))),
    }
}
