//! RTCO MCP Server — exposes compression and analysis as MCP tools.
//!
//! Implements a minimal JSON-RPC over stdin/stdout MCP server with three tools:
//! - `rtco_compress` — compress CLI output
//! - `rtco_analyze` — analyze CLI output without compressing
//! - `rtco_retrieve` — retrieve previous compression results by ID
//!
//! # Protocol
//!
//! Each line on stdin is a JSON-RPC request.  Each request produces one JSON-RPC
//! response line on stdout.  Stderr is reserved for diagnostics.
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"rtco_compress","arguments":{"content":"..."}}}
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use rtco_core::content_detector;
use rtco_core::utils;

// ---------------------------------------------------------------------------
// JSON-RPC types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: serde_json::Value,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }

    fn notification() -> Self {
        Self {
            jsonrpc: "2.0",
            id: serde_json::Value::Null,
            result: None,
            error: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

/// Descriptor returned by `tools/list`.
#[derive(Serialize)]
struct ToolDescription {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

fn list_tools() -> Vec<ToolDescription> {
    vec![
        ToolDescription {
            name: "rtco_compress".into(),
            description: "Compress CLI output to reduce LLM token consumption. Detects content type automatically and applies the best compression strategy.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Raw CLI output to compress"
                    }
                },
                "required": ["content"]
            }),
        },
        ToolDescription {
            name: "rtco_analyze".into(),
            description: "Analyze CLI output without compressing it. Returns content type, token estimate, and redundancy metrics.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "CLI output to analyze"
                    }
                },
                "required": ["content"]
            }),
        },
        ToolDescription {
            name: "rtco_retrieve".into(),
            description: "Retrieve a previous compression result by its ID. IDs are returned by rtco_compress.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Compression result ID returned by rtco_compress"
                    }
                },
                "required": ["id"]
            }),
        },
    ]
}

// ---------------------------------------------------------------------------
// In-memory result store
// ---------------------------------------------------------------------------

type ResultStore = std::sync::Mutex<HashMap<String, serde_json::Value>>;

fn store_result(store: &ResultStore, id: String, result: serde_json::Value) {
    if let Ok(mut guard) = store.lock() {
        guard.insert(id, result);
        // Keep at most 1024 entries
        while guard.len() > 1024 {
            let first = guard.keys().next().cloned();
            if let Some(k) = first {
                guard.remove(&k);
            }
        }
    }
}

