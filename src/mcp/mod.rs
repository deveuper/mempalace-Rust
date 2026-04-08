#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::storage::{Embedder, VectorStore};

pub mod protocol;
pub mod tools;

use protocol::{JsonRpcRequest, JsonRpcResponse, McpError};
use tools::McpTools;

/// Run the MCP server
pub async fn run_server(config: &Config, transport: &str) -> Result<()> {
    info!("Starting MemPalace MCP server");
    info!("Transport: {}", transport);

    let embedder = Embedder::new(config.embedding_model())?;
    let store = VectorStore::open(&config.db_path, embedder)?;
    let tools = McpTools::new(store);

    match transport {
        "stdio" => run_stdio_server(tools).await,
        "http" => {
            warn!("HTTP transport not yet implemented, falling back to stdio");
            run_stdio_server(tools).await
        }
        _ => Err(anyhow::anyhow!("Unknown transport: {}", transport)),
    }
}

/// Run MCP server over stdio
async fn run_stdio_server(tools: McpTools) -> Result<()> {
    info!("MCP server listening on stdio");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Send initialization response
    let init_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 0,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "mempalace",
                "version": "3.0.0"
            }
        }
    });

    writeln!(stdout, "{}", serde_json::to_string(&init_response)?)?;
    stdout.flush()?;

    // Process incoming requests
    for line in stdin.lock().lines() {
        let line = line?;
        debug!("Received: {}", line);

        match handle_request(&line, &tools).await {
            Ok(response) => {
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
            Err(e) => {
                error!("Error handling request: {}", e);
                let error_response = JsonRpcResponse::error(
                    None,
                    McpError::internal_error(e.to_string()),
                );
                writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

/// Handle a single MCP request
async fn handle_request(line: &str, tools: &McpTools) -> Result<JsonRpcResponse> {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(req) => req,
        Err(e) => {
            return Ok(JsonRpcResponse::error(
                None,
                McpError::parse_error(e.to_string()),
            ));
        }
    };

    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => handle_initialize(id).await,
        "tools/list" => handle_tools_list(id).await,
        "tools/call" => handle_tools_call(id, request.params, tools).await,
        _ => Ok(JsonRpcResponse::error(
            id,
            McpError::method_not_found(request.method),
        )),
    }
}

/// Handle initialize request
async fn handle_initialize(id: Option<Value>) -> Result<JsonRpcResponse> {
    Ok(JsonRpcResponse::success(id, serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "mempalace",
            "version": "3.0.0"
        }
    })))
}

/// Handle tools/list request
async fn handle_tools_list(id: Option<Value>) -> Result<JsonRpcResponse> {
    let tools = serde_json::json!([
        {
            "name": "mempalace_search",
            "description": "Search the memory palace for relevant information",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "wing": {
                        "type": "string",
                        "description": "Optional wing filter"
                    },
                    "room": {
                        "type": "string",
                        "description": "Optional room filter"
                    },
                    "n_results": {
                        "type": "integer",
                        "description": "Number of results to return",
                        "default": 5
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "mempalace_wake_up",
            "description": "Get the wake-up context for the AI",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "wing": {
                        "type": "string",
                        "description": "Optional wing for specific context"
                    }
                }
            }
        },
        {
            "name": "mempalace_status",
            "description": "Get the status of the memory palace",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ]);

    Ok(JsonRpcResponse::success(id, tools))
}

/// Handle tools/call request
async fn handle_tools_call(
    id: Option<Value>,
    params: Option<Value>,
    tools: &McpTools,
) -> Result<JsonRpcResponse> {
    let params = params.unwrap_or(serde_json::json!({}));

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    let result = match name {
        "mempalace_search" => tools.search(arguments).await,
        "mempalace_wake_up" => tools.wake_up(arguments).await,
        "mempalace_status" => tools.status().await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    };

    match result {
        Ok(content) => Ok(JsonRpcResponse::success(id, content)),
        Err(e) => Ok(JsonRpcResponse::error(
            id,
            McpError::internal_error(e.to_string()),
        )),
    }
}
