/// MCP stdio server implementation.
///
/// Reads JSON-RPC 2.0 messages from stdin, processes them, and writes
/// responses to stdout. Logging goes to stderr.
///
/// Lifecycle: initialize -> operation (tools/list, tools/call) -> shutdown (stdin closed)
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::db::Database;
use crate::storage::Format;

use super::tools::{all_tools, handle_tool_call};
use super::{
    InitializeParams, InitializeResult, JsonRpcError, JsonRpcMessage, JsonRpcResponse,
    ServerCapabilities, ServerInfo, ToolCallParams, ToolListResult, ToolsCapability,
    PROTOCOL_VERSION, SERVER_NAME,
};

/// State of the MCP server lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Waiting for initialize request
    AwaitingInit,
    /// Initialize response sent, waiting for initialized notification
    Initializing,
    /// Normal operation
    Ready,
    /// Server is shutting down
    Shutdown,
}

/// MCP server configuration.
pub struct McpServerConfig {
    pub vault_path: PathBuf,
    pub format: Format,
}

/// Runs the MCP server on stdin/stdout.
///
/// This is the main entry point called by `ztlgr mcp`.
/// Blocks until stdin is closed or an error occurs.
pub fn run_server(config: McpServerConfig) -> crate::error::Result<()> {
    let db_path = config.vault_path.join(".ztlgr").join("vault.db");
    let db = Database::new(&db_path)?;

    log_stderr(&format!(
        "ztlgr MCP server starting (vault: {})",
        config.vault_path.display()
    ));

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut state = ServerState::AwaitingInit;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                log_stderr(&format!("stdin read error: {}", e));
                break;
            }
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let msg: JsonRpcMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                let resp = JsonRpcResponse::error(
                    serde_json::Value::Null,
                    JsonRpcError::parse_error(&format!("Invalid JSON: {}", e)),
                );
                send_response(&stdout, &resp);
                continue;
            }
        };

        let response = process_message(&msg, &mut state, &config.vault_path, &db, config.format);

        if let Some(resp) = response {
            send_response(&stdout, &resp);
        }

        if state == ServerState::Shutdown {
            break;
        }
    }

    log_stderr("ztlgr MCP server shutting down");
    Ok(())
}

/// Processes a single JSON-RPC message and returns an optional response.
/// Notifications (no id) do not get a response.
fn process_message(
    msg: &JsonRpcMessage,
    state: &mut ServerState,
    vault_path: &Path,
    db: &Database,
    format: Format,
) -> Option<JsonRpcResponse> {
    let method = match &msg.method {
        Some(m) => m.as_str(),
        None => {
            // No method: could be a response from client, ignore
            return None;
        }
    };

    let is_notification = msg.id.is_none();

    match method {
        "initialize" => {
            if *state != ServerState::AwaitingInit {
                return msg.id.as_ref().map(|id| {
                    JsonRpcResponse::error(
                        id.clone(),
                        JsonRpcError::invalid_params("Already initialized"),
                    )
                });
            }

            let id = match &msg.id {
                Some(id) => id.clone(),
                None => {
                    return None; // initialize must be a request, not notification
                }
            };

            // Parse params (optional, we accept any client)
            let _params: Option<InitializeParams> = msg
                .params
                .as_ref()
                .and_then(|p| serde_json::from_value(p.clone()).ok());

            if let Some(ref params) = _params {
                log_stderr(&format!(
                    "Client: {} (protocol: {})",
                    params
                        .client_info
                        .as_ref()
                        .map(|i| i.name.as_str())
                        .unwrap_or("unknown"),
                    params.protocol_version
                ));
            }

            let result = InitializeResult {
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability {
                        list_changed: false,
                    },
                },
                server_info: ServerInfo {
                    name: SERVER_NAME.to_string(),
                    version: crate::VERSION.to_string(),
                },
                instructions: Some(format!(
                    "ztlgr grimoire MCP server. Vault: {}. Use tools to search, read, and create notes.",
                    vault_path.display()
                )),
            };

            *state = ServerState::Initializing;
            Some(JsonRpcResponse::success(
                id,
                serde_json::to_value(result).unwrap_or_default(),
            ))
        }

        "notifications/initialized" => {
            if *state == ServerState::Initializing {
                *state = ServerState::Ready;
                log_stderr("MCP session initialized, entering operation phase");
            }
            None // notifications don't get responses
        }

        "tools/list" => {
            if *state != ServerState::Ready {
                return msg.id.as_ref().map(|id| {
                    JsonRpcResponse::error(
                        id.clone(),
                        JsonRpcError::invalid_params("Server not initialized"),
                    )
                });
            }

            let id = match &msg.id {
                Some(id) => id.clone(),
                None => return None,
            };

            let result = ToolListResult {
                tools: all_tools(),
                next_cursor: None,
            };

            Some(JsonRpcResponse::success(
                id,
                serde_json::to_value(result).unwrap_or_default(),
            ))
        }

        "tools/call" => {
            if *state != ServerState::Ready {
                return msg.id.as_ref().map(|id| {
                    JsonRpcResponse::error(
                        id.clone(),
                        JsonRpcError::invalid_params("Server not initialized"),
                    )
                });
            }

            let id = match &msg.id {
                Some(id) => id.clone(),
                None => return None,
            };

            let params: ToolCallParams = match msg
                .params
                .as_ref()
                .and_then(|p| serde_json::from_value(p.clone()).ok())
            {
                Some(p) => p,
                None => {
                    return Some(JsonRpcResponse::error(
                        id,
                        JsonRpcError::invalid_params("Missing or invalid tool call params"),
                    ));
                }
            };

            log_stderr(&format!("Tool call: {}", params.name));

            let arguments = params.arguments.unwrap_or(serde_json::json!({}));
            let result = handle_tool_call(&params.name, &arguments, vault_path, db, format);

            Some(JsonRpcResponse::success(
                id,
                serde_json::to_value(result).unwrap_or_default(),
            ))
        }

        "ping" => msg
            .id
            .as_ref()
            .map(|id| JsonRpcResponse::success(id.clone(), serde_json::json!({}))),

        _ if is_notification => {
            // Unknown notifications are ignored per spec
            log_stderr(&format!("Ignoring unknown notification: {}", method));
            None
        }

        _ => msg
            .id
            .as_ref()
            .map(|id| JsonRpcResponse::error(id.clone(), JsonRpcError::method_not_found(method))),
    }
}