fn retrieve_result(store: &ResultStore, id: &str) -> Option<serde_json::Value> {
    store.lock().ok().and_then(|guard| guard.get(id).cloned())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn handle_compress(content: &str) -> serde_json::Value {
    let original_tokens = utils::count_tokens(content);
    let content_type = content_detector::detect_content_type(content);

    // Simple compression logic:
    // - Strip ANSI codes
    // - Remove blank lines (for build output / logs)
    // For now this is a heuristic; future versions will use ContentRouter.
    let stripped = utils::strip_ansi(content);
    let compressed: String = match content_type {
        content_detector::ContentType::BuildOutput
        | content_detector::ContentType::SearchResults => stripped
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => {
            // Keep output mostly as-is, just strip ANSI
            stripped
        }
    };

    let compressed_tokens = utils::count_tokens(&compressed);
    let savings = if original_tokens > 0 {
        ((original_tokens - compressed_tokens) as f64 / original_tokens as f64 * 100.0 * 100.0)
            .round()
            / 100.0
    } else {
        0.0
    };

    serde_json::json!({
        "compressed": compressed,
        "original_tokens": original_tokens,
        "compressed_tokens": compressed_tokens,
        "savings_percent": savings,
        "content_type": format!("{:?}", content_type),
    })
}

fn handle_analyze(content: &str) -> serde_json::Value {
    let total_lines = content.lines().count();
    let non_empty_lines = content.lines().filter(|l| !l.trim().is_empty()).count();
    let content_type = content_detector::detect_content_type(content);
    let estimated_tokens = utils::count_tokens(content);

    serde_json::json!({
        "detected_type": format!("{:?}", content_type),
        "line_count": total_lines,
        "non_empty_line_count": non_empty_lines,
        "estimated_tokens": estimated_tokens,
        "character_count": content.len(),
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Try to extract a string argument from the tool-call params.  Returns
/// `None` if the key is missing or not a string.
fn get_string_arg<'a>(
    params: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    params.get(key).and_then(|v| v.as_str())
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

fn main() {
    let result_store: ResultStore = std::sync::Mutex::new(HashMap::new());
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut result_counter: u64 = 0;

    // Send server info notification on startup (optional, MCP spec allows it)
    let server_info = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "server/info",
        "params": {
            "name": "rtco-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "RTCO MCP Server — compress and analyze CLI output for LLM context efficiency"
        }
    });
    {
        let mut out = stdout.lock();
        let line = serde_json::to_string(&server_info).unwrap_or_default();
        writeln!(out, "{line}").ok();
        out.flush().ok();
    }

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("rtco-mcp: stdin error: {e}");
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err = JsonRpcResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {e}"),
                );
                let out = serde_json::to_string(&err).unwrap_or_default();
                println!("{out}");
                continue;
            }
        };

        let response: JsonRpcResponse = match request.method.as_str() {
            "tools/list" => {
                let result = serde_json::json!({ "tools": list_tools() });
                JsonRpcResponse::success(request.id, result)
            }
            "tools/call" => {
                // Extract params map; error and continue on failure.
                let params = match request.params.as_object() {
                    Some(p) => p,
                    None => {
                        let err =
                            JsonRpcResponse::error(request.id, -32602, "Invalid params".into());
                        let out = serde_json::to_string(&err).unwrap_or_default();
                        println!("{out}");
                        continue;
                    }
                };

                let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => {
                        let err =
                            JsonRpcResponse::error(request.id, -32602, "Missing tool name".into());
                        let out = serde_json::to_string(&err).unwrap_or_default();
                        println!("{out}");
                        continue;
                    }
                };

                let arguments = params
                    .get("arguments")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();

                match tool_name {
                    "rtco_compress" => {
                        let content = match get_string_arg(&arguments, "content") {
                            Some(c) => c,
                            None => {
                                let err = JsonRpcResponse::error(
                                    request.id,
                                    -32602,
                                    "Missing 'content' argument".into(),
                                );
                                let out = serde_json::to_string(&err).unwrap_or_default();
                                println!("{out}");
                                continue;
                            }
                        };
                        let result = handle_compress(content);
                        let result_id = {
                            result_counter += 1;
                            format!("{result_counter:x}")
                        };
                        store_result(&result_store, result_id.clone(), result.clone());
                        JsonRpcResponse::success(
                            request.id,
                            serde_json::json!({
                                "content": result,
                                "result_id": result_id,
                            }),
                        )
                    }
                    "rtco_analyze" => {
                        let content = match get_string_arg(&arguments, "content") {
                            Some(c) => c,
                            None => {
                                let err = JsonRpcResponse::error(
                                    request.id,
                                    -32602,
                                    "Missing 'content' argument".into(),
                                );
                                let out = serde_json::to_string(&err).unwrap_or_default();
                                println!("{out}");
                                continue;
                            }
                        };
                        let result = handle_analyze(content);
                        JsonRpcResponse::success(request.id, result)
                    }
                    "rtco_retrieve" => {
                        let id = match get_string_arg(&arguments, "id") {
                            Some(i) => i,
                            None => {
                                let err = JsonRpcResponse::error(
                                    request.id,
                                    -32602,
                                    "Missing 'id' argument".into(),
                                );
                                let out = serde_json::to_string(&err).unwrap_or_default();
                                println!("{out}");
                                continue;
                            }
                        };
                        match retrieve_result(&result_store, id) {
                            Some(data) => JsonRpcResponse::success(request.id, data),
                            None => JsonRpcResponse::error(
                                request.id,
                                -32000,
                                format!("Result not found: {id}"),
                            ),
                        }
                    }
                    _ => JsonRpcResponse::error(
                        request.id,
                        -32601,
                        format!("Unknown tool: {tool_name}"),
                    ),
                }
            }
            "notifications/initialized" => {
                // Acknowledge initialization silently
                JsonRpcResponse::notification()
            }
            "shutdown" => {
                // Graceful shutdown — send empty response and exit
                let ok = JsonRpcResponse::success(request.id, serde_json::json!(null));
                let out = serde_json::to_string(&ok).unwrap_or_default();
                println!("{out}");
                return;
            }
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        };

        let out = serde_json::to_string(&response).unwrap_or_default();
        println!("{out}");
    }
}
