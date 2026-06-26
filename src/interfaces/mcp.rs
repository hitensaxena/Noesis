//! MCP (Model Context Protocol) — external AI agent integration.
//!
//! Defines protocol types for LLM tool calling and a lightweight JSON-RPC
//! server that exposes Noesis's cognitive capabilities to external AI agents
//! (Claude, GPT, etc.). Each tool maps to a cognitive operation:
//!
//! - `recall`: Search episodic memory
//! - `inject`: Inject a raw experience → triggers full cascade
//! - `field_state`: Read any field's current state snapshot
//! - `capabilities`: List all registered capabilities
//! - `inject_signal`: Inject an arbitrary cognitive signal
//!
//! Run with: `cargo run -- start --mcp` (adds :8645 endpoint)

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// MCP Protocol Types (based on Model Context Protocol spec)
// ---------------------------------------------------------------------------

/// A tool that an LLM can call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// A resource that an LLM can read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// A prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<MCPPromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

// ---------------------------------------------------------------------------
// JSON-RPC message types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONRPCRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONRPCResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JSONRPCError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONRPCError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JSONRPCResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0".to_string(), id, result: Some(result), error: None }
    }

    pub fn error(id: Value, code: i32, message: &str) -> Self {
        Self { jsonrpc: "2.0".to_string(), id, result: None, error: Some(JSONRPCError { code, message: message.to_string(), data: None }) }
    }
}

// ---------------------------------------------------------------------------
// Tool definitions for Noesis
// ---------------------------------------------------------------------------

/// Returns the list of MCP tools available for external AI agents.
pub fn noesis_tools() -> Vec<MCPTool> {
    vec![
        MCPTool {
            name: "recall".to_string(),
            description: "Search episodic memory for experiences matching a query".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"},
                    "k": {"type": "integer", "description": "Max results (default 10)", "default": 10}
                },
                "required": ["query"]
            }),
        },
        MCPTool {
            name: "inject".to_string(),
            description: "Inject a raw experience into the cognitive system — triggers full signal cascade".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "Experience text"},
                    "source": {"type": "string", "description": "Source label (default 'mcp')", "default": "mcp"}
                },
                "required": ["text"]
            }),
        },
        MCPTool {
            name: "field_state".to_string(),
            description: "Read the current state snapshot of any field".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "field": {
                        "type": "string",
                        "enum": ["memory", "identity", "agency", "action", "awareness", "reasoning", "simulation", "knowledge_graph"],
                        "description": "Field name"
                    }
                },
                "required": ["field"]
            }),
        },
        MCPTool {
            name: "capabilities".to_string(),
            description: "List all registered capabilities in the system".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
        MCPTool {
            name: "inject_signal".to_string(),
            description: "Inject an arbitrary cognitive signal by type".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "signal_type": {"type": "string", "description": "Signal type string (e.g. goal.created)"},
                },
                "required": ["signal_type"]
            }),
        },
    ]
}

/// Maps MCP tool method names to their schema definitions for the `tools/list` response.
pub fn tools_list_response() -> Value {
    serde_json::json!(noesis_tools())
}