/// Sends a JSON-RPC response to stdout (newline-delimited).
fn send_response(stdout: &io::Stdout, response: &JsonRpcResponse) {
    let json = match serde_json::to_string(response) {
        Ok(j) => j,
        Err(e) => {
            log_stderr(&format!("Failed to serialize response: {}", e));
            return;
        }
    };

    let mut out = stdout.lock();
    if let Err(e) = writeln!(out, "{}", json) {
        log_stderr(&format!("Failed to write to stdout: {}", e));
    }
    let _ = out.flush();
}

/// Logs a message to stderr (visible to the MCP client for debugging).
fn log_stderr(msg: &str) {
    eprintln!("[ztlgr-mcp] {}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> (tempfile::TempDir, Database) {
        let temp = tempfile::TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        (temp, db)
    }

    fn init_msg(id: u64) -> JsonRpcMessage {
        serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {
                    "name": "TestClient",
                    "version": "1.0"
                }
            }
        }))
        .unwrap()
    }

    fn initialized_notification() -> JsonRpcMessage {
        serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .unwrap()
    }

    fn tools_list_msg(id: u64) -> JsonRpcMessage {
        serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/list",
            "params": {}
        }))
        .unwrap()
    }

    fn tools_call_msg(id: u64, name: &str, args: serde_json::Value) -> JsonRpcMessage {
        serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": args
            }
        }))
        .unwrap()
    }

    #[test]
    fn test_server_state_transitions() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        // Initialize
        let resp = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        assert_eq!(state, ServerState::Initializing);

        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(resp_val["result"]["protocolVersion"].is_string());
        assert_eq!(resp_val["result"]["serverInfo"]["name"], "ztlgr");

        // Initialized notification
        let resp = process_message(
            &initialized_notification(),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );
        assert!(resp.is_none()); // notifications don't get responses
        assert_eq!(state, ServerState::Ready);
    }

    #[test]
    fn test_double_initialize_rejected() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        // First init succeeds
        let _ = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        assert_eq!(state, ServerState::Initializing);

        // Second init fails
        let resp = process_message(&init_msg(2), &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(resp_val.get("error").is_some());
        assert!(resp_val["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Already initialized"));
    }

    #[test]
    fn test_tools_list_before_init_rejected() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        let resp = process_message(
            &tools_list_msg(1),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(resp_val.get("error").is_some());
    }

    #[test]
    fn test_tools_list_returns_all_tools() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        // Initialize
        let _ = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        let _ = process_message(
            &initialized_notification(),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );
        assert_eq!(state, ServerState::Ready);

        // List tools
        let resp = process_message(
            &tools_list_msg(2),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        let tools = resp_val["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 9);

        // Verify tool names
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"search"));
        assert!(names.contains(&"get_note"));
        assert!(names.contains(&"create_note"));
        assert!(names.contains(&"read_index"));
    }

    #[test]
    fn test_tools_call_search() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        // Initialize
        let _ = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        let _ = process_message(
            &initialized_notification(),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );

        // Create a note for search
        let note = crate::note::Note::new(
            "Rust Ownership".to_string(),
            "Rust ownership rules".to_string(),
        );
        db.create_note(&note).unwrap();

        // Call search
        let msg = tools_call_msg(3, "search", serde_json::json!({"query": "Rust"}));
        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        let content = &resp_val["result"]["content"][0]["text"];
        assert!(content.as_str().unwrap().contains("Rust Ownership"));
    }

    #[test]
    fn test_tools_call_unknown_tool() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        let _ = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        let _ = process_message(
            &initialized_notification(),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );

        let msg = tools_call_msg(3, "unknown_tool", serde_json::json!({}));
        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        let is_error = resp_val["result"]["isError"].as_bool().unwrap_or(false);
        assert!(is_error);
    }

    #[test]
    fn test_tools_call_before_init() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        let msg = tools_call_msg(1, "search", serde_json::json!({"query": "test"}));
        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(resp_val.get("error").is_some());
    }

    #[test]
    fn test_ping_response() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        let msg: JsonRpcMessage = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "ping"
        }))
        .unwrap();

        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert_eq!(resp_val["id"], 99);
        assert!(resp_val["result"].is_object());
    }

    #[test]
    fn test_unknown_method_returns_error() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::Ready;

        let msg: JsonRpcMessage = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/list"
        }))
        .unwrap();

        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(resp_val.get("error").is_some());
        assert_eq!(resp_val["error"]["code"], -32601);
    }

    #[test]
    fn test_unknown_notification_ignored() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::Ready;

        let msg: JsonRpcMessage = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/some_unknown"
        }))
        .unwrap();

        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_none()); // notifications are ignored
    }

    #[test]
    fn test_message_without_method_ignored() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::Ready;

        let msg: JsonRpcMessage = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"ok": true}
        }))
        .unwrap();

        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_none());
    }

    #[test]
    fn test_initialize_response_has_correct_structure() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        let resp = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();

        // Check JSON-RPC envelope
        assert_eq!(resp_val["jsonrpc"], "2.0");
        assert_eq!(resp_val["id"], 1);

        // Check MCP initialize result
        let result = &resp_val["result"];
        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(result["serverInfo"]["name"], SERVER_NAME);
        assert!(result["serverInfo"]["version"].is_string());
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["instructions"].is_string());
    }

    #[test]
    fn test_tools_call_with_missing_params() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::Ready;

        // tools/call without params
        let msg: JsonRpcMessage = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call"
        }))
        .unwrap();

        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let resp_val = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(resp_val.get("error").is_some());
        assert!(resp_val["error"]["message"]
            .as_str()
            .unwrap()
            .contains("params"));
    }

    #[test]
    fn test_full_lifecycle_search_flow() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        // 1. Initialize
        let resp = process_message(&init_msg(1), &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        assert_eq!(state, ServerState::Initializing);

        // 2. Initialized
        let resp = process_message(
            &initialized_notification(),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );
        assert!(resp.is_none());
        assert_eq!(state, ServerState::Ready);

        // 3. List tools
        let resp = process_message(
            &tools_list_msg(2),
            &mut state,
            vault_path,
            &db,
            Format::Markdown,
        );
        assert!(resp.is_some());
        let tools = serde_json::to_value(resp.unwrap()).unwrap();
        assert!(tools["result"]["tools"].as_array().unwrap().len() > 0);

        // 4. Create note via tool
        let create_msg = tools_call_msg(
            3,
            "create_note",
            serde_json::json!({
                "title": "Lifecycle Test",
                "content": "Created during MCP lifecycle test"
            }),
        );
        // Need permanent dir for file creation
        std::fs::create_dir_all(vault_path.join("permanent")).unwrap();
        let resp = process_message(&create_msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let create_result = serde_json::to_value(resp.unwrap()).unwrap();
        let text = create_result["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("created successfully"));

        // 5. Search for the created note
        let search_msg = tools_call_msg(4, "search", serde_json::json!({"query": "Lifecycle"}));
        let resp = process_message(&search_msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let search_result = serde_json::to_value(resp.unwrap()).unwrap();
        let text = search_result["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("Lifecycle Test"));
    }

    #[test]
    fn test_tools_call_read_index() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::Ready;

        // Create index file
        let ztlgr_dir = vault_path.join(".ztlgr");
        std::fs::create_dir_all(&ztlgr_dir).unwrap();
        std::fs::write(
            ztlgr_dir.join("index.md"),
            "# Grimoire Index\n\nNotes here.",
        )
        .unwrap();

        let msg = tools_call_msg(1, "read_index", serde_json::json!({}));
        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_some());
        let val = serde_json::to_value(resp.unwrap()).unwrap();
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Grimoire Index"));
    }

    #[test]
    fn test_initialize_notification_only_is_ignored() {
        let (temp, db) = make_db();
        let vault_path = temp.path();
        let mut state = ServerState::AwaitingInit;

        // initialize as notification (no id) should be ignored
        let msg: JsonRpcMessage = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26"
            }
        }))
        .unwrap();

        let resp = process_message(&msg, &mut state, vault_path, &db, Format::Markdown);
        assert!(resp.is_none());
        assert_eq!(state, ServerState::AwaitingInit); // state unchanged
    }
}
